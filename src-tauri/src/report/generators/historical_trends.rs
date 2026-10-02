use crate::ledger::currency::{convert_minor_units, normalize_fx_rate, DEFAULT_FX_RATE};
use crate::report::dto::{DailyTrendPoint, HistoricalTrendsReport};
use crate::shared::AppError;
use chrono::{Duration, NaiveDate};
use rusqlite::Connection;
use std::collections::HashMap;

struct AccountMeta {
    account_type: String,
    currency: String,
    is_liquid_cash: bool,
}

pub fn generate(
    conn: &Connection,
    from_date: &str,
    to_date: &str,
    fx_rate: i64,
) -> Result<HistoricalTrendsReport, AppError> {
    let current_fx_rate = normalize_fx_rate(fx_rate, DEFAULT_FX_RATE);

    let start = NaiveDate::parse_from_str(from_date, "%Y-%m-%d")
        .map_err(|e| AppError::InvalidInput(format!("invalid from_date: {e}")))?;
    let end = NaiveDate::parse_from_str(to_date, "%Y-%m-%d")
        .map_err(|e| AppError::InvalidInput(format!("invalid to_date: {e}")))?;

    if start > end {
        return Err(AppError::InvalidInput(
            "from_date cannot be after to_date".into(),
        ));
    }

    let mut acct_stmt = conn.prepare(
        "SELECT id, code, name, currency, type
         FROM accounts
         WHERE placeholder = 0;",
    )?;

    let mut accounts: HashMap<String, AccountMeta> = HashMap::new();
    let rows = acct_stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;

    for r in rows {
        let (id, code, name, currency, acc_type) = r?;
        let name_lower = name.to_lowercase();
        let is_liquid = acc_type == "ASSET"
            && (code.starts_with("101")
                || code.starts_with("1020")
                || name_lower.contains("cash")
                || name_lower.contains("kas")
                || name_lower.contains("bank"));

        accounts.insert(
            id,
            AccountMeta {
                account_type: acc_type,
                currency,
                is_liquid_cash: is_liquid,
            },
        );
    }

    let mut post_stmt = conn.prepare(
        "SELECT je.date, p.account_id, p.amount
         FROM postings p
         JOIN journal_entries je ON je.id = p.entry_id
         WHERE je.date <= ?1
         ORDER BY je.date ASC, je.id ASC;",
    )?;

    let post_rows = post_stmt.query_map(rusqlite::params![to_date], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;

    let mut prior_postings = Vec::new();
    let mut range_postings_by_date: HashMap<String, Vec<(String, i64)>> = HashMap::new();

    for r in post_rows {
        let (date, account_id, amount) = r?;
        if date.as_str() < from_date {
            prior_postings.push((account_id, amount));
        } else {
            range_postings_by_date
                .entry(date)
                .or_default()
                .push((account_id, amount));
        }
    }

    let mut assets_by_curr: HashMap<String, i64> = HashMap::new();
    let mut liabilities_by_curr: HashMap<String, i64> = HashMap::new();
    let mut liquid_cash_by_curr: HashMap<String, i64> = HashMap::new();

    let apply_posting = |acc: &AccountMeta,
                         amount: i64,
                         assets: &mut HashMap<String, i64>,
                         liabilities: &mut HashMap<String, i64>,
                         liquid: &mut HashMap<String, i64>| {
        let curr = acc.currency.trim().to_uppercase();
        if acc.account_type == "ASSET" {
            let e = assets.entry(curr.clone()).or_insert(0);
            *e = e.saturating_add(amount);
            if acc.is_liquid_cash {
                let le = liquid.entry(curr).or_insert(0);
                *le = le.saturating_add(amount);
            }
        } else if acc.account_type == "LIABILITY" {
            let e = liabilities.entry(curr).or_insert(0);
            *e = e.saturating_sub(amount);
        }
    };

    for (acc_id, amount) in prior_postings {
        if let Some(acc) = accounts.get(&acc_id) {
            apply_posting(
                acc,
                amount,
                &mut assets_by_curr,
                &mut liabilities_by_curr,
                &mut liquid_cash_by_curr,
            );
        }
    }

    let mut points = Vec::new();
    let mut current_date = start;

    while current_date <= end {
        let date_str = current_date.format("%Y-%m-%d").to_string();

        if let Some(day_postings) = range_postings_by_date.get(&date_str) {
            for (acc_id, amount) in day_postings {
                if let Some(acc) = accounts.get(acc_id) {
                    apply_posting(
                        acc,
                        *amount,
                        &mut assets_by_curr,
                        &mut liabilities_by_curr,
                        &mut liquid_cash_by_curr,
                    );
                }
            }
        }

        let assets: i64 = assets_by_curr
            .iter()
            .map(|(curr, amt)| convert_minor_units(*amt, curr, "IDR", current_fx_rate))
            .sum();
        let liabilities: i64 = liabilities_by_curr
            .iter()
            .map(|(curr, amt)| convert_minor_units(*amt, curr, "IDR", current_fx_rate))
            .sum();
        let liquid_cash: i64 = liquid_cash_by_curr
            .iter()
            .map(|(curr, amt)| convert_minor_units(*amt, curr, "IDR", current_fx_rate))
            .sum();
        let net_worth = assets.saturating_sub(liabilities);

        points.push(DailyTrendPoint {
            date: date_str,
            net_worth,
            assets,
            liabilities,
            liquid_cash,
        });

        current_date += Duration::days(1);
    }

    Ok(HistoricalTrendsReport {
        from_date: from_date.to_string(),
        to_date: to_date.to_string(),
        points,
    })
}

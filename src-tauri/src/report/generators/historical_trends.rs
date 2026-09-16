use crate::ledger::currency::{normalize_fx_rate, usd_minor_to_idr, DEFAULT_FX_RATE};
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

    let mut assets_idr: i64 = 0;
    let mut assets_usd: i64 = 0;
    let mut liabilities_idr: i64 = 0;
    let mut liabilities_usd: i64 = 0;
    let mut liquid_cash_idr: i64 = 0;
    let mut liquid_cash_usd: i64 = 0;

    let apply_posting = |acc: &AccountMeta,
                         amount: i64,
                         assets_idr: &mut i64,
                         assets_usd: &mut i64,
                         liabilities_idr: &mut i64,
                         liabilities_usd: &mut i64,
                         liquid_cash_idr: &mut i64,
                         liquid_cash_usd: &mut i64| {
        if acc.account_type == "ASSET" {
            if acc.currency == "USD" {
                *assets_usd += amount;
            } else {
                *assets_idr += amount;
            }
            if acc.is_liquid_cash {
                if acc.currency == "USD" {
                    *liquid_cash_usd += amount;
                } else {
                    *liquid_cash_idr += amount;
                }
            }
        } else if acc.account_type == "LIABILITY" {
            if acc.currency == "USD" {
                *liabilities_usd -= amount;
            } else {
                *liabilities_idr -= amount;
            }
        }
    };

    for (acc_id, amount) in prior_postings {
        if let Some(acc) = accounts.get(&acc_id) {
            apply_posting(
                acc,
                amount,
                &mut assets_idr,
                &mut assets_usd,
                &mut liabilities_idr,
                &mut liabilities_usd,
                &mut liquid_cash_idr,
                &mut liquid_cash_usd,
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
                        &mut assets_idr,
                        &mut assets_usd,
                        &mut liabilities_idr,
                        &mut liabilities_usd,
                        &mut liquid_cash_idr,
                        &mut liquid_cash_usd,
                    );
                }
            }
        }

        let assets = assets_idr + usd_minor_to_idr(assets_usd, current_fx_rate);
        let liabilities = liabilities_idr + usd_minor_to_idr(liabilities_usd, current_fx_rate);
        let liquid_cash = liquid_cash_idr + usd_minor_to_idr(liquid_cash_usd, current_fx_rate);
        let net_worth = assets - liabilities;

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

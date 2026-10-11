use crate::ledger::currency::{convert_minor_units, normalize_fx_rate, DEFAULT_FX_RATE};
use crate::report::dto::{AccountReportRow, ProfitLossReport};
use crate::shared::AppError;
use rusqlite::Connection;

pub fn generate(
    conn: &Connection,
    from_date: Option<&str>,
    to_date: Option<&str>,
) -> Result<ProfitLossReport, AppError> {
    let spot_fx: i64 = conn
        .query_row(
            "SELECT fx_rate FROM journal_entries WHERE fx_rate > 0 AND fx_rate != 1000000 ORDER BY date DESC, posted_at DESC LIMIT 1;",
            [],
            |r| r.get(0),
        )
        .unwrap_or(DEFAULT_FX_RATE);

    let mut stmt = conn.prepare(
        "SELECT a.id, a.code, a.name, a.currency, a.type, COALESCE(SUM(p.amount), 0)
         FROM accounts a
         LEFT JOIN postings p ON p.account_id = a.id
         LEFT JOIN journal_entries je ON je.id = p.entry_id
         WHERE a.type IN ('INCOME', 'EXPENSE')
           AND a.placeholder = 0
           AND (?1 IS NULL OR je.date >= ?1)
           AND (?2 IS NULL OR je.date <= ?2)
         GROUP BY a.id
         ORDER BY a.code ASC;",
    )?;

    let rows = stmt.query_map(rusqlite::params![from_date, to_date], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, i64>(5)?,
        ))
    })?;

    let mut income_rows = Vec::new();
    let mut expense_rows = Vec::new();

    for r in rows {
        let (account_id, code, name, currency, acc_type, raw_balance) = r?;

        if acc_type == "INCOME" {
            let amount = -raw_balance;
            if amount != 0 {
                income_rows.push(AccountReportRow {
                    account_id,
                    code,
                    name,
                    currency,
                    amount,
                });
            }
        } else if acc_type == "EXPENSE" {
            let amount = raw_balance;
            if amount != 0 {
                expense_rows.push(AccountReportRow {
                    account_id,
                    code,
                    name,
                    currency,
                    amount,
                });
            }
        }
    }

    // Compute totals normalized to base currency (IDR)
    let mut post_stmt = conn.prepare(
        "SELECT a.type, COALESCE(p.currency, a.currency), p.amount, COALESCE(p.fx_rate, je.fx_rate), p.cost_amount
         FROM postings p
         JOIN accounts a ON p.account_id = a.id
         JOIN journal_entries je ON je.id = p.entry_id
         WHERE a.type IN ('INCOME', 'EXPENSE')
           AND a.placeholder = 0
           AND (?1 IS NULL OR je.date >= ?1)
           AND (?2 IS NULL OR je.date <= ?2);",
    )?;

    let post_rows = post_stmt.query_map(rusqlite::params![from_date, to_date], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, Option<i64>>(4)?,
        ))
    })?;

    let mut total_income: i64 = 0;
    let mut total_expenses: i64 = 0;

    for pr in post_rows {
        let (acc_type, curr, amount, entry_fx, cost_amount) = pr?;
        let in_idr = if let Some(cost) = cost_amount {
            amount.signum() * cost.abs()
        } else if curr != "IDR" {
            convert_minor_units(amount, &curr, "IDR", normalize_fx_rate(entry_fx, spot_fx))
        } else {
            amount
        };

        if acc_type == "INCOME" {
            total_income = total_income.saturating_add(-in_idr);
        } else if acc_type == "EXPENSE" {
            total_expenses = total_expenses.saturating_add(in_idr);
        }
    }

    let net_income = total_income - total_expenses;

    Ok(ProfitLossReport {
        from_date: from_date.map(ToString::to_string),
        to_date: to_date.map(ToString::to_string),
        income_rows,
        expense_rows,
        total_income,
        total_expenses,
        net_income,
    })
}

use crate::ledger::currency::{convert_minor_units, normalize_fx_rate, DEFAULT_FX_RATE};
use crate::report::dto::{TrialBalanceReport, TrialBalanceRow};
use crate::shared::AppError;
use rusqlite::Connection;

pub fn generate(
    conn: &Connection,
    as_of_date: Option<&str>,
) -> Result<TrialBalanceReport, AppError> {
    let spot_fx: i64 = conn
        .query_row(
            "SELECT fx_rate FROM journal_entries WHERE fx_rate > 0 AND fx_rate != 1000000 ORDER BY date DESC, posted_at DESC LIMIT 1;",
            [],
            |r| r.get(0),
        )
        .unwrap_or(DEFAULT_FX_RATE);

    let mut stmt = conn.prepare(
        "SELECT a.id, a.code, a.name, a.type, a.currency, COALESCE(SUM(p.amount), 0)
         FROM accounts a
         LEFT JOIN postings p ON p.account_id = a.id
         LEFT JOIN journal_entries je ON je.id = p.entry_id
         WHERE a.placeholder = 0
           AND (?1 IS NULL OR je.date <= ?1)
         GROUP BY a.id
         ORDER BY a.code ASC;",
    )?;

    let rows = stmt.query_map(rusqlite::params![as_of_date], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, i64>(5)?,
        ))
    })?;

    let mut tb_rows = Vec::new();

    for r in rows {
        let (account_id, code, name, account_type, currency, raw_balance) = r?;

        if raw_balance == 0 {
            continue;
        }

        let (debit, credit) = if raw_balance > 0 {
            (raw_balance, 0)
        } else {
            (0, raw_balance.abs())
        };

        tb_rows.push(TrialBalanceRow {
            account_id,
            code,
            name,
            account_type,
            currency,
            debit,
            credit,
        });
    }

    // Compute total debit and credit normalized in base currency (IDR)
    let mut post_stmt = conn.prepare(
        "SELECT COALESCE(p.currency, a.currency), p.amount, COALESCE(p.fx_rate, je.fx_rate), p.cost_amount
         FROM postings p
         JOIN accounts a ON p.account_id = a.id
         JOIN journal_entries je ON je.id = p.entry_id
         WHERE a.placeholder = 0
           AND (?1 IS NULL OR je.date <= ?1);",
    )?;

    let post_rows = post_stmt.query_map(rusqlite::params![as_of_date], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, Option<i64>>(3)?,
        ))
    })?;

    let mut total_debit: i64 = 0;
    let mut total_credit: i64 = 0;

    for pr in post_rows {
        let (curr, amount, entry_fx, cost_amount) = pr?;
        let in_idr = if let Some(cost) = cost_amount {
            amount.signum() * cost.abs()
        } else if curr != "IDR" {
            convert_minor_units(amount, &curr, "IDR", normalize_fx_rate(entry_fx, spot_fx))
        } else {
            amount
        };

        if in_idr > 0 {
            total_debit = total_debit.saturating_add(in_idr);
        } else {
            total_credit = total_credit.saturating_add(in_idr.abs());
        }
    }

    let is_balanced = total_debit == total_credit;

    Ok(TrialBalanceReport {
        as_of_date: as_of_date.map(ToString::to_string),
        rows: tb_rows,
        total_debit,
        total_credit,
        is_balanced,
    })
}

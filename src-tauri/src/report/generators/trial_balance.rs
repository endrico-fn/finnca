use crate::report::dto::{TrialBalanceReport, TrialBalanceRow};
use crate::shared::AppError;
use rusqlite::Connection;

pub fn generate(
    conn: &Connection,
    as_of_date: Option<&str>,
) -> Result<TrialBalanceReport, AppError> {
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
    let mut total_debit: i64 = 0;
    let mut total_credit: i64 = 0;

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

        total_debit += debit;
        total_credit += credit;

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

    let is_balanced = total_debit == total_credit;

    Ok(TrialBalanceReport {
        as_of_date: as_of_date.map(ToString::to_string),
        rows: tb_rows,
        total_debit,
        total_credit,
        is_balanced,
    })
}

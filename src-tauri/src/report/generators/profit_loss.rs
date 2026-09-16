use crate::report::dto::{AccountReportRow, ProfitLossReport};
use crate::shared::AppError;
use rusqlite::Connection;

pub fn generate(
    conn: &Connection,
    from_date: Option<&str>,
    to_date: Option<&str>,
) -> Result<ProfitLossReport, AppError> {
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
    let mut total_income: i64 = 0;
    let mut total_expenses: i64 = 0;

    for r in rows {
        let (account_id, code, name, currency, acc_type, raw_balance) = r?;

        if acc_type == "INCOME" {
            let amount = -raw_balance;
            if amount != 0 {
                total_income += amount;
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
                total_expenses += amount;
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

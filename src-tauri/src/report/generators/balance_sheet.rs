use crate::report::dto::{AccountReportRow, BalanceSheetReport};
use crate::shared::AppError;
use rusqlite::Connection;

pub fn generate(
    conn: &Connection,
    as_of_date: Option<&str>,
) -> Result<BalanceSheetReport, AppError> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.code, a.name, a.currency, a.type, COALESCE(SUM(p.amount), 0)
         FROM accounts a
         LEFT JOIN postings p ON p.account_id = a.id
         LEFT JOIN journal_entries je ON je.id = p.entry_id
         WHERE a.type IN ('ASSET', 'LIABILITY', 'EQUITY', 'INCOME', 'EXPENSE')
           AND a.placeholder = 0
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

    let mut asset_rows = Vec::new();
    let mut liability_rows = Vec::new();
    let mut equity_rows = Vec::new();
    let mut total_assets: i64 = 0;
    let mut total_liabilities: i64 = 0;
    let mut total_equity: i64 = 0;
    let mut total_income: i64 = 0;
    let mut total_expenses: i64 = 0;

    for r in rows {
        let (account_id, code, name, currency, acc_type, raw_balance) = r?;

        match acc_type.as_str() {
            "ASSET" => {
                let amount = raw_balance;
                if amount != 0 {
                    total_assets += amount;
                    asset_rows.push(AccountReportRow {
                        account_id,
                        code,
                        name,
                        currency,
                        amount,
                    });
                }
            }
            "LIABILITY" => {
                let amount = -raw_balance;
                if amount != 0 {
                    total_liabilities += amount;
                    liability_rows.push(AccountReportRow {
                        account_id,
                        code,
                        name,
                        currency,
                        amount,
                    });
                }
            }
            "EQUITY" => {
                let amount = -raw_balance;
                if amount != 0 {
                    total_equity += amount;
                    equity_rows.push(AccountReportRow {
                        account_id,
                        code,
                        name,
                        currency,
                        amount,
                    });
                }
            }
            "INCOME" => {
                total_income += -raw_balance;
            }
            "EXPENSE" => {
                total_expenses += raw_balance;
            }
            _ => {}
        }
    }

    let net_income = total_income - total_expenses;
    let discrepancy = total_assets - (total_liabilities + total_equity + net_income);
    let is_balanced = discrepancy == 0;

    Ok(BalanceSheetReport {
        as_of_date: as_of_date.map(ToString::to_string),
        asset_rows,
        liability_rows,
        equity_rows,
        total_assets,
        total_liabilities,
        total_equity,
        net_income,
        discrepancy,
        is_balanced,
    })
}

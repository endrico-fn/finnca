use crate::report::dto::{CashFlowActivityRow, CashFlowReport};
use crate::shared::AppError;
use rusqlite::Connection;

pub fn generate(
    conn: &Connection,
    from_date: Option<&str>,
    to_date: Option<&str>,
) -> Result<CashFlowReport, AppError> {
    let starting_cash: i64 = if let Some(from) = from_date {
        conn.query_row(
            "SELECT COALESCE(SUM(p.amount), 0)
             FROM postings p
             JOIN accounts a ON a.id = p.account_id
             JOIN journal_entries je ON je.id = p.entry_id
             WHERE a.type = 'ASSET' AND a.placeholder = 0 AND je.date < ?1;",
            rusqlite::params![from],
            |r| r.get(0),
        )?
    } else {
        0
    };

    let mut stmt = conn.prepare(
        "SELECT je.id, je.description, p.amount, a.type
         FROM postings p
         JOIN accounts a ON a.id = p.account_id
         JOIN journal_entries je ON je.id = p.entry_id
         WHERE (?1 IS NULL OR je.date >= ?1)
           AND (?2 IS NULL OR je.date <= ?2)
         ORDER BY je.date ASC;",
    )?;

    let rows = stmt.query_map(rusqlite::params![from_date, to_date], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    let mut operating_cash_flow: i64 = 0;
    let investing_cash_flow: i64 = 0;
    let mut financing_cash_flow: i64 = 0;
    let mut operating_rows = Vec::new();

    for r in rows {
        let (_entry_id, description, amount, acc_type) = r?;

        match acc_type.as_str() {
            "INCOME" => {
                let cash_in = -amount;
                operating_cash_flow += cash_in;
                operating_rows.push(CashFlowActivityRow {
                    category: "OPERATING_INCOME".into(),
                    description,
                    amount: cash_in,
                });
            }
            "EXPENSE" => {
                let cash_out = -amount;
                operating_cash_flow += cash_out;
                operating_rows.push(CashFlowActivityRow {
                    category: "OPERATING_EXPENSE".into(),
                    description,
                    amount: cash_out,
                });
            }
            "LIABILITY" | "EQUITY" => {
                let change = -amount;
                financing_cash_flow += change;
            }
            _ => {}
        }
    }

    let net_cash_change = operating_cash_flow + investing_cash_flow + financing_cash_flow;
    let ending_cash = starting_cash + net_cash_change;

    Ok(CashFlowReport {
        from_date: from_date.map(ToString::to_string),
        to_date: to_date.map(ToString::to_string),
        starting_cash,
        operating_cash_flow,
        investing_cash_flow,
        financing_cash_flow,
        net_cash_change,
        ending_cash,
        operating_rows,
    })
}

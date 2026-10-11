use crate::ledger::currency::{convert_minor_units, normalize_fx_rate, DEFAULT_FX_RATE};
use crate::report::dto::MonthlyCashflowPoint;
use crate::shared::AppError;
use rusqlite::Connection;
use std::collections::BTreeMap;

pub fn generate(conn: &Connection, months: u32) -> Result<Vec<MonthlyCashflowPoint>, AppError> {
    let limit = months.clamp(1, 60) as usize;
    let spot_fx: i64 = conn
        .query_row(
            "SELECT fx_rate FROM journal_entries WHERE fx_rate > 0 AND fx_rate != 1000000 ORDER BY date DESC, posted_at DESC LIMIT 1;",
            [],
            |r| r.get(0),
        )
        .unwrap_or(DEFAULT_FX_RATE);

    let mut stmt = conn.prepare(
        "SELECT strftime('%Y-%m', je.date) AS m,
                a.type,
                COALESCE(p.currency, a.currency, 'IDR'),
                p.amount,
                COALESCE(p.fx_rate, je.fx_rate, 16000),
                p.cost_amount
         FROM journal_entries je
         JOIN postings p ON p.entry_id = je.id
         JOIN accounts a ON a.id = p.account_id
         WHERE a.type IN ('INCOME', 'EXPENSE')
           AND a.placeholder = 0
         ORDER BY m DESC;",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, Option<String>>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, Option<i64>>(5)?,
        ))
    })?;

    let mut month_totals: BTreeMap<String, (i64, i64)> = BTreeMap::new();

    for r in rows {
        let (month_opt, acc_type, curr, amount, entry_fx, cost_amount) = r?;
        let month = match month_opt {
            Some(m) if !m.is_empty() => m,
            _ => continue,
        };

        let in_idr = if let Some(cost) = cost_amount {
            amount.signum() * cost.abs()
        } else if curr != "IDR" {
            convert_minor_units(amount, &curr, "IDR", normalize_fx_rate(entry_fx, spot_fx))
        } else {
            amount
        };

        let entry = month_totals.entry(month).or_insert((0, 0));
        if acc_type == "INCOME" {
            entry.0 = entry.0.saturating_add(-in_idr);
        } else if acc_type == "EXPENSE" {
            entry.1 = entry.1.saturating_add(in_idr);
        }
    }

    let mut points: Vec<MonthlyCashflowPoint> = month_totals
        .into_iter()
        .rev()
        .take(limit)
        .map(|(month, (income, expense))| MonthlyCashflowPoint {
            month,
            income,
            expense,
            net: income.saturating_sub(expense),
        })
        .collect();

    points.reverse();
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monthly_cashflow_empty_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE accounts (id TEXT PRIMARY KEY, type TEXT, currency TEXT DEFAULT 'IDR', placeholder INTEGER);
             CREATE TABLE journal_entries (id TEXT PRIMARY KEY, date TEXT, fx_rate INTEGER DEFAULT 16000, posted_at INTEGER DEFAULT 0);
             CREATE TABLE postings (id TEXT PRIMARY KEY, entry_id TEXT, account_id TEXT, amount INTEGER, currency TEXT DEFAULT 'IDR', fx_rate INTEGER, cost_amount INTEGER);",
        ).unwrap();

        let res = generate(&conn, 6).unwrap();
        assert_eq!(res.len(), 0);
    }
}

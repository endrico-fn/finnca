use crate::ledger::currency::{normalize_fx_rate, usd_minor_to_idr, DEFAULT_FX_RATE};
use crate::report::dto::{FxRevaluationItem, FxRevaluationReport};
use crate::shared::AppError;
use rusqlite::Connection;

pub fn generate(
    conn: &Connection,
    as_of_date: Option<&str>,
    fx_rate: i64,
) -> Result<FxRevaluationReport, AppError> {
    let current_fx_rate = normalize_fx_rate(fx_rate, DEFAULT_FX_RATE);

    let mut acct_stmt = conn.prepare(
        "SELECT id, code, name, currency
         FROM accounts
         WHERE currency != 'IDR' AND placeholder = 0
         ORDER BY code ASC;",
    )?;

    let accounts = acct_stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    let mut items = Vec::new();
    let mut total_cost_basis_idr: i64 = 0;
    let mut total_current_value_idr: i64 = 0;
    let mut total_unrealized_gain_idr: i64 = 0;

    let mut post_stmt = conn.prepare(
        "SELECT je.date, je.fx_rate, p.amount
         FROM postings p
         JOIN journal_entries je ON je.id = p.entry_id
         WHERE p.account_id = ?1 AND (?2 IS NULL OR je.date <= ?2)
         ORDER BY je.date ASC, je.id ASC;",
    )?;

    for a in accounts {
        let (account_id, code, name, currency) = a?;

        let rows = post_stmt.query_map(rusqlite::params![account_id, as_of_date], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        let mut native_balance: i64 = 0;
        let mut remaining_qty: i64 = 0;
        let mut cost_basis_idr: i64 = 0;

        for r in rows {
            let (_date, entry_fx, amount) = r?;
            native_balance += amount;
            let tx_rate = normalize_fx_rate(entry_fx, current_fx_rate);
            let amount_idr = usd_minor_to_idr(amount, tx_rate);

            if amount > 0 {
                cost_basis_idr += amount_idr;
                remaining_qty += amount;
            } else if amount < 0 && remaining_qty > 0 {
                let deduction = ((cost_basis_idr as i128 * (-amount) as i128)
                    / remaining_qty as i128) as i64;
                cost_basis_idr -= deduction;
                remaining_qty += amount;
            } else {
                cost_basis_idr += amount_idr;
                remaining_qty += amount;
            }
        }

        if native_balance != 0 {
            let current_value_idr = usd_minor_to_idr(native_balance, current_fx_rate);
            let unrealized_gain_idr = current_value_idr - cost_basis_idr;

            total_cost_basis_idr += cost_basis_idr;
            total_current_value_idr += current_value_idr;
            total_unrealized_gain_idr += unrealized_gain_idr;

            items.push(FxRevaluationItem {
                account_id,
                code,
                name,
                currency,
                native_balance,
                cost_basis_idr,
                current_value_idr,
                unrealized_gain_idr,
            });
        }
    }

    Ok(FxRevaluationReport {
        as_of_date: as_of_date.map(ToString::to_string),
        current_fx_rate,
        items,
        total_cost_basis_idr,
        total_current_value_idr,
        total_unrealized_gain_idr,
    })
}

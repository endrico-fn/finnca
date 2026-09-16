use super::models::BudgetAllocation;
use crate::shared::AppError;
use rusqlite::{params, Connection, Row};

fn map_row(row: &Row<'_>) -> rusqlite::Result<BudgetAllocation> {
    Ok(BudgetAllocation {
        id: row.get(0)?,
        month: row.get(1)?,
        account_id: row.get(2)?,
        amount: row.get(3)?,
        created_at: row.get(4)?,
    })
}

pub fn upsert(conn: &Connection, budget: &BudgetAllocation) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO budgets (id, month, account_id, amount, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(month, account_id) DO UPDATE SET
             id = excluded.id,
             amount = excluded.amount;",
        params![
            budget.id,
            budget.month,
            budget.account_id,
            budget.amount,
            budget.created_at,
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM budgets WHERE id = ?1;", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("budget id={id}")));
    }
    Ok(())
}

pub fn list_up_to_month(conn: &Connection, month: &str) -> Result<Vec<BudgetAllocation>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, month, account_id, amount, created_at
         FROM budgets
         WHERE month <= ?1
         ORDER BY month ASC;",
    )?;
    let rows = stmt.query_map(params![month], map_row)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn list_for_month(conn: &Connection, month: &str) -> Result<Vec<BudgetAllocation>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, month, account_id, amount, created_at
         FROM budgets
         WHERE month = ?1;",
    )?;
    let rows = stmt.query_map(params![month], map_row)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

use super::models::{PaymentPlan, PlanFrequency, PlanStatus, PlanType};
use crate::shared::AppError;
use rusqlite::{params, Connection, Row};
use std::str::FromStr;

fn map_row(row: &Row<'_>) -> rusqlite::Result<PaymentPlan> {
    let type_str: String = row.get(2)?;
    let plan_type = PlanType::from_str(&type_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;

    let status_str: String = row.get(3)?;
    let status = PlanStatus::from_str(&status_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;

    let freq_str: String = row.get(6)?;
    let frequency = PlanFrequency::from_str(&freq_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            6,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;

    let day_of_month: Option<i64> = row.get(9)?;
    let auto_post_int: i64 = row.get(15)?;

    Ok(PaymentPlan {
        id: row.get(0)?,
        title: row.get(1)?,
        plan_type,
        status,
        total_amount: row.get(4)?,
        installment_amount: row.get(5)?,
        frequency,
        start_date: row.get(7)?,
        due_date: row.get(8)?,
        day_of_month: day_of_month.map(|d| d as u32),
        from_account_id: row.get(10)?,
        to_account_id: row.get(11)?,
        notes: row.get(12)?,
        created_at: row.get(13)?,
        last_posted_date: row.get(14)?,
        auto_post: auto_post_int != 0,
    })
}

pub fn insert(conn: &Connection, plan: &PaymentPlan) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO plans (
            id, title, type, status, total_amount, installment_amount,
            frequency, start_date, due_date, day_of_month, from_account_id,
            to_account_id, notes, created_at, last_posted_date, auto_post
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16);",
        params![
            plan.id,
            plan.title,
            plan.plan_type.as_str(),
            plan.status.as_str(),
            plan.total_amount,
            plan.installment_amount,
            plan.frequency.as_str(),
            plan.start_date,
            plan.due_date,
            plan.day_of_month,
            plan.from_account_id,
            plan.to_account_id,
            plan.notes,
            plan.created_at,
            plan.last_posted_date,
            if plan.auto_post { 1 } else { 0 },
        ],
    )?;
    Ok(())
}

pub fn update(conn: &Connection, plan: &PaymentPlan) -> Result<(), AppError> {
    let affected = conn.execute(
        "UPDATE plans SET
            title = ?2,
            type = ?3,
            status = ?4,
            total_amount = ?5,
            installment_amount = ?6,
            frequency = ?7,
            start_date = ?8,
            due_date = ?9,
            day_of_month = ?10,
            from_account_id = ?11,
            to_account_id = ?12,
            notes = ?13,
            last_posted_date = ?14,
            auto_post = ?15
         WHERE id = ?1;",
        params![
            plan.id,
            plan.title,
            plan.plan_type.as_str(),
            plan.status.as_str(),
            plan.total_amount,
            plan.installment_amount,
            plan.frequency.as_str(),
            plan.start_date,
            plan.due_date,
            plan.day_of_month,
            plan.from_account_id,
            plan.to_account_id,
            plan.notes,
            plan.last_posted_date,
            if plan.auto_post { 1 } else { 0 },
        ],
    )?;

    if affected == 0 {
        return Err(AppError::NotFound(format!("plan id={}", plan.id)));
    }

    Ok(())
}

pub fn update_last_posted_date(conn: &Connection, id: &str, date: &str) -> Result<(), AppError> {
    let affected = conn.execute(
        "UPDATE plans SET last_posted_date = ?2 WHERE id = ?1;",
        params![id, date],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("plan id={id}")));
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM plans WHERE id = ?1;", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("plan id={id}")));
    }
    Ok(())
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<PaymentPlan>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, type, status, total_amount, installment_amount,
                frequency, start_date, due_date, day_of_month, from_account_id,
                to_account_id, notes, created_at, last_posted_date, auto_post
         FROM plans WHERE id = ?1;",
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_row(row)?))
    } else {
        Ok(None)
    }
}

pub fn list_all(conn: &Connection) -> Result<Vec<PaymentPlan>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, type, status, total_amount, installment_amount,
                frequency, start_date, due_date, day_of_month, from_account_id,
                to_account_id, notes, created_at, last_posted_date, auto_post
         FROM plans ORDER BY created_at DESC;",
    )?;

    let rows = stmt.query_map([], map_row)?;
    let mut plans = Vec::new();
    for row in rows {
        plans.push(row?);
    }
    Ok(plans)
}

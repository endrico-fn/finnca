use super::dto::{CreatePlanInput, PlanProgressView, UpdatePlanInput};
use super::models::{PaymentPlan, PlanStatus};
use super::repository;
use crate::audit::{self, models::AuditAction};
use crate::shared::AppError;
use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};
use ulid::Ulid;

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn create_plan(
    conn: &Connection,
    input: CreatePlanInput,
    actor: &str,
) -> Result<PaymentPlan, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::InvalidInput("plan title cannot be empty".into()));
    }
    if input.total_amount <= 0 {
        return Err(AppError::InvalidInput(
            "total_amount must be greater than zero".into(),
        ));
    }
    if input.installment_amount <= 0 {
        return Err(AppError::InvalidInput(
            "installment_amount must be greater than zero".into(),
        ));
    }

    let plan = PaymentPlan {
        id: Ulid::new().to_string(),
        title: input.title.trim().to_string(),
        plan_type: input.plan_type,
        status: input.status.unwrap_or(PlanStatus::Active),
        total_amount: input.total_amount,
        installment_amount: input.installment_amount,
        frequency: input.frequency,
        start_date: input.start_date,
        due_date: input.due_date,
        day_of_month: input.day_of_month,
        from_account_id: input.from_account_id,
        to_account_id: input.to_account_id,
        notes: input.notes,
        created_at: now_unix(),
    };

    repository::insert(conn, &plan)?;

    let _ = audit::record(
        conn,
        actor,
        AuditAction::CreatePlan,
        "PLAN",
        &plan.id,
        Some(&format!("Created plan '{}'", plan.title)),
    );

    Ok(plan)
}

pub fn update_plan(
    conn: &Connection,
    id: &str,
    input: UpdatePlanInput,
    actor: &str,
) -> Result<PaymentPlan, AppError> {
    let mut plan = repository::get_by_id(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("plan id={id}")))?;

    if let Some(title) = input.title {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err(AppError::InvalidInput("plan title cannot be empty".into()));
        }
        plan.title = trimmed.to_string();
    }
    if let Some(plan_type) = input.plan_type {
        plan.plan_type = plan_type;
    }
    if let Some(status) = input.status {
        plan.status = status;
    }
    if let Some(total_amount) = input.total_amount {
        if total_amount <= 0 {
            return Err(AppError::InvalidInput(
                "total_amount must be greater than zero".into(),
            ));
        }
        plan.total_amount = total_amount;
    }
    if let Some(installment_amount) = input.installment_amount {
        if installment_amount <= 0 {
            return Err(AppError::InvalidInput(
                "installment_amount must be greater than zero".into(),
            ));
        }
        plan.installment_amount = installment_amount;
    }
    if let Some(frequency) = input.frequency {
        plan.frequency = frequency;
    }
    if let Some(start_date) = input.start_date {
        plan.start_date = start_date;
    }
    if let Some(due_date) = input.due_date {
        plan.due_date = due_date;
    }
    if let Some(day_of_month) = input.day_of_month {
        plan.day_of_month = day_of_month;
    }
    if let Some(from_account_id) = input.from_account_id {
        plan.from_account_id = from_account_id;
    }
    if let Some(to_account_id) = input.to_account_id {
        plan.to_account_id = to_account_id;
    }
    if let Some(notes) = input.notes {
        plan.notes = notes;
    }

    repository::update(conn, &plan)?;

    let _ = audit::record(
        conn,
        actor,
        AuditAction::UpdatePlan,
        "PLAN",
        &plan.id,
        Some(&format!("Updated plan '{}'", plan.title)),
    );

    Ok(plan)
}

pub fn delete_plan(conn: &Connection, id: &str, actor: &str) -> Result<(), AppError> {
    let plan = repository::get_by_id(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("plan id={id}")))?;

    repository::delete(conn, id)?;

    let _ = audit::record(
        conn,
        actor,
        AuditAction::DeletePlan,
        "PLAN",
        id,
        Some(&format!("Deleted plan '{}'", plan.title)),
    );

    Ok(())
}

pub fn get_plan(conn: &Connection, id: &str) -> Result<PaymentPlan, AppError> {
    repository::get_by_id(conn, id)?.ok_or_else(|| AppError::NotFound(format!("plan id={id}")))
}

pub fn list_plans(conn: &Connection) -> Result<Vec<PaymentPlan>, AppError> {
    repository::list_all(conn)
}

pub fn calculate_plan_progress(
    conn: &Connection,
    plan: &PaymentPlan,
) -> Result<PlanProgressView, AppError> {
    let title_pattern = format!("%{}%", plan.title.to_lowercase());

    let mut stmt = conn.prepare(
        "SELECT p.amount
         FROM postings p
         JOIN journal_entries je ON je.id = p.entry_id
         WHERE (p.account_id = ?1 OR p.account_id = ?2)
           AND (je.plan_id = ?3 OR LOWER(je.description) LIKE ?4 OR LOWER(COALESCE(je.notes, '')) LIKE ?4)
           AND p.amount > 0;",
    )?;

    let rows = stmt.query_map(
        params![
            plan.to_account_id,
            plan.from_account_id,
            plan.id,
            title_pattern
        ],
        |row| row.get::<_, i64>(0),
    )?;

    let mut paid_amount: i64 = 0;
    let mut count: i64 = 0;

    for amt in rows {
        paid_amount += amt?;
        count += 1;
    }

    let total = if plan.total_amount > 0 {
        plan.total_amount
    } else {
        1
    };
    let progress_percent = ((paid_amount * 100) / total).min(100);
    let remaining_amount = plan.total_amount.saturating_sub(paid_amount);
    let is_settled = paid_amount >= plan.total_amount && plan.total_amount > 0;

    Ok(PlanProgressView {
        plan: plan.clone(),
        paid_amount,
        remaining_amount,
        progress_percent,
        is_settled,
        installments_paid_count: count,
    })
}

pub fn list_plans_with_progress(conn: &Connection) -> Result<Vec<PlanProgressView>, AppError> {
    let plans = repository::list_all(conn)?;
    let mut views = Vec::with_capacity(plans.len());
    for p in plans {
        views.push(calculate_plan_progress(conn, &p)?);
    }
    Ok(views)
}

use super::dto::{
    CreatePlanInput, DueRecurringPlanView, PlanProgressView, PostDueRecurringBatchInput,
    RecordInstallmentInput, UpdatePlanInput,
};
use super::models::{PaymentPlan, PlanFrequency, PlanStatus, PlanType};
use super::repository;
use crate::audit::{self, models::AuditAction};
use crate::shared::AppError;
use chrono::{Datelike, Local, NaiveDate};
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
    if input.plan_type != PlanType::Recurring && input.total_amount <= 0 {
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
        last_posted_date: input.last_posted_date,
        auto_post: input.auto_post.unwrap_or(false),
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
        if plan.plan_type != PlanType::Recurring && total_amount <= 0 {
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
    if let Some(lpd) = input.last_posted_date {
        plan.last_posted_date = lpd;
    }
    if let Some(ap) = input.auto_post {
        plan.auto_post = ap;
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
    // 1. Primary accurate calculation: strictly matched by relational foreign key je.plan_id
    let mut stmt = conn.prepare(
        "SELECT p.amount
         FROM postings p
         JOIN journal_entries je ON je.id = p.entry_id
         WHERE (p.account_id = ?1 OR p.account_id = ?2)
           AND je.plan_id = ?3
           AND p.amount > 0;",
    )?;

    let rows = stmt.query_map(
        params![plan.to_account_id, plan.from_account_id, plan.id],
        |row| row.get::<_, i64>(0),
    )?;

    let mut paid_amount: i64 = 0;
    let mut count: i64 = 0;

    for amt in rows {
        paid_amount = paid_amount.saturating_add(amt?);
        count = count.saturating_add(1);
    }

    // 2. Backward-compatibility fallback: if no entries linked via plan_id, check legacy unlinked entries
    if count == 0 {
        let title_pattern = format!("%{}%", plan.title.to_lowercase());
        let mut stmt_legacy = conn.prepare(
            "SELECT p.amount
             FROM postings p
             JOIN journal_entries je ON je.id = p.entry_id
             WHERE (p.account_id = ?1 OR p.account_id = ?2)
               AND je.plan_id IS NULL
               AND (LOWER(je.description) LIKE ?3 OR LOWER(COALESCE(je.notes, '')) LIKE ?3)
               AND p.amount > 0;",
        )?;

        let legacy_rows = stmt_legacy.query_map(
            params![plan.to_account_id, plan.from_account_id, title_pattern],
            |row| row.get::<_, i64>(0),
        )?;

        for amt in legacy_rows {
            paid_amount = paid_amount.saturating_add(amt?);
            count = count.saturating_add(1);
        }
    }

    let total = if plan.total_amount > 0 {
        plan.total_amount
    } else {
        1
    };
    let progress_percent = (paid_amount.saturating_mul(100) / total).min(100);
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

pub fn record_installment(
    conn: &mut Connection,
    input: RecordInstallmentInput,
    actor: &str,
) -> Result<crate::ledger::dto::JournalEntryView, AppError> {
    let plan = repository::get_by_id(conn, &input.plan_id)?
        .ok_or_else(|| AppError::NotFound(format!("plan id={}", input.plan_id)))?;

    if input.amount <= 0 {
        return Err(AppError::InvalidInput(
            "Installment amount must be greater than 0".into(),
        ));
    }

    // Resolve accounts to inherit account currency dynamically instead of hardcoding
    let to_acc = crate::accounts::repository::get_by_id(conn, &plan.to_account_id)?
        .ok_or_else(|| AppError::NotFound(format!("Target account id={}", plan.to_account_id)))?;
    let from_acc = crate::accounts::repository::get_by_id(conn, &plan.from_account_id)?
        .ok_or_else(|| AppError::NotFound(format!("Source account id={}", plan.from_account_id)))?;

    if to_acc.placeholder {
        return Err(AppError::InvalidInput(format!(
            "Target account '{}' is a placeholder",
            to_acc.name
        )));
    }
    if from_acc.placeholder {
        return Err(AppError::InvalidInput(format!(
            "Source account '{}' is a placeholder",
            from_acc.name
        )));
    }

    let currency = if !to_acc.currency.trim().is_empty() {
        to_acc.currency
    } else if !from_acc.currency.trim().is_empty() {
        from_acc.currency
    } else {
        "IDR".to_string()
    };

    let fx_rate = crate::ledger::currency::DEFAULT_FX_RATE;

    let description = if let Some(ref n) = input.notes {
        if !n.trim().is_empty() {
            format!("{}: {}", plan.title, n.trim())
        } else {
            format!("Installment: {}", plan.title)
        }
    } else {
        format!("Installment: {}", plan.title)
    };

    let entry_input = crate::ledger::dto::CreateJournalEntryInput {
        id: None,
        date: input.date,
        description,
        notes: input.notes,
        reference_no: input.reference_no,
        due_date: None,
        plan_id: Some(plan.id.clone()),
        currency: Some(currency),
        fx_rate: Some(fx_rate),
        postings: vec![
            crate::ledger::dto::PostingInput {
                id: None,
                account_id: plan.to_account_id.clone(),
                amount: input.amount,
                memo: Some(format!("Plan payment: {}", plan.title)),
                action: None,
                reconcile: None,
                ..Default::default()
            },
            crate::ledger::dto::PostingInput {
                id: None,
                account_id: plan.from_account_id.clone(),
                amount: -input.amount,
                memo: Some(format!("Plan payment: {}", plan.title)),
                action: None,
                reconcile: None,
                ..Default::default()
            },
        ],
    };

    let entry = crate::ledger::service::post_journal_entry(conn, entry_input, actor)?;

    let _ = repository::update_last_posted_date(conn, &plan.id, &entry.date);

    let prog = calculate_plan_progress(conn, &plan)?;
    if prog.is_settled && plan.status != PlanStatus::Completed {
        let mut updated = plan.clone();
        updated.status = PlanStatus::Completed;
        repository::update(conn, &updated)?;
    }

    Ok(entry)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .and_then(|d| d.pred_opt())
    .map(|d| d.day())
    .unwrap_or(30)
}

pub fn calculate_next_due_date(plan: &PaymentPlan, _as_of: NaiveDate) -> Option<NaiveDate> {
    let start_date = NaiveDate::parse_from_str(&plan.start_date, "%Y-%m-%d").ok()?;
    let last_posted = plan
        .last_posted_date
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    match plan.frequency {
        PlanFrequency::Daily => {
            if let Some(lp) = last_posted {
                Some(lp + chrono::Duration::days(1))
            } else {
                Some(start_date)
            }
        }
        PlanFrequency::Weekly => {
            if let Some(lp) = last_posted {
                Some(lp + chrono::Duration::days(7))
            } else {
                Some(start_date)
            }
        }
        PlanFrequency::Monthly => {
            let target_day = plan
                .day_of_month
                .unwrap_or_else(|| start_date.day())
                .clamp(1, 31);
            if let Some(lp) = last_posted {
                let (next_y, next_m) = if lp.month() == 12 {
                    (lp.year() + 1, 1)
                } else {
                    (lp.year(), lp.month() + 1)
                };
                let max_d = days_in_month(next_y, next_m);
                let actual_d = target_day.min(max_d);
                NaiveDate::from_ymd_opt(next_y, next_m, actual_d)
            } else {
                let max_d = days_in_month(start_date.year(), start_date.month());
                let actual_d = target_day.min(max_d);
                let this_month_date =
                    NaiveDate::from_ymd_opt(start_date.year(), start_date.month(), actual_d)?;
                if this_month_date >= start_date {
                    Some(this_month_date)
                } else {
                    let (next_y, next_m) = if start_date.month() == 12 {
                        (start_date.year() + 1, 1)
                    } else {
                        (start_date.year(), start_date.month() + 1)
                    };
                    let max_d = days_in_month(next_y, next_m);
                    let actual_d = target_day.min(max_d);
                    NaiveDate::from_ymd_opt(next_y, next_m, actual_d)
                }
            }
        }
    }
}

pub fn get_due_recurring_plans(
    conn: &Connection,
    as_of_date: Option<&str>,
) -> Result<Vec<DueRecurringPlanView>, AppError> {
    let as_of = if let Some(d) = as_of_date {
        NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| AppError::InvalidInput(format!("Invalid as_of_date: {d}")))?
    } else {
        Local::now().date_naive()
    };

    let all_plans = repository::list_all(conn)?;
    let mut due_views = Vec::new();

    for plan in all_plans {
        if plan.status != PlanStatus::Active {
            continue;
        }

        if plan.plan_type != PlanType::Recurring {
            let prog = calculate_plan_progress(conn, &plan)?;
            if prog.is_settled {
                continue;
            }
        }

        if let Some(next_due) = calculate_next_due_date(&plan, as_of) {
            let days_diff = (next_due - as_of).num_days();

            // Include if overdue, due today, or upcoming within 7 days
            if days_diff <= 7 {
                let from_acc_name =
                    crate::accounts::repository::get_by_id(conn, &plan.from_account_id)?
                        .map(|a| a.name)
                        .unwrap_or_else(|| plan.from_account_id.clone());
                let to_acc_name =
                    crate::accounts::repository::get_by_id(conn, &plan.to_account_id)?
                        .map(|a| a.name)
                        .unwrap_or_else(|| plan.to_account_id.clone());

                let currency = crate::accounts::repository::get_by_id(conn, &plan.to_account_id)?
                    .map(|a| a.currency)
                    .filter(|c| !c.trim().is_empty())
                    .unwrap_or_else(|| "IDR".to_string());

                due_views.push(DueRecurringPlanView {
                    plan_id: plan.id.clone(),
                    title: plan.title.clone(),
                    plan_type: plan.plan_type,
                    amount: plan.installment_amount,
                    currency,
                    from_account_id: plan.from_account_id.clone(),
                    from_account_name: from_acc_name,
                    to_account_id: plan.to_account_id.clone(),
                    to_account_name: to_acc_name,
                    next_due_date: next_due.format("%Y-%m-%d").to_string(),
                    is_overdue: days_diff < 0,
                    days_diff,
                    last_posted_date: plan.last_posted_date.clone(),
                });
            }
        }
    }

    due_views.sort_by_key(|v| v.days_diff);

    Ok(due_views)
}

pub fn post_due_recurring_batch(
    conn: &mut Connection,
    input: PostDueRecurringBatchInput,
    actor: &str,
) -> Result<Vec<crate::ledger::dto::JournalEntryView>, AppError> {
    let post_date = input
        .date
        .unwrap_or_else(|| Local::now().date_naive().format("%Y-%m-%d").to_string());

    let mut posted_entries = Vec::with_capacity(input.plan_ids.len());

    for plan_id in input.plan_ids {
        let plan = repository::get_by_id(conn, &plan_id)?
            .ok_or_else(|| AppError::NotFound(format!("plan id={plan_id}")))?;

        let inst_input = RecordInstallmentInput {
            plan_id: plan.id,
            date: post_date.clone(),
            amount: plan.installment_amount,
            reference_no: None,
            notes: Some("Auto-posted via Recurring Runner".to_string()),
        };

        let entry = record_installment(conn, inst_input, actor)?;
        posted_entries.push(entry);
    }

    Ok(posted_entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;
    use crate::plan::models::{PlanFrequency, PlanStatus, PlanType};

    #[test]
    fn test_calculate_next_due_date_monthly() {
        let plan = PaymentPlan {
            id: "test1".into(),
            title: "Netflix".into(),
            plan_type: PlanType::Recurring,
            status: PlanStatus::Active,
            total_amount: 0,
            installment_amount: 186000,
            frequency: PlanFrequency::Monthly,
            start_date: "2026-01-15".into(),
            due_date: None,
            day_of_month: Some(15),
            from_account_id: "acc_from".into(),
            to_account_id: "acc_to".into(),
            notes: None,
            last_posted_date: None,
            auto_post: false,
            created_at: 1000,
        };

        // When never posted and as_of is before day_of_month in start_date's month
        let as_of = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
        let next = calculate_next_due_date(&plan, as_of).unwrap();
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());

        // When posted on 2026-01-15
        let mut plan_posted = plan.clone();
        plan_posted.last_posted_date = Some("2026-01-15".into());
        let next_after_post = calculate_next_due_date(&plan_posted, as_of).unwrap();
        assert_eq!(
            next_after_post,
            NaiveDate::from_ymd_opt(2026, 2, 15).unwrap()
        );

        // Clamping to month end (e.g. 31st in February)
        let mut plan_month_end = plan.clone();
        plan_month_end.day_of_month = Some(31);
        plan_month_end.last_posted_date = Some("2026-01-31".into());
        let next_feb = calculate_next_due_date(&plan_month_end, as_of).unwrap();
        assert_eq!(next_feb, NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }

    #[test]
    fn test_calculate_next_due_date_weekly_and_daily() {
        let mut plan = PaymentPlan {
            id: "test_daily".into(),
            title: "Daily Coffee".into(),
            plan_type: PlanType::Recurring,
            status: PlanStatus::Active,
            total_amount: 0,
            installment_amount: 25000,
            frequency: PlanFrequency::Daily,
            start_date: "2026-09-01".into(),
            due_date: None,
            day_of_month: None,
            from_account_id: "acc_from".into(),
            to_account_id: "acc_to".into(),
            notes: None,
            last_posted_date: Some("2026-09-10".into()),
            auto_post: false,
            created_at: 1000,
        };

        let as_of = NaiveDate::from_ymd_opt(2026, 9, 12).unwrap();
        let next = calculate_next_due_date(&plan, as_of).unwrap();
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 9, 11).unwrap());

        plan.frequency = PlanFrequency::Weekly;
        let next_weekly = calculate_next_due_date(&plan, as_of).unwrap();
        assert_eq!(next_weekly, NaiveDate::from_ymd_opt(2026, 9, 17).unwrap());
    }

    #[test]
    fn test_get_due_recurring_plans_in_memory_db() {
        let mut conn = Connection::open_in_memory().expect("open memory db");
        run_migrations(&mut conn).expect("run migrations");

        // Seed basic accounts
        conn.execute(
            "INSERT INTO accounts (id, code, name, type, currency, placeholder, hidden, created_at)
             VALUES ('acc_bank', '1101', 'Bank BCA', 'ASSET', 'IDR', 0, 0, 1000);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO accounts (id, code, name, type, currency, placeholder, hidden, created_at)
             VALUES ('acc_exp', '5101', 'Internet Bill', 'EXPENSE', 'IDR', 0, 0, 1000);",
            [],
        )
        .unwrap();

        // Create an active recurring plan due on the 20th
        let input = CreatePlanInput {
            title: "Indihome Wifi".into(),
            plan_type: PlanType::Recurring,
            status: Some(PlanStatus::Active),
            total_amount: 0,
            installment_amount: 350000,
            frequency: PlanFrequency::Monthly,
            start_date: "2026-09-01".into(),
            due_date: None,
            day_of_month: Some(20),
            from_account_id: "acc_bank".into(),
            to_account_id: "acc_exp".into(),
            notes: Some("Monthly internet".into()),
            last_posted_date: None,
            auto_post: Some(false),
        };
        create_plan(&conn, input, "TEST").expect("create plan");

        // As of 2026-09-23: plan due on 2026-09-20 is OVERDUE (days_diff = -3)
        let due_list = get_due_recurring_plans(&conn, Some("2026-09-23")).expect("get due");
        assert_eq!(due_list.len(), 1);
        assert_eq!(due_list[0].title, "Indihome Wifi");
        assert!(due_list[0].is_overdue);
        assert_eq!(due_list[0].days_diff, -3);

        // Batch post this plan
        let batch_input = PostDueRecurringBatchInput {
            plan_ids: vec![due_list[0].plan_id.clone()],
            date: Some("2026-09-23".into()),
        };
        let posted = post_due_recurring_batch(&mut conn, batch_input, "TEST").expect("post batch");
        assert_eq!(posted.len(), 1);
        assert_eq!(posted[0].postings.len(), 2);

        // Now, as of 2026-09-23, next due date moved to 2026-10-20 (> 7 days ahead), so not due now!
        let due_after = get_due_recurring_plans(&conn, Some("2026-09-23")).expect("get due after");
        assert_eq!(due_after.len(), 0);
    }
}

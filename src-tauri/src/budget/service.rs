use super::dto::{BudgetMonthSummary, EnvelopeView, UpsertBudgetInput};
use super::models::BudgetAllocation;
use super::repository;
use crate::shared::AppError;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use ulid::Ulid;

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn upsert_budget(
    conn: &Connection,
    input: UpsertBudgetInput,
) -> Result<BudgetAllocation, AppError> {
    if input.month.len() != 7 || input.month.chars().nth(4) != Some('-') {
        return Err(AppError::InvalidInput(
            "month must be YYYY-MM format".into(),
        ));
    }

    let id = input.id.unwrap_or_else(|| Ulid::new().to_string());
    let budget = BudgetAllocation {
        id: id.clone(),
        month: input.month,
        account_id: input.account_id,
        amount: input.amount,
        created_at: now_unix(),
    };

    repository::upsert(conn, &budget)?;
    Ok(budget)
}

pub fn delete_budget(conn: &Connection, id: &str) -> Result<(), AppError> {
    repository::delete(conn, id)
}

pub fn get_budget_month_summary(
    conn: &Connection,
    month: &str,
) -> Result<BudgetMonthSummary, AppError> {
    let all_budgets = repository::list_up_to_month(conn, month)?;
    let this_month_budgets = repository::list_for_month(conn, month)?;

    let mut assigned_all_time: HashMap<String, i64> = HashMap::new();
    let mut assigned_this_month: HashMap<String, i64> = HashMap::new();

    for b in &all_budgets {
        *assigned_all_time.entry(b.account_id.clone()).or_default() += b.amount;
    }
    for b in &this_month_budgets {
        *assigned_this_month.entry(b.account_id.clone()).or_default() += b.amount;
    }

    let mut activity_all_time: HashMap<String, i64> = HashMap::new();
    let mut activity_this_month: HashMap<String, i64> = HashMap::new();
    let mut total_income_all_time: i64 = 0;

    {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.type, p.amount, je.date
             FROM postings p
             JOIN accounts a ON a.id = p.account_id
             JOIN journal_entries je ON je.id = p.entry_id
             WHERE a.type IN ('INCOME', 'EXPENSE')
               AND a.placeholder = 0
               AND je.date <= ?1;",
        )?;

        let cutoff = format!("{month}-31");
        let rows = stmt.query_map(params![cutoff], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        for row in rows {
            let (account_id, account_type, amount, date) = row?;
            let tx_month = &date[..7];
            let is_this_month = tx_month == month;

            match account_type.as_str() {
                "INCOME" => {
                    total_income_all_time += -amount;
                }
                "EXPENSE" => {
                    *activity_all_time.entry(account_id.clone()).or_default() += amount;
                    if is_this_month {
                        *activity_this_month.entry(account_id.clone()).or_default() += amount;
                    }
                }
                _ => {}
            }
        }
    }

    let expense_accounts: Vec<(String, String, String)> = {
        let mut stmt = conn.prepare(
            "SELECT id, code, name FROM accounts WHERE type = 'EXPENSE' AND placeholder = 0 ORDER BY code ASC;",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let mut list = Vec::new();
        for row in rows {
            list.push(row?);
        }
        list
    };

    let mut envelopes = Vec::new();
    let mut total_assigned: i64 = 0;
    let mut total_activity: i64 = 0;
    let mut total_assigned_all_time: i64 = 0;

    for (account_id, account_code, account_name) in expense_accounts {
        let a_all = assigned_all_time.get(&account_id).copied().unwrap_or(0);
        let a_this = assigned_this_month.get(&account_id).copied().unwrap_or(0);
        let act_all = activity_all_time.get(&account_id).copied().unwrap_or(0);
        let act_this = activity_this_month.get(&account_id).copied().unwrap_or(0);

        total_assigned_all_time += a_all;
        total_assigned += a_this;
        total_activity += act_this;

        envelopes.push(EnvelopeView {
            account_id,
            account_code,
            account_name,
            assigned: a_this,
            activity: act_this,
            available: a_all - act_all,
        });
    }

    Ok(BudgetMonthSummary {
        month: month.to_string(),
        envelopes,
        total_assigned,
        total_activity,
        to_be_budgeted: total_income_all_time - total_assigned_all_time,
    })
}

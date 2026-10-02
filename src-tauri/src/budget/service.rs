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
    let parts: Vec<&str> = month.split('-').collect();
    if parts.len() != 2
        || parts[0].len() != 4
        || parts[1].len() != 2
        || !parts[0].chars().all(|c| c.is_ascii_digit())
        || !parts[1].chars().all(|c| c.is_ascii_digit())
    {
        return Err(AppError::InvalidInput(
            "month must be YYYY-MM format".into(),
        ));
    }
    let m_num: u32 = parts[1].parse().unwrap_or(0);
    if !(1..=12).contains(&m_num) {
        return Err(AppError::InvalidInput(
            "month must be between 01 and 12".into(),
        ));
    }

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

    let total_income_all_time: i64 = conn.query_row(
        "SELECT COALESCE(SUM(-p.amount), 0)
         FROM postings p
         JOIN accounts a ON a.id = p.account_id
         JOIN journal_entries je ON je.id = p.entry_id
         WHERE a.type = 'INCOME'
           AND a.placeholder = 0
           AND strftime('%Y-%m', je.date) <= ?1;",
        params![month],
        |row| row.get(0),
    )?;

    let mut activity_all_time: HashMap<String, i64> = HashMap::new();
    let mut activity_this_month: HashMap<String, i64> = HashMap::new();

    {
        let mut stmt = conn.prepare(
            "SELECT 
                p.account_id,
                COALESCE(SUM(p.amount), 0),
                COALESCE(SUM(CASE WHEN strftime('%Y-%m', je.date) = ?1 THEN p.amount ELSE 0 END), 0)
             FROM postings p
             JOIN accounts a ON a.id = p.account_id
             JOIN journal_entries je ON je.id = p.entry_id
             WHERE a.type = 'EXPENSE'
               AND a.placeholder = 0
               AND strftime('%Y-%m', je.date) <= ?1
             GROUP BY p.account_id;",
        )?;

        let rows = stmt.query_map(params![month], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        for row in rows {
            let (account_id, act_all, act_this) = row?;
            activity_all_time.insert(account_id.clone(), act_all);
            activity_this_month.insert(account_id, act_this);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().expect("open memory db");
        run_migrations(&mut conn).expect("run migrations");

        conn.execute(
            "INSERT INTO accounts (id, code, name, type, currency, placeholder, hidden, created_at)
             VALUES ('acc_cash', '1001', 'Cash', 'ASSET', 'IDR', 0, 0, 1000);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO accounts (id, code, name, type, currency, placeholder, hidden, created_at)
             VALUES ('acc_inc', '4001', 'Salary', 'INCOME', 'IDR', 0, 0, 1000);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO accounts (id, code, name, type, currency, placeholder, hidden, created_at)
             VALUES ('acc_food', '5001', 'Food', 'EXPENSE', 'IDR', 0, 0, 1000);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO accounts (id, code, name, type, currency, placeholder, hidden, created_at)
             VALUES ('acc_exp_parent', '5000', 'Expense Parent', 'EXPENSE', 'IDR', 1, 0, 1000);",
            [],
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_budget_summary_aggregation_and_corrupt_date_safety() {
        let conn = setup_test_db();

        upsert_budget(
            &conn,
            UpsertBudgetInput {
                id: None,
                month: "2026-02".into(),
                account_id: "acc_food".into(),
                amount: 500_000,
            },
        )
        .unwrap();

        conn.execute(
            "INSERT INTO journal_entries (id, date, description, posted_at)
             VALUES ('je_1', '2026-01-15', 'Jan Salary', 1000);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO postings (id, entry_id, account_id, amount)
             VALUES ('p_1a', 'je_1', 'acc_cash', 1000000),
                    ('p_1b', 'je_1', 'acc_inc', -1000000);",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO journal_entries (id, date, description, posted_at)
             VALUES ('je_2', '2026-02-28', 'Feb Grocery', 1000);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO postings (id, entry_id, account_id, amount)
             VALUES ('p_2a', 'je_2', 'acc_food', 200000),
                    ('p_2b', 'je_2', 'acc_cash', -200000);",
            [],
        )
        .unwrap();

        // Corrupted / short date string in journal entry should not panic
        conn.execute(
            "INSERT INTO journal_entries (id, date, description, posted_at)
             VALUES ('je_corrupt', 'bad', 'Corrupted Date', 1000);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO postings (id, entry_id, account_id, amount)
             VALUES ('p_ca', 'je_corrupt', 'acc_food', 50000),
                    ('p_cb', 'je_corrupt', 'acc_cash', -50000);",
            [],
        )
        .unwrap();

        // Future expense in March 2026 should not be counted in Feb summary
        conn.execute(
            "INSERT INTO journal_entries (id, date, description, posted_at)
             VALUES ('je_3', '2026-03-05', 'Mar Expense', 1000);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO postings (id, entry_id, account_id, amount)
             VALUES ('p_3a', 'je_3', 'acc_food', 150000),
                    ('p_3b', 'je_3', 'acc_cash', -150000);",
            [],
        )
        .unwrap();

        let summary =
            get_budget_month_summary(&conn, "2026-02").expect("summary succeeded without panic");
        assert_eq!(summary.month, "2026-02");
        assert_eq!(summary.total_assigned, 500_000);
        assert_eq!(summary.total_activity, 200_000);
        assert_eq!(summary.to_be_budgeted, 500_000);

        let food_env = summary
            .envelopes
            .iter()
            .find(|e| e.account_id == "acc_food")
            .unwrap();
        assert_eq!(food_env.assigned, 500_000);
        assert_eq!(food_env.activity, 200_000);
        assert_eq!(food_env.available, 300_000);

        // Invalid month inputs
        assert!(matches!(
            get_budget_month_summary(&conn, "bad"),
            Err(AppError::InvalidInput(_))
        ));
        assert!(matches!(
            get_budget_month_summary(&conn, "2026-13"),
            Err(AppError::InvalidInput(_))
        ));
        assert!(matches!(
            get_budget_month_summary(&conn, "2026-00"),
            Err(AppError::InvalidInput(_))
        ));
        assert!(matches!(
            get_budget_month_summary(&conn, "abcd-ef"),
            Err(AppError::InvalidInput(_))
        ));
    }
}

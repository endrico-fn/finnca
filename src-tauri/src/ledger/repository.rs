use super::dto::{
    AccountRunningLedgerItem, DashboardMetricsView, JournalEntryView, LedgerTotalsView, PostingView,
};
use super::models::{JournalEntry, Posting, ReconcileState};
use crate::accounts::models::AccountType;
use crate::shared::AppError;
use rusqlite::{params, Connection, Transaction};
use std::str::FromStr;

pub fn insert_entry(tx: &Transaction<'_>, entry: &JournalEntry) -> Result<(), AppError> {
    tx.execute(
        "INSERT INTO journal_entries (id, date, description, notes, reference_no, due_date, plan_id, currency, fx_rate, posted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);",
        params![
            entry.id,
            entry.date,
            entry.description,
            entry.notes,
            entry.reference_no,
            entry.due_date,
            entry.plan_id,
            entry.currency,
            entry.fx_rate,
            entry.posted_at,
        ],
    )?;
    Ok(())
}

pub fn insert_posting(tx: &Transaction<'_>, posting: &Posting) -> Result<(), AppError> {
    tx.execute(
        "INSERT INTO postings (id, entry_id, account_id, amount, memo, action, reconciled, reconciled_at, currency, fx_rate, cost_amount)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11);",
        params![
            posting.id,
            posting.entry_id,
            posting.account_id,
            posting.amount,
            posting.memo,
            posting.action,
            posting.reconcile.as_str(),
            posting.reconciled_at,
            posting.currency,
            posting.fx_rate,
            posting.cost_amount,
        ],
    )?;
    Ok(())
}

pub fn delete_entry(tx: &Transaction<'_>, id: &str) -> Result<(), AppError> {
    tx.execute("DELETE FROM journal_entries WHERE id = ?1;", params![id])?;
    Ok(())
}

pub fn delete_postings_by_entry(tx: &Transaction<'_>, entry_id: &str) -> Result<(), AppError> {
    tx.execute(
        "DELETE FROM postings WHERE entry_id = ?1;",
        params![entry_id],
    )?;
    Ok(())
}

pub fn update_entry(tx: &Transaction<'_>, entry: &JournalEntry) -> Result<(), AppError> {
    tx.execute(
        "UPDATE journal_entries SET
            date = ?2,
            description = ?3,
            notes = ?4,
            reference_no = ?5,
            due_date = ?6,
            plan_id = ?7,
            currency = ?8,
            fx_rate = ?9
         WHERE id = ?1;",
        params![
            entry.id,
            entry.date,
            entry.description,
            entry.notes,
            entry.reference_no,
            entry.due_date,
            entry.plan_id,
            entry.currency,
            entry.fx_rate,
        ],
    )?;
    Ok(())
}

pub fn get_entry_by_id(conn: &Connection, id: &str) -> Result<Option<JournalEntryView>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, date, description, notes, reference_no, due_date, plan_id, currency, fx_rate, posted_at
         FROM journal_entries WHERE id = ?1;",
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        let entry_id: String = row.get(0)?;
        let date: String = row.get(1)?;
        let description: String = row.get(2)?;
        let notes: Option<String> = row.get(3)?;
        let reference_no: Option<String> = row.get(4)?;
        let due_date: Option<String> = row.get(5)?;
        let plan_id: Option<String> = row.get(6)?;
        let currency: String = row.get(7)?;
        let fx_rate: i64 = row.get(8)?;
        let posted_at: i64 = row.get(9)?;

        let postings = get_postings_for_entry(conn, &entry_id)?;

        Ok(Some(JournalEntryView {
            id: entry_id,
            date,
            description,
            notes,
            reference_no,
            due_date,
            plan_id,
            currency,
            fx_rate,
            posted_at,
            postings,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_postings_for_entry(
    conn: &Connection,
    entry_id: &str,
) -> Result<Vec<PostingView>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.entry_id, p.account_id, a.code, a.name, a.type, p.amount, p.memo, p.action, p.reconciled, p.reconciled_at, p.currency, p.fx_rate, p.cost_amount
         FROM postings p
         JOIN accounts a ON p.account_id = a.id
         WHERE p.entry_id = ?1
         ORDER BY p.amount DESC, p.id ASC;",
    )?;

    let rows = stmt.query_map(params![entry_id], |row| {
        let type_str: String = row.get(5)?;
        let account_type = AccountType::from_str(&type_str).unwrap_or(AccountType::Asset);
        let rec_str: String = row.get(9)?;
        let reconcile = ReconcileState::from_str(&rec_str).unwrap_or(ReconcileState::None);

        Ok(PostingView {
            id: row.get(0)?,
            entry_id: row.get(1)?,
            account_id: row.get(2)?,
            account_code: row.get(3)?,
            account_name: row.get(4)?,
            account_type,
            amount: row.get(6)?,
            memo: row.get(7)?,
            action: row.get(8)?,
            reconcile,
            reconciled_at: row.get(10)?,
            currency: row.get(11)?,
            fx_rate: row.get(12)?,
            cost_amount: row.get(13)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

pub fn list_entries(
    conn: &Connection,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<JournalEntryView>, AppError> {
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let mut stmt = conn.prepare(
        "SELECT id, date, description, notes, reference_no, due_date, plan_id, currency, fx_rate, posted_at
         FROM journal_entries
         ORDER BY date DESC, posted_at DESC, id DESC
         LIMIT ?1 OFFSET ?2;",
    )?;

    let entry_rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, i64>(8)?,
            row.get::<_, i64>(9)?,
        ))
    })?;

    let mut entries = Vec::new();
    for row in entry_rows {
        let (
            id,
            date,
            description,
            notes,
            reference_no,
            due_date,
            plan_id,
            currency,
            fx_rate,
            posted_at,
        ) = row?;
        let postings = get_postings_for_entry(conn, &id)?;
        entries.push(JournalEntryView {
            id,
            date,
            description,
            notes,
            reference_no,
            due_date,
            plan_id,
            currency,
            fx_rate,
            posted_at,
            postings,
        });
    }

    Ok(entries)
}

pub fn get_account_running_ledger(
    conn: &Connection,
    account_id: &str,
) -> Result<Vec<AccountRunningLedgerItem>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.entry_id, j.date, j.description, p.amount, p.reconciled, p.memo,
         (
             SELECT CASE 
                 WHEN COUNT(DISTINCT other.account_id) = 1 THEN MAX(a.code || ' ' || a.name)
                 WHEN COUNT(DISTINCT other.account_id) > 1 THEN '-- Split --'
                 ELSE ''
             END
             FROM postings other
             JOIN accounts a ON other.account_id = a.id
             WHERE other.entry_id = p.entry_id AND other.account_id != p.account_id
         ) AS offset_account
         FROM postings p
         JOIN journal_entries j ON p.entry_id = j.id
         WHERE p.account_id = ?1
         ORDER BY j.date ASC, j.posted_at ASC, p.id ASC;",
    )?;

    let rows = stmt.query_map(params![account_id], |row| {
        let rec_str: String = row.get(5)?;
        let reconcile = ReconcileState::from_str(&rec_str).unwrap_or(ReconcileState::None);
        let memo: Option<String> = row.get(6)?;
        let offset_account: Option<String> = row.get(7)?;

        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            reconcile,
            memo,
            offset_account.filter(|s| !s.is_empty()),
        ))
    })?;

    let mut items = Vec::new();
    let mut running: i64 = 0;
    for r in rows {
        let (posting_id, entry_id, date, description, amount, reconcile, memo, offset_account) = r?;
        running = running.saturating_add(amount);
        items.push(AccountRunningLedgerItem {
            posting_id,
            entry_id,
            date,
            description,
            amount,
            running_balance: running,
            reconcile,
            memo,
            offset_account,
        });
    }

    Ok(items)
}

use crate::ledger::currency::{convert_minor_units, normalize_fx_rate};

pub fn calculate_totals(conn: &Connection, fx_rate: i64) -> Result<LedgerTotalsView, AppError> {
    // 1. P&L: Income and Expense recognized at the transaction-time exchange rate
    let mut pl_stmt = conn.prepare(
        "SELECT a.type, COALESCE(p.currency, a.currency), p.amount, COALESCE(p.fx_rate, j.fx_rate), p.cost_amount
         FROM postings p
         JOIN accounts a ON p.account_id = a.id
         JOIN journal_entries j ON p.entry_id = j.id
         WHERE a.type IN ('INCOME', 'EXPENSE')
           AND a.placeholder = 0;",
    )?;

    let mut total_income: i64 = 0;
    let mut total_expenses: i64 = 0;

    let pl_rows = pl_stmt.query_map([], |row| {
        let type_str: String = row.get(0)?;
        let acc_type = AccountType::from_str(&type_str).unwrap_or(AccountType::Income);
        let curr: String = row.get(1)?;
        let amount: i64 = row.get(2)?;
        let entry_fx: i64 = row.get(3)?;
        let cost_amount: Option<i64> = row.get(4)?;
        Ok((acc_type, curr, amount, entry_fx, cost_amount))
    })?;

    for r in pl_rows {
        let (acc_type, curr, amount, entry_fx, cost_amount) = r?;
        let in_idr = if let Some(cost) = cost_amount {
            amount.signum() * cost.abs()
        } else if curr != "IDR" {
            convert_minor_units(amount, &curr, "IDR", normalize_fx_rate(entry_fx, fx_rate))
        } else {
            amount
        };
        if acc_type == AccountType::Income {
            total_income = total_income.saturating_add(-in_idr);
        } else {
            total_expenses = total_expenses.saturating_add(in_idr);
        }
    }

    // 2. Balance Sheet: Assets, Liabilities, Equity (leaf accounts at spot fx rate)
    let mut bs_stmt = conn.prepare(
        "SELECT a.type, COALESCE(p.currency, a.currency), p.amount, p.cost_amount, p.fx_rate
         FROM accounts a
         JOIN postings p ON a.id = p.account_id
         WHERE a.placeholder = 0
           AND a.type IN ('ASSET', 'LIABILITY', 'EQUITY')
           AND a.id NOT IN (SELECT DISTINCT parent_id FROM accounts WHERE parent_id IS NOT NULL);",
    )?;

    let mut total_assets: i64 = 0;
    let mut total_liabilities: i64 = 0;
    let mut total_equity: i64 = 0;

    let bs_rows = bs_stmt.query_map([], |row| {
        let type_str: String = row.get(0)?;
        let acc_type = AccountType::from_str(&type_str).unwrap_or(AccountType::Asset);
        let curr: String = row.get(1)?;
        let amount: i64 = row.get(2)?;
        let cost_amount: Option<i64> = row.get(3)?;
        let posting_fx: Option<i64> = row.get(4)?;
        Ok((acc_type, curr, amount, cost_amount, posting_fx))
    })?;

    for r in bs_rows {
        let (acc_type, curr, amount, cost_amount, _posting_fx) = r?;
        let in_idr = if let Some(cost) = cost_amount {
            amount.signum() * cost.abs()
        } else if curr != "IDR" {
            convert_minor_units(amount, &curr, "IDR", fx_rate)
        } else {
            amount
        };
        match acc_type {
            AccountType::Asset => total_assets = total_assets.saturating_add(in_idr),
            AccountType::Liability => total_liabilities = total_liabilities.saturating_add(-in_idr),
            AccountType::Equity => total_equity = total_equity.saturating_add(-in_idr),
            _ => {}
        }
    }

    let net_income = total_income.saturating_sub(total_expenses);
    let balance_sheet_discrepancy = total_assets
        .saturating_sub(total_liabilities)
        .saturating_sub(total_equity)
        .saturating_sub(net_income);

    Ok(LedgerTotalsView {
        total_assets,
        total_liabilities,
        total_equity,
        total_income,
        total_expenses,
        net_income,
        balance_sheet_discrepancy,
        is_balance_sheet_aligned: balance_sheet_discrepancy == 0,
    })
}

pub fn calculate_dashboard_metrics(
    conn: &Connection,
    fx_rate: i64,
    today: &str,
) -> Result<DashboardMetricsView, AppError> {
    let totals = calculate_totals(conn, fx_rate)?;
    let net_worth = totals.total_assets.saturating_sub(totals.total_liabilities);

    let d30_str = if let Ok(today_date) = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d") {
        (today_date - chrono::Duration::days(30))
            .format("%Y-%m-%d")
            .to_string()
    } else {
        format!("{}-01", &today[..7.min(today.len())])
    };

    let monthly_burn_30d: i64 = {
        let mut stmt = conn.prepare(
            "SELECT COALESCE(p.currency, a.currency), p.amount, COALESCE(p.fx_rate, je.fx_rate), p.cost_amount
             FROM postings p
             JOIN accounts a ON a.id = p.account_id
             JOIN journal_entries je ON je.id = p.entry_id
             WHERE a.type = 'EXPENSE'
               AND a.placeholder = 0
               AND je.date >= ?1;",
        )?;
        let rows = stmt.query_map(params![d30_str], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, Option<i64>>(3)?,
            ))
        })?;
        let mut burn = 0i64;
        for r in rows {
            let (curr, amount, entry_fx, cost_amount) = r?;
            let in_idr = if let Some(cost) = cost_amount {
                amount.signum() * cost.abs()
            } else if curr != "IDR" {
                convert_minor_units(amount, &curr, "IDR", normalize_fx_rate(entry_fx, fx_rate))
            } else {
                amount
            };
            burn = burn.saturating_add(in_idr);
        }
        burn
    };

    let runway_months = if monthly_burn_30d > 0 {
        let tenths = totals.total_assets.saturating_mul(10) / monthly_burn_30d;
        tenths as f64 / 10.0
    } else {
        999.0
    };

    let month_start = if today.len() >= 7 {
        format!("{}-01", &today[..7])
    } else {
        today.to_string()
    };

    let (this_month_income, this_month_expense): (i64, i64) = {
        let mut stmt = conn.prepare(
            "SELECT a.type, COALESCE(p.currency, a.currency), p.amount, COALESCE(p.fx_rate, je.fx_rate), p.cost_amount
             FROM postings p
             JOIN accounts a ON a.id = p.account_id
             JOIN journal_entries je ON je.id = p.entry_id
             WHERE a.type IN ('INCOME', 'EXPENSE')
               AND a.placeholder = 0
               AND je.date >= ?1;",
        )?;
        let rows = stmt.query_map(params![month_start], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?,
            ))
        })?;

        let mut inc = 0i64;
        let mut exp = 0i64;
        for r in rows {
            let (t, curr, amount, entry_fx, cost_amount) = r?;
            let in_idr = if let Some(cost) = cost_amount {
                amount.signum() * cost.abs()
            } else if curr != "IDR" {
                convert_minor_units(amount, &curr, "IDR", normalize_fx_rate(entry_fx, fx_rate))
            } else {
                amount
            };
            if t == "INCOME" {
                inc = inc.saturating_add(-in_idr);
            } else if t == "EXPENSE" {
                exp = exp.saturating_add(in_idr);
            }
        }
        (inc, exp)
    };

    let this_month_net = this_month_income.saturating_sub(this_month_expense);

    let (overdue_invoices_count, overdue_invoices_amount): (i64, i64) = {
        let mut stmt = conn.prepare(
            "SELECT je.id, COALESCE(p.currency, a.currency), p.amount, COALESCE(p.fx_rate, je.fx_rate), p.cost_amount
             FROM journal_entries je
             JOIN postings p ON p.entry_id = je.id
             JOIN accounts a ON a.id = p.account_id
             WHERE je.due_date IS NOT NULL
               AND je.due_date < ?1
               AND (je.notes IS NULL OR je.notes NOT LIKE '%[Settled]%')
               AND je.due_date NOT LIKE '%[Settled]%';",
        )?;
        let rows = stmt.query_map(params![today], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?,
            ))
        })?;
        let mut distinct_ids = std::collections::HashSet::new();
        let mut total_amt = 0i64;
        for r in rows {
            let (id, curr, amount, entry_fx, cost_amount) = r?;
            distinct_ids.insert(id);
            let in_idr = if let Some(cost) = cost_amount {
                amount.signum() * cost.abs()
            } else if curr != "IDR" {
                convert_minor_units(amount, &curr, "IDR", normalize_fx_rate(entry_fx, fx_rate))
            } else {
                amount
            };
            total_amt = total_amt.saturating_add(in_idr.abs());
        }
        (distinct_ids.len() as i64, total_amt / 2)
    };

    let d30_ahead = if let Ok(today_date) = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d") {
        (today_date + chrono::Duration::days(30))
            .format("%Y-%m-%d")
            .to_string()
    } else {
        format!("{}-31", &today[..7.min(today.len())])
    };

    let (upcoming_due_count, upcoming_due_amount): (i64, i64) = {
        let mut stmt = conn.prepare(
            "SELECT je.id, COALESCE(p.currency, a.currency), p.amount, COALESCE(p.fx_rate, je.fx_rate), p.cost_amount
             FROM journal_entries je
             JOIN postings p ON p.entry_id = je.id
             JOIN accounts a ON a.id = p.account_id
             WHERE je.due_date IS NOT NULL
               AND je.due_date >= ?1
               AND je.due_date <= ?2
               AND (je.notes IS NULL OR je.notes NOT LIKE '%[Settled]%')
               AND je.due_date NOT LIKE '%[Settled]%';",
        )?;
        let rows = stmt.query_map(params![today, d30_ahead], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?,
            ))
        })?;
        let mut distinct_ids = std::collections::HashSet::new();
        let mut total_amt = 0i64;
        for r in rows {
            let (id, curr, amount, entry_fx, cost_amount) = r?;
            distinct_ids.insert(id);
            let in_idr = if let Some(cost) = cost_amount {
                amount.signum() * cost.abs()
            } else if curr != "IDR" {
                convert_minor_units(amount, &curr, "IDR", normalize_fx_rate(entry_fx, fx_rate))
            } else {
                amount
            };
            total_amt = total_amt.saturating_add(in_idr.abs());
        }
        (distinct_ids.len() as i64, total_amt / 2)
    };

    Ok(DashboardMetricsView {
        total_assets: totals.total_assets,
        total_liabilities: totals.total_liabilities,
        net_worth,
        monthly_burn_30d,
        runway_months,
        this_month_income,
        this_month_expense,
        this_month_net,
        overdue_invoices_count,
        overdue_invoices_amount,
        upcoming_due_count,
        upcoming_due_amount,
    })
}

pub fn get_closing_date(conn: &Connection) -> Result<Option<String>, AppError> {
    let mut stmt = conn.prepare("SELECT closing_date FROM book_closing WHERE id = 1;")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let date: Option<String> = row.get(0)?;
        Ok(date)
    } else {
        Ok(None)
    }
}

pub fn set_closing_date(conn: &Connection, closing_date: Option<&str>) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute(
        "INSERT INTO book_closing (id, closing_date, updated_at)
         VALUES (1, ?1, ?2)
         ON CONFLICT(id) DO UPDATE SET closing_date = ?1, updated_at = ?2;",
        params![closing_date, now],
    )?;
    Ok(())
}

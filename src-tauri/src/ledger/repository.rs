use super::dto::{AccountRunningLedgerItem, JournalEntryView, LedgerTotalsView, PostingView};
use super::models::{JournalEntry, Posting, ReconcileState};
use crate::accounts::models::AccountType;
use crate::shared::AppError;
use rusqlite::{params, Connection, Transaction};
use std::str::FromStr;

pub fn insert_entry(tx: &Transaction<'_>, entry: &JournalEntry) -> Result<(), AppError> {
    tx.execute(
        "INSERT INTO journal_entries (id, date, description, notes, currency, fx_rate, posted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
        params![
            entry.id,
            entry.date,
            entry.description,
            entry.notes,
            entry.currency,
            entry.fx_rate,
            entry.posted_at,
        ],
    )?;
    Ok(())
}

pub fn insert_posting(tx: &Transaction<'_>, posting: &Posting) -> Result<(), AppError> {
    tx.execute(
        "INSERT INTO postings (id, entry_id, account_id, amount, memo, action, reconciled, reconciled_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
        params![
            posting.id,
            posting.entry_id,
            posting.account_id,
            posting.amount,
            posting.memo,
            posting.action,
            posting.reconcile.as_str(),
            posting.reconciled_at,
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
            currency = ?5,
            fx_rate = ?6
         WHERE id = ?1;",
        params![
            entry.id,
            entry.date,
            entry.description,
            entry.notes,
            entry.currency,
            entry.fx_rate,
        ],
    )?;
    Ok(())
}

pub fn get_entry_by_id(conn: &Connection, id: &str) -> Result<Option<JournalEntryView>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, date, description, notes, currency, fx_rate, posted_at
         FROM journal_entries WHERE id = ?1;",
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        let entry_id: String = row.get(0)?;
        let date: String = row.get(1)?;
        let description: String = row.get(2)?;
        let notes: Option<String> = row.get(3)?;
        let currency: String = row.get(4)?;
        let fx_rate: i64 = row.get(5)?;
        let posted_at: i64 = row.get(6)?;

        let postings = get_postings_for_entry(conn, &entry_id)?;

        Ok(Some(JournalEntryView {
            id: entry_id,
            date,
            description,
            notes,
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
        "SELECT p.id, p.entry_id, p.account_id, a.code, a.name, a.type, p.amount, p.memo, p.action, p.reconciled
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
        "SELECT id, date, description, notes, currency, fx_rate, posted_at
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
            row.get::<_, String>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
        ))
    })?;

    let mut entries = Vec::new();
    for row in entry_rows {
        let (id, date, description, notes, currency, fx_rate, posted_at) = row?;
        let postings = get_postings_for_entry(conn, &id)?;
        entries.push(JournalEntryView {
            id,
            date,
            description,
            notes,
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
        "SELECT p.entry_id, j.date, j.description, p.amount, p.reconciled
         FROM postings p
         JOIN journal_entries j ON p.entry_id = j.id
         WHERE p.account_id = ?1
         ORDER BY j.date ASC, j.posted_at ASC, p.id ASC;",
    )?;

    let rows = stmt.query_map(params![account_id], |row| {
        let rec_str: String = row.get(4)?;
        let reconcile = ReconcileState::from_str(&rec_str).unwrap_or(ReconcileState::None);

        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            reconcile,
        ))
    })?;

    let mut items = Vec::new();
    let mut running: i64 = 0;
    for r in rows {
        let (entry_id, date, description, amount, reconcile) = r?;
        running += amount;
        items.push(AccountRunningLedgerItem {
            entry_id,
            date,
            description,
            amount,
            running_balance: running,
            reconcile,
        });
    }

    Ok(items)
}

use crate::ledger::currency::{normalize_fx_rate, usd_minor_to_idr};

pub fn calculate_totals(conn: &Connection, fx_rate: i64) -> Result<LedgerTotalsView, AppError> {
    // 1. P&L: Income and Expense recognized at the transaction-time exchange rate
    let mut pl_stmt = conn.prepare(
        "SELECT a.type, a.currency, p.amount, j.fx_rate
         FROM postings p
         JOIN accounts a ON p.account_id = a.id
         JOIN journal_entries j ON p.entry_id = j.id
         WHERE a.type IN ('INCOME', 'EXPENSE');",
    )?;

    let mut total_income: i64 = 0;
    let mut total_expenses: i64 = 0;

    let pl_rows = pl_stmt.query_map([], |row| {
        let type_str: String = row.get(0)?;
        let acc_type = AccountType::from_str(&type_str).unwrap_or(AccountType::Income);
        let curr: String = row.get(1)?;
        let amount: i64 = row.get(2)?;
        let entry_fx: i64 = row.get(3)?;
        Ok((acc_type, curr, amount, entry_fx))
    })?;

    for r in pl_rows {
        let (acc_type, curr, amount, entry_fx) = r?;
        let in_idr = if curr == "USD" {
            usd_minor_to_idr(amount, normalize_fx_rate(entry_fx, fx_rate))
        } else {
            amount
        };
        if acc_type == AccountType::Income {
            total_income += -in_idr;
        } else {
            total_expenses += in_idr;
        }
    }

    // 2. Balance Sheet: Assets, Liabilities, Equity (leaf accounts at spot fx rate)
    let mut bs_stmt = conn.prepare(
        "SELECT a.type, a.currency, COALESCE(SUM(p.amount), 0)
         FROM accounts a
         JOIN postings p ON a.id = p.account_id
         WHERE a.placeholder = 0
           AND a.type IN ('ASSET', 'LIABILITY', 'EQUITY')
           AND a.id NOT IN (SELECT DISTINCT parent_id FROM accounts WHERE parent_id IS NOT NULL)
         GROUP BY a.id, a.type, a.currency;",
    )?;

    let mut total_assets: i64 = 0;
    let mut total_liabilities: i64 = 0;
    let mut total_equity: i64 = 0;

    let bs_rows = bs_stmt.query_map([], |row| {
        let type_str: String = row.get(0)?;
        let acc_type = AccountType::from_str(&type_str).unwrap_or(AccountType::Asset);
        let curr: String = row.get(1)?;
        let amount: i64 = row.get(2)?;
        Ok((acc_type, curr, amount))
    })?;

    for r in bs_rows {
        let (acc_type, curr, amount) = r?;
        let in_idr = if curr == "USD" {
            usd_minor_to_idr(amount, fx_rate)
        } else {
            amount
        };
        match acc_type {
            AccountType::Asset => total_assets += in_idr,
            AccountType::Liability => total_liabilities += -in_idr,
            AccountType::Equity => total_equity += -in_idr,
            _ => {}
        }
    }

    let net_income = total_income - total_expenses;
    let balance_sheet_discrepancy = total_assets - total_liabilities - total_equity - net_income;

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

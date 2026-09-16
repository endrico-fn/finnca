use super::dto::{
    AccountRunningLedgerItem, CreateJournalEntryInput, JournalEntryView, LedgerTotalsView,
    UpdateJournalEntryInput,
};
use super::models::{JournalEntry, Posting, ReconcileState};
use super::repository;
use super::validation;
use crate::accounts::repository as accounts_repo;
use crate::audit::{self, models::AuditAction};
use crate::shared::{generate_id, AppError};
use rusqlite::Connection;

pub fn post_journal_entry(
    conn: &mut Connection,
    input: CreateJournalEntryInput,
    actor: &str,
) -> Result<JournalEntryView, AppError> {
    // 1. Pure validation
    validation::validate_date_format(&input.date)?;
    validation::validate_postings_balance(&input.postings)?;

    if let Some(fx) = input.fx_rate {
        if fx <= 0 {
            return Err(AppError::InvalidInput(
                "fx_rate must be greater than 0".into(),
            ));
        }
    }

    let description = input.description.trim();
    if description.is_empty() {
        return Err(AppError::InvalidInput(
            "Transaction description cannot be empty".into(),
        ));
    }

    // 2. State & Relational validation
    for posting in &input.postings {
        let acc = accounts_repo::get_by_id(conn, &posting.account_id)?.ok_or_else(|| {
            AppError::NotFound(format!("Account '{}' not found", posting.account_id))
        })?;

        if acc.placeholder {
            return Err(AppError::InvalidInput(format!(
                "Cannot post to placeholder account '{}' ({})",
                acc.name, acc.code
            )));
        }
    }

    let entry_id = input.id.unwrap_or_else(generate_id);
    let now = chrono::Utc::now().timestamp_millis();

    let entry = JournalEntry {
        id: entry_id.clone(),
        date: input.date,
        description: description.to_string(),
        notes: input.notes,
        currency: input.currency.unwrap_or_else(|| "IDR".to_string()),
        fx_rate: input.fx_rate.unwrap_or(16000),
        posted_at: now,
    };

    // 3. Database transaction
    let tx = conn.transaction()?;
    repository::insert_entry(&tx, &entry)?;

    for p in input.postings {
        let posting_id = p.id.unwrap_or_else(generate_id);
        let posting = Posting {
            id: posting_id,
            entry_id: entry_id.clone(),
            account_id: p.account_id,
            amount: p.amount,
            memo: p.memo,
            action: p.action,
            reconcile: p.reconcile.unwrap_or(ReconcileState::None),
            reconciled_at: None,
        };
        repository::insert_posting(&tx, &posting)?;
    }

    let _ = audit::record(
        &tx,
        actor,
        AuditAction::PostJournalEntry,
        "JOURNAL_ENTRY",
        &entry_id,
        Some(&format!("Posted '{}'", entry.description)),
    );

    tx.commit()?;

    repository::get_entry_by_id(conn, &entry_id)?
        .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
}

pub fn update_journal_entry(
    conn: &mut Connection,
    id: &str,
    input: UpdateJournalEntryInput,
    actor: &str,
) -> Result<JournalEntryView, AppError> {
    let existing = repository::get_entry_by_id(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("Journal entry '{id}' not found")))?;

    if let Some(date) = &input.date {
        validation::validate_date_format(date)?;
    }

    if let Some(fx) = input.fx_rate {
        if fx <= 0 {
            return Err(AppError::InvalidInput(
                "fx_rate must be greater than 0".into(),
            ));
        }
    }

    if let Some(postings) = &input.postings {
        validation::validate_postings_balance(postings)?;
        for posting in postings {
            let acc = accounts_repo::get_by_id(conn, &posting.account_id)?.ok_or_else(|| {
                AppError::NotFound(format!("Account '{}' not found", posting.account_id))
            })?;

            if acc.placeholder {
                return Err(AppError::InvalidInput(format!(
                    "Cannot post to placeholder account '{}' ({})",
                    acc.name, acc.code
                )));
            }
        }
    }

    let updated_entry = JournalEntry {
        id: id.to_string(),
        date: input.date.unwrap_or(existing.date),
        description: input
            .description
            .unwrap_or(existing.description)
            .trim()
            .to_string(),
        notes: match input.notes {
            Some(n) => n,
            None => existing.notes,
        },
        currency: input.currency.unwrap_or(existing.currency),
        fx_rate: input.fx_rate.unwrap_or(existing.fx_rate),
        posted_at: existing.posted_at,
    };

    let tx = conn.transaction()?;
    repository::update_entry(&tx, &updated_entry)?;

    if let Some(new_postings) = input.postings {
        repository::delete_postings_by_entry(&tx, id)?;
        for p in new_postings {
            let posting_id = p.id.unwrap_or_else(generate_id);
            let posting = Posting {
                id: posting_id,
                entry_id: id.to_string(),
                account_id: p.account_id,
                amount: p.amount,
                memo: p.memo,
                action: p.action,
                reconcile: p.reconcile.unwrap_or(ReconcileState::None),
                reconciled_at: None,
            };
            repository::insert_posting(&tx, &posting)?;
        }
    }

    let _ = audit::record(
        &tx,
        actor,
        AuditAction::UpdateJournalEntry,
        "JOURNAL_ENTRY",
        id,
        Some(&format!("Updated '{}'", updated_entry.description)),
    );

    tx.commit()?;

    repository::get_entry_by_id(conn, id)?
        .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
}

pub fn delete_journal_entry(conn: &mut Connection, id: &str, actor: &str) -> Result<(), AppError> {
    let existing = repository::get_entry_by_id(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("Journal entry '{id}' not found")))?;

    let tx = conn.transaction()?;
    repository::delete_entry(&tx, id)?;
    let _ = audit::record(
        &tx,
        actor,
        AuditAction::DeleteJournalEntry,
        "JOURNAL_ENTRY",
        id,
        Some(&format!("Deleted '{}'", existing.description)),
    );
    tx.commit()?;
    Ok(())
}

pub fn get_journal_entry(conn: &Connection, id: &str) -> Result<JournalEntryView, AppError> {
    repository::get_entry_by_id(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("Journal entry '{id}' not found")))
}

pub fn list_journal_entries(
    conn: &Connection,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<JournalEntryView>, AppError> {
    repository::list_entries(conn, limit, offset)
}

pub fn get_account_running_ledger(
    conn: &Connection,
    account_id: &str,
) -> Result<Vec<AccountRunningLedgerItem>, AppError> {
    repository::get_account_running_ledger(conn, account_id)
}

pub fn get_totals(conn: &Connection, fx_rate: Option<i64>) -> Result<LedgerTotalsView, AppError> {
    let fx = fx_rate.unwrap_or(16000);
    repository::calculate_totals(conn, fx)
}

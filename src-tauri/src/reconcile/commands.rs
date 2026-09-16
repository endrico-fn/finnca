use super::dto::{
    MatchStatementOutput, PostingReconcileView, ReconciliationStatusView, StatementRow,
};
use super::matcher;
use crate::audit::{self, models::AuditAction};
use crate::shared::AppError;
use crate::state::AppState;
use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, State};

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn get_actor(app: &AppHandle) -> String {
    crate::config::load(app)
        .ok()
        .and_then(|c| c.username)
        .unwrap_or_else(|| "user".to_string())
}

pub fn get_account_reconciliation_status(
    conn: &Connection,
    account_id: &str,
) -> Result<ReconciliationStatusView, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.entry_id, je.date, je.description, p.amount, p.memo, p.reconciled, p.reconciled_at
         FROM postings p
         JOIN journal_entries je ON je.id = p.entry_id
         WHERE p.account_id = ?1
         ORDER BY je.date ASC, je.id ASC;",
    )?;

    let rows = stmt.query_map(params![account_id], |row| {
        Ok(PostingReconcileView {
            posting_id: row.get(0)?,
            entry_id: row.get(1)?,
            date: row.get(2)?,
            description: row.get(3)?,
            amount: row.get(4)?,
            memo: row.get(5)?,
            reconciled: row.get(6)?,
            reconciled_at: row.get(7)?,
        })
    })?;

    let mut reconciled_balance: i64 = 0;
    let mut cleared_balance: i64 = 0;
    let mut uncleared_balance: i64 = 0;
    let mut total_balance: i64 = 0;
    let mut uncleared_postings = Vec::new();

    for r in rows {
        let p = r?;
        total_balance += p.amount;

        match p.reconciled.as_str() {
            "y" => {
                reconciled_balance += p.amount;
                cleared_balance += p.amount;
            }
            "c" => {
                cleared_balance += p.amount;
                uncleared_balance += p.amount;
                uncleared_postings.push(p);
            }
            _ => {
                uncleared_balance += p.amount;
                uncleared_postings.push(p);
            }
        }
    }

    Ok(ReconciliationStatusView {
        account_id: account_id.to_string(),
        reconciled_balance,
        cleared_balance,
        uncleared_balance,
        total_balance,
        uncleared_postings,
    })
}

#[tauri::command]
pub fn get_reconciliation_status_cmd(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<ReconciliationStatusView, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    get_account_reconciliation_status(&conn, &account_id)
}

#[tauri::command]
pub fn set_posting_reconciled_cmd(
    state: State<'_, AppState>,
    posting_id: String,
    status: String,
) -> Result<(), AppError> {
    if !matches!(status.as_str(), "n" | "c" | "y") {
        return Err(AppError::InvalidInput(
            "reconciled status must be 'n', 'c', or 'y'".into(),
        ));
    }

    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;

    let reconciled_at = if status == "y" {
        Some(now_unix())
    } else {
        None
    };

    let affected = conn.execute(
        "UPDATE postings SET reconciled = ?2, reconciled_at = ?3 WHERE id = ?1;",
        params![posting_id, status, reconciled_at],
    )?;

    if affected == 0 {
        return Err(AppError::NotFound(format!("posting id={posting_id}")));
    }

    Ok(())
}

#[tauri::command]
pub fn bulk_set_postings_reconciled_cmd(
    state: State<'_, AppState>,
    posting_ids: Vec<String>,
    status: String,
) -> Result<(), AppError> {
    if !matches!(status.as_str(), "n" | "c" | "y") {
        return Err(AppError::InvalidInput(
            "reconciled status must be 'n', 'c', or 'y'".into(),
        ));
    }

    let db = state.get_db()?;
    let mut conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    let tx = conn.transaction()?;

    let reconciled_at = if status == "y" {
        Some(now_unix())
    } else {
        None
    };

    {
        let mut stmt =
            tx.prepare("UPDATE postings SET reconciled = ?2, reconciled_at = ?3 WHERE id = ?1;")?;
        for id in posting_ids {
            stmt.execute(params![id, status, reconciled_at])?;
        }
    }

    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn finish_reconciliation_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    account_id: String,
    posting_ids: Vec<String>,
) -> Result<(), AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let mut conn = db.lock().map_err(|_| AppError::VaultLocked)?;

    let now = now_unix();
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "UPDATE postings SET reconciled = 'y', reconciled_at = ?3
             WHERE id = ?1 AND account_id = ?2;",
        )?;

        for id in &posting_ids {
            stmt.execute(params![id, account_id, now])?;
        }
    }

    let _ = audit::record(
        &tx,
        &actor,
        AuditAction::ReconcileAccount,
        "ACCOUNT",
        &account_id,
        Some(&format!(
            "Reconciled {} postings for account {}",
            posting_ids.len(),
            account_id
        )),
    );

    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn match_statement_cmd(
    state: State<'_, AppState>,
    account_id: String,
    statements: Vec<StatementRow>,
    auto_clear: bool,
) -> Result<MatchStatementOutput, AppError> {
    let db = state.get_db()?;
    let mut conn = db.lock().map_err(|_| AppError::VaultLocked)?;

    let status = get_account_reconciliation_status(&conn, &account_id)?;
    let result =
        matcher::match_statements_against_postings(&statements, &status.uncleared_postings, 4);

    if auto_clear && !result.matches.is_empty() {
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "UPDATE postings SET reconciled = 'c' WHERE id = ?1 AND reconciled = 'n';",
            )?;
            for m in &result.matches {
                stmt.execute(params![m.posting_id])?;
            }
        }
        tx.commit()?;
    }

    Ok(result)
}

const MAX_STATEMENT_FILE_BYTES: u64 = 5 * 1024 * 1024;

#[tauri::command]
pub fn read_statement_file_cmd(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, AppError> {
    let _ = state.get_db()?;

    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("file path cannot be empty".into()));
    }

    let p = std::path::Path::new(trimmed);
    if !p.exists() || !p.is_file() {
        return Err(AppError::NotFound(format!("statement file at '{trimmed}'")));
    }

    let meta = std::fs::metadata(p)?;
    if meta.len() > MAX_STATEMENT_FILE_BYTES {
        return Err(AppError::InvalidInput(format!(
            "statement file size ({} bytes) exceeds 5 MB limit",
            meta.len()
        )));
    }

    let content = std::fs::read_to_string(p)?;
    Ok(content)
}

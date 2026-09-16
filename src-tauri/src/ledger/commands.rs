use super::dto::{
    AccountRunningLedgerItem, CreateJournalEntryInput, JournalEntryView, LedgerTotalsView,
    UpdateJournalEntryInput,
};
use super::service;
use crate::shared::AppError;
use crate::state::AppState;
use tauri::{AppHandle, State};

fn get_actor(app: &AppHandle) -> String {
    crate::config::load(app)
        .ok()
        .and_then(|c| c.username)
        .unwrap_or_else(|| "user".to_string())
}

#[tauri::command]
pub fn post_journal_entry_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateJournalEntryInput,
) -> Result<JournalEntryView, AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let mut conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::post_journal_entry(&mut conn, input, &actor)
}

#[tauri::command]
pub fn update_journal_entry_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdateJournalEntryInput,
) -> Result<JournalEntryView, AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let mut conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::update_journal_entry(&mut conn, &id, input, &actor)
}

#[tauri::command]
pub fn delete_journal_entry_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let mut conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::delete_journal_entry(&mut conn, &id, &actor)
}

#[tauri::command]
pub fn get_journal_entry_cmd(
    state: State<'_, AppState>,
    id: String,
) -> Result<JournalEntryView, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::get_journal_entry(&conn, &id)
}

#[tauri::command]
pub fn list_journal_entries_cmd(
    state: State<'_, AppState>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<JournalEntryView>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::list_journal_entries(&conn, limit, offset)
}

#[tauri::command]
pub fn get_account_ledger_cmd(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<Vec<AccountRunningLedgerItem>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::get_account_running_ledger(&conn, &account_id)
}

#[tauri::command]
pub fn get_ledger_totals_cmd(
    state: State<'_, AppState>,
    fx_rate: Option<i64>,
) -> Result<LedgerTotalsView, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::get_totals(&conn, fx_rate)
}

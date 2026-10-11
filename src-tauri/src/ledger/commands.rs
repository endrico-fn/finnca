use super::dto::{
    AccountRunningLedgerItem, CreateJournalEntryInput, DashboardMetricsView, JournalEntryView,
    LedgerTotalsView, UpdateJournalEntryInput, VaultHealthReport,
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
#[specta::specta]
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
#[specta::specta]
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
#[specta::specta]
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
#[specta::specta]
pub fn get_journal_entry_cmd(
    state: State<'_, AppState>,
    id: String,
) -> Result<JournalEntryView, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::get_journal_entry(&conn, &id)
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
pub fn get_account_ledger_cmd(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<Vec<AccountRunningLedgerItem>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::get_account_running_ledger(&conn, &account_id)
}

#[tauri::command]
#[specta::specta]
pub fn get_ledger_totals_cmd(
    state: State<'_, AppState>,
    fx_rate: Option<i64>,
) -> Result<LedgerTotalsView, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::get_totals(&conn, fx_rate)
}

#[tauri::command]
#[specta::specta]
pub fn get_dashboard_metrics_cmd(
    state: State<'_, AppState>,
    fx_rate: Option<i64>,
    today: Option<String>,
) -> Result<DashboardMetricsView, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::get_dashboard_metrics(&conn, fx_rate, today)
}

#[tauri::command]
#[specta::specta]
pub fn diagnose_vault_health_cmd(
    state: State<'_, AppState>,
) -> Result<VaultHealthReport, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::diagnose_vault_health(&conn)
}

#[tauri::command]
#[specta::specta]
pub fn get_closing_date_cmd(state: State<'_, AppState>) -> Result<Option<String>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::get_closing_date(&conn)
}

#[tauri::command]
#[specta::specta]
pub fn set_closing_date_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    closing_date: Option<String>,
) -> Result<(), AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::set_closing_date(&conn, closing_date.as_deref(), &actor)
}

#[tauri::command]
#[specta::specta]
pub fn export_beancount_cmd(
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<String, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    let content = super::beancount_export::generate_beancount_export(&conn)?;
    if let Some(target_path) = path {
        let session =
            crate::vault::commands::require_session(&state).map_err(AppError::InvalidInput)?;
        let safe_path = crate::vault::commands::validate_safe_export_target(
            &target_path,
            &session,
            content.len(),
        )
        .map_err(AppError::InvalidInput)?;
        crate::atomic_write(&safe_path, content.as_bytes())
            .map_err(|e| AppError::Io(std::io::Error::other(e)))?;
    }
    Ok(content)
}

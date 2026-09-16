use super::dto::{AccountBalanceView, CreateAccountInput, UpdateAccountInput};
use super::models::Account;
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
pub fn create_account_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateAccountInput,
) -> Result<Account, AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::create_account(&conn, input, &actor)
}

#[tauri::command]
pub fn update_account_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdateAccountInput,
) -> Result<Account, AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::update_account(&conn, &id, input, &actor)
}

#[tauri::command]
pub fn delete_account_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::delete_account(&conn, &id, &actor)
}

#[tauri::command]
pub fn get_account_cmd(state: State<'_, AppState>, id: String) -> Result<Account, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::get_account(&conn, &id)
}

#[tauri::command]
pub fn list_accounts_cmd(state: State<'_, AppState>) -> Result<Vec<AccountBalanceView>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::list_accounts_with_balances(&conn)
}

#[tauri::command]
pub fn seed_root_accounts_cmd(state: State<'_, AppState>) -> Result<Vec<Account>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::seed_root_placeholder_accounts(&conn, "IDR")
}

#[tauri::command]
pub fn seed_starter_accounts_cmd(
    state: State<'_, AppState>,
    language: Option<String>,
) -> Result<Vec<Account>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    let lang = language.as_deref().unwrap_or("en");
    service::seed_comprehensive_accounts(&conn, lang, "IDR")
}

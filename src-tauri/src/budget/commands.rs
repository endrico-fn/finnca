use super::dto::{BudgetMonthSummary, UpsertBudgetInput};
use super::models::BudgetAllocation;
use super::service;
use crate::shared::AppError;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn upsert_budget_cmd(
    state: State<'_, AppState>,
    input: UpsertBudgetInput,
) -> Result<BudgetAllocation, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::upsert_budget(&conn, input)
}

#[tauri::command]
pub fn delete_budget_cmd(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::delete_budget(&conn, &id)
}

#[tauri::command]
pub fn get_budget_summary_cmd(
    state: State<'_, AppState>,
    month: String,
) -> Result<BudgetMonthSummary, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::get_budget_month_summary(&conn, &month)
}

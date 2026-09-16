use super::dto::{CreatePlanInput, PlanProgressView, UpdatePlanInput};
use super::models::PaymentPlan;
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
pub fn create_plan_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreatePlanInput,
) -> Result<PaymentPlan, AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::create_plan(&conn, input, &actor)
}

#[tauri::command]
pub fn update_plan_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdatePlanInput,
) -> Result<PaymentPlan, AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::update_plan(&conn, &id, input, &actor)
}

#[tauri::command]
pub fn delete_plan_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::delete_plan(&conn, &id, &actor)
}

#[tauri::command]
pub fn get_plan_cmd(state: State<'_, AppState>, id: String) -> Result<PaymentPlan, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::get_plan(&conn, &id)
}

#[tauri::command]
pub fn list_plans_cmd(state: State<'_, AppState>) -> Result<Vec<PaymentPlan>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::list_plans(&conn)
}

#[tauri::command]
pub fn list_plans_with_progress_cmd(
    state: State<'_, AppState>,
) -> Result<Vec<PlanProgressView>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    service::list_plans_with_progress(&conn)
}

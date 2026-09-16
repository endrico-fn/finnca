use super::dto::AuditPage;
use super::repository;
use crate::shared::AppError;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_audit_log_cmd(
    state: State<'_, AppState>,
    page: u32,
    per_page: u32,
) -> Result<AuditPage, AppError> {
    let per_page = per_page.clamp(1, 200);
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    let (entries, total) = repository::list_paginated(&conn, page.max(1), per_page)?;
    Ok(AuditPage {
        entries,
        total,
        page: page.max(1),
        per_page,
    })
}

#[tauri::command]
pub fn get_entity_audit_log_cmd(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<super::models::AuditEntry>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    repository::list_by_entity(&conn, &entity_type, &entity_id)
}

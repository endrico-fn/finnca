pub mod commands;
pub mod dto;
pub mod models;
pub mod repository;

use crate::shared::AppError;
use models::AuditAction;
use rusqlite::Connection;

pub fn record(
    conn: &Connection,
    actor: &str,
    action: AuditAction,
    entity_type: &str,
    entity_id: &str,
    detail: Option<&str>,
) -> Result<(), AppError> {
    repository::insert(conn, actor, action, entity_type, entity_id, detail)
}

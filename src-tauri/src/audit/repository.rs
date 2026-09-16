use super::models::{AuditAction, AuditEntry};
use crate::shared::AppError;
use rusqlite::{params, Connection, Row};
use std::time::{SystemTime, UNIX_EPOCH};
use ulid::Ulid;

fn map_row(row: &Row<'_>) -> rusqlite::Result<AuditEntry> {
    Ok(AuditEntry {
        id: row.get(0)?,
        actor: row.get(1)?,
        action: row.get(2)?,
        entity_type: row.get(3)?,
        entity_id: row.get(4)?,
        detail: row.get(5)?,
        created_at: row.get(6)?,
    })
}

pub fn insert(
    conn: &Connection,
    actor: &str,
    action: AuditAction,
    entity_type: &str,
    entity_id: &str,
    detail: Option<&str>,
) -> Result<(), AppError> {
    let id = Ulid::new().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO audit_log (id, actor, action, entity_type, entity_id, detail, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
        params![
            id,
            actor,
            action.as_str(),
            entity_type,
            entity_id,
            detail,
            now
        ],
    )?;
    Ok(())
}

pub fn list_paginated(
    conn: &Connection,
    page: u32,
    per_page: u32,
) -> Result<(Vec<AuditEntry>, i64), AppError> {
    let offset = page.saturating_sub(1) * per_page;

    let total: i64 = conn.query_row("SELECT count(*) FROM audit_log;", [], |r| r.get(0))?;

    let mut stmt = conn.prepare(
        "SELECT id, actor, action, entity_type, entity_id, detail, created_at
         FROM audit_log
         ORDER BY created_at DESC
         LIMIT ?1 OFFSET ?2;",
    )?;

    let rows = stmt.query_map(params![per_page, offset], map_row)?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }

    Ok((entries, total))
}

pub fn list_by_entity(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> Result<Vec<AuditEntry>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, actor, action, entity_type, entity_id, detail, created_at
         FROM audit_log
         WHERE entity_type = ?1 AND entity_id = ?2
         ORDER BY created_at DESC;",
    )?;

    let rows = stmt.query_map(params![entity_type, entity_id], map_row)?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

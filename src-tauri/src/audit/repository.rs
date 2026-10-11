use super::models::{AuditAction, AuditEntry, AuditIntegrityReport};
use crate::shared::AppError;
use rusqlite::{params, Connection, OptionalExtension, Row};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use ulid::Ulid;

pub fn compute_entry_hash(
    prev_hash: &str,
    actor: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    detail: Option<&str>,
    created_at: i64,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(b"|");
    hasher.update(actor.as_bytes());
    hasher.update(b"|");
    hasher.update(action.as_bytes());
    hasher.update(b"|");
    hasher.update(entity_type.as_bytes());
    hasher.update(b"|");
    hasher.update(entity_id.as_bytes());
    hasher.update(b"|");
    hasher.update(detail.unwrap_or("").as_bytes());
    hasher.update(b"|");
    hasher.update(created_at.to_string().as_bytes());
    hex::encode(hasher.finalize())
}

fn map_row(row: &Row<'_>) -> rusqlite::Result<AuditEntry> {
    Ok(AuditEntry {
        id: row.get(0)?,
        actor: row.get(1)?,
        action: row.get(2)?,
        entity_type: row.get(3)?,
        entity_id: row.get(4)?,
        detail: row.get(5)?,
        created_at: row.get(6)?,
        prev_hash: row.get(7)?,
        hash: row.get(8)?,
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

    let last_hash: Option<String> = conn
        .query_row(
            "SELECT hash FROM audit_log WHERE hash IS NOT NULL ORDER BY created_at DESC, rowid DESC LIMIT 1;",
            [],
            |r| r.get(0),
        )
        .optional()?;

    let prev_hash = last_hash.unwrap_or_else(|| "GENESIS".to_string());
    let hash = compute_entry_hash(
        &prev_hash,
        actor,
        action.as_str(),
        entity_type,
        entity_id,
        detail,
        now,
    );

    conn.execute(
        "INSERT INTO audit_log (id, actor, action, entity_type, entity_id, detail, created_at, prev_hash, hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
        params![
            id,
            actor,
            action.as_str(),
            entity_type,
            entity_id,
            detail,
            now,
            prev_hash,
            hash
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
        "SELECT id, actor, action, entity_type, entity_id, detail, created_at, prev_hash, hash
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
        "SELECT id, actor, action, entity_type, entity_id, detail, created_at, prev_hash, hash
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

pub fn verify_integrity(conn: &Connection) -> Result<AuditIntegrityReport, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, actor, action, entity_type, entity_id, detail, created_at, prev_hash, hash
         FROM audit_log
         WHERE hash IS NOT NULL
         ORDER BY created_at ASC, rowid ASC;",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, Option<String>>(8)?,
        ))
    })?;

    let mut expected_prev = "GENESIS".to_string();
    let mut count = 0;

    for (idx, item) in rows.enumerate() {
        let (
            id,
            actor,
            action,
            entity_type,
            entity_id,
            detail,
            created_at,
            prev_hash_opt,
            hash_opt,
        ) = item?;
        let prev_hash = prev_hash_opt.unwrap_or_default();
        let current_hash = hash_opt.unwrap_or_default();

        if prev_hash != expected_prev {
            return Ok(AuditIntegrityReport {
                is_valid: false,
                total_verified: count,
                broken_index: Some(idx as i64),
                error_message: Some(format!(
                    "Hash chain broken at entry {}: prev_hash mismatch",
                    id
                )),
            });
        }

        let computed = compute_entry_hash(
            &prev_hash,
            &actor,
            &action,
            &entity_type,
            &entity_id,
            detail.as_deref(),
            created_at,
        );

        if computed != current_hash {
            return Ok(AuditIntegrityReport {
                is_valid: false,
                total_verified: count,
                broken_index: Some(idx as i64),
                error_message: Some(format!(
                    "Hash chain broken at entry {}: hash tampering detected",
                    id
                )),
            });
        }

        expected_prev = current_hash;
        count += 1;
    }

    Ok(AuditIntegrityReport {
        is_valid: true,
        total_verified: count,
        broken_index: None,
        error_message: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;

    #[test]
    fn test_audit_hash_chain_integrity_and_tamper_detection() {
        let mut conn = Connection::open_in_memory().expect("open memory db");
        run_migrations(&mut conn).expect("run migrations");

        insert(
            &conn,
            "system",
            AuditAction::CreateAccount,
            "ACCOUNT",
            "acc-1",
            Some("initial account"),
        )
        .expect("insert 1");

        insert(
            &conn,
            "alice",
            AuditAction::PostJournalEntry,
            "JOURNAL_ENTRY",
            "tx-1",
            Some("salary entry"),
        )
        .expect("insert 2");

        insert(
            &conn,
            "alice",
            AuditAction::UpdateClosingDate,
            "CLOSING_DATE",
            "2026-08-31",
            None,
        )
        .expect("insert 3");

        let report = verify_integrity(&conn).expect("verify integrity");
        assert!(report.is_valid);
        assert_eq!(report.total_verified, 3);
        assert!(report.broken_index.is_none());

        conn.execute(
            "UPDATE audit_log SET detail = 'tampered detail' WHERE entity_id = 'tx-1';",
            [],
        )
        .expect("tamper db");

        let tampered_report = verify_integrity(&conn).expect("verify tampered");
        assert!(!tampered_report.is_valid);
        assert_eq!(tampered_report.broken_index, Some(1));
    }
}

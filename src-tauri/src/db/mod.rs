pub mod schema;

use crate::shared::AppError;
use rusqlite::Connection;
use std::path::Path;
use zeroize::Zeroizing;

pub fn open_vault_db(path: &Path, dek: &[u8; 32]) -> Result<Connection, AppError> {
    let mut conn = Connection::open(path)?;

    // 1. Set key via SQLCipher raw hex format with heap zeroization
    {
        let dek_hex = Zeroizing::new(hex::encode(dek));
        let pragma_cmd = Zeroizing::new(format!("PRAGMA key = \"x'{}'\";", dek_hex.as_str()));
        conn.execute_batch(&pragma_cmd)?;
    }

    // 2. Set performance, memory-security, and in-memory temp storage pragmas
    conn.execute_batch(
        "
        PRAGMA cipher_compatibility = 4;
        PRAGMA cipher_memory_security = ON;
        PRAGMA cipher_default_temp_store = MEMORY;
        PRAGMA temp_store = MEMORY;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;
    ",
    )?;

    // 3. Verify key validity by querying master table
    let check: Result<i64, _> =
        conn.query_row("SELECT count(*) FROM sqlite_master;", [], |r| r.get(0));
    if let Err(e) = check {
        return Err(AppError::Crypto(format!(
            "Failed to decrypt vault database: {e}"
        )));
    }

    // 4. Validate database integrity on startup
    let quick_check: String = conn.query_row("PRAGMA quick_check;", [], |r| r.get(0))?;
    if quick_check != "ok" {
        return Err(AppError::Crypto(format!(
            "Database integrity check failed: {quick_check}"
        )));
    }

    // 5. Run schema migrations
    schema::run_migrations(&mut conn)?;

    Ok(conn)
}

pub fn export_encrypted_snapshot(
    conn: &Connection,
    dest_path: &Path,
    target_dek: &[u8; 32],
) -> Result<(), AppError> {
    let dest_str = dest_path
        .to_str()
        .ok_or_else(|| AppError::InvalidInput("Destination path is not valid UTF-8".into()))?;
    let user_version: u32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;

    let safe_dest = dest_str.replace('\'', "''");
    let target_dek_hex = Zeroizing::new(hex::encode(target_dek));
    let attach_sql = Zeroizing::new(format!(
        "ATTACH DATABASE '{safe_dest}' AS backup KEY \"x'{}'\";",
        target_dek_hex.as_str()
    ));
    conn.execute_batch(&attach_sql)?;

    let export_res = conn.execute_batch(&format!(
        "SELECT sqlcipher_export('backup'); PRAGMA backup.user_version = {user_version};"
    ));

    let detach_res = conn.execute_batch("DETACH DATABASE backup;");

    export_res?;
    detach_res?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_database_roundtrip() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let db_path = temp_dir.path().join("vault.db");
        let dek1: [u8; 32] = [42u8; 32];
        let dek2: [u8; 32] = [99u8; 32];

        // 1. Create and write with dek1
        {
            let conn = open_vault_db(&db_path, &dek1).expect("open db with dek1");
            conn.execute(
                "INSERT INTO accounts (id, code, name, type, currency, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
                rusqlite::params!["acc_1", "1000", "Cash", "ASSET", "IDR", 1700000000],
            ).expect("insert account");
        }

        // 2. Try opening with incorrect dek2 -> must fail
        {
            let err = open_vault_db(&db_path, &dek2);
            assert!(err.is_err(), "Opening with incorrect DEK must fail");
        }

        // 3. Re-open with correct dek1 -> must succeed and data must be intact
        {
            let conn = open_vault_db(&db_path, &dek1).expect("reopen db with dek1");
            let name: String = conn
                .query_row("SELECT name FROM accounts WHERE id = ?1;", ["acc_1"], |r| {
                    r.get(0)
                })
                .expect("query account");
            assert_eq!(name, "Cash");
        }
    }

    #[test]
    fn test_sqlcipher_export_snapshot() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let src_path = temp_dir.path().join("source.db");
        let dest_path = temp_dir.path().join("backup.db");
        let dek_src: [u8; 32] = [11u8; 32];
        let dek_backup: [u8; 32] = [77u8; 32];

        // 1. Create source database with custom migration data
        {
            let conn = open_vault_db(&src_path, &dek_src).expect("open src db");
            conn.execute(
                "INSERT INTO accounts (id, code, name, type, currency, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
                rusqlite::params!["acc_backup", "1010", "Bank Central", "ASSET", "IDR", 1700000000],
            ).expect("insert account");

            // 2. Export encrypted snapshot with dek_backup
            export_encrypted_snapshot(&conn, &dest_path, &dek_backup).expect("export snapshot");
        }

        // 3. Verify backup file exists and is encrypted (cannot be read with plain SQLite)
        {
            let plain_conn = rusqlite::Connection::open(&dest_path).expect("open plain");
            let check: Result<i64, _> =
                plain_conn.query_row("SELECT count(*) FROM sqlite_master;", [], |r| r.get(0));
            assert!(
                check.is_err(),
                "Backup file MUST NOT be readable as plain unencrypted SQLite!"
            );
        }

        // 4. Verify backup file CAN be decrypted and read with dek_backup
        {
            let backup_conn =
                open_vault_db(&dest_path, &dek_backup).expect("open backup with dek_backup");
            let name: String = backup_conn
                .query_row(
                    "SELECT name FROM accounts WHERE id = ?1;",
                    ["acc_backup"],
                    |r| r.get(0),
                )
                .expect("query account from backup");
            assert_eq!(name, "Bank Central");

            let version: u32 = backup_conn
                .query_row("PRAGMA user_version;", [], |r| r.get(0))
                .expect("query user_version");
            assert!(version >= 9, "Schema user_version must be preserved");
        }
    }
}

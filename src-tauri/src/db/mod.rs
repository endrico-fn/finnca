pub mod schema;

use crate::shared::AppError;
use rusqlite::Connection;
use std::path::Path;

pub fn open_vault_db(path: &Path, dek: &[u8; 32]) -> Result<Connection, AppError> {
    let mut conn = Connection::open(path)?;

    // 1. Set key via SQLCipher raw hex format
    let dek_hex = hex::encode(dek);
    conn.execute_batch(&format!("PRAGMA key = \"x'{dek_hex}'\";"))?;

    // 2. Set performance and integrity pragmas
    conn.execute_batch(
        "
        PRAGMA cipher_compatibility = 4;
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

    // 4. Run schema migrations
    schema::run_migrations(&mut conn)?;

    Ok(conn)
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
}

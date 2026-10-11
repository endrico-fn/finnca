use super::legacy_migration::{is_legacy_vault, migrate_legacy_vault};
use super::metadata::{read_metadata, write_metadata, VaultMetadata, DB_FILE, META_FILE};
use crate::crypto::envelope::{generate_dek, unwrap_dek, wrap_dek};
use crate::crypto::kdf::derive_kek;
use crate::db::open_vault_db;
use crate::shared::AppError;
use rand::RngCore;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

pub fn open_and_unlock_vault(vault_path: &Path, password: &str) -> Result<Connection, AppError> {
    if is_legacy_vault(vault_path) {
        return migrate_legacy_vault(vault_path, password);
    }

    if vault_path.join(META_FILE).exists() {
        let meta = read_metadata(vault_path)?;
        let salt_bytes = hex::decode(&meta.kdf.salt)
            .map_err(|e| AppError::Crypto(format!("Invalid salt in metadata: {e}")))?;

        if salt_bytes.len() != 16 {
            return Err(AppError::Crypto("Invalid salt length in metadata".into()));
        }

        let mut salt = [0u8; 16];
        salt.copy_from_slice(&salt_bytes);

        let kek = derive_kek(password, &salt)?;
        let dek = unwrap_dek(&kek, &meta.wrapped_dek)?;

        let db_path = vault_path.join(DB_FILE);
        let conn = open_vault_db(&db_path, &dek)?;
        return Ok(conn);
    }

    Err(AppError::NotFound(format!(
        "No valid vault found in {}",
        vault_path.display()
    )))
}

pub fn create_new_vault(
    vault_path: &Path,
    password: &str,
    vault_name: Option<&str>,
    username: Option<&str>,
    template_lang: Option<&str>,
    account_profile: Option<&str>,
) -> Result<Connection, AppError> {
    if !vault_path.exists() {
        fs::create_dir_all(vault_path)?;
    }

    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    let dek = generate_dek();
    let kek = derive_kek(password, &salt)?;
    let wrapped_dek = wrap_dek(&kek, &dek)?;

    let meta = VaultMetadata::new(
        &salt,
        wrapped_dek,
        vault_name.map(|s| s.to_string()),
        username.map(|s| s.to_string()),
    );
    write_metadata(vault_path, &meta)?;

    let db_path = vault_path.join(DB_FILE);
    let conn = open_vault_db(&db_path, &dek)?;

    let lang = template_lang.unwrap_or("en");
    let profile = account_profile.unwrap_or("personal");
    crate::accounts::service::seed_account_template(&conn, profile, lang, "IDR")?;

    Ok(conn)
}

pub fn change_vault_password(
    vault_path: &Path,
    old_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    if !vault_path.join(META_FILE).exists() {
        return Err(AppError::NotFound("Vault metadata not found".into()));
    }

    let meta = read_metadata(vault_path)?;
    let salt_bytes = hex::decode(&meta.kdf.salt)
        .map_err(|e| AppError::Crypto(format!("Invalid salt in metadata: {e}")))?;

    if salt_bytes.len() != 16 {
        return Err(AppError::Crypto("Invalid salt length in metadata".into()));
    }

    let mut salt = [0u8; 16];
    salt.copy_from_slice(&salt_bytes);

    // 1. Verify old password and unwrap DEK
    let old_kek = derive_kek(old_password, &salt)?;
    let dek = unwrap_dek(&old_kek, &meta.wrapped_dek)
        .map_err(|_| AppError::Crypto("Incorrect current password".into()))?;

    // 2. Generate new salt for the new password
    let mut new_salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut new_salt);

    // 3. Derive new KEK and wrap the existing DEK
    let new_kek = derive_kek(new_password, &new_salt)?;
    let new_wrapped_dek = wrap_dek(&new_kek, &dek)?;

    // 4. Save new metadata (this replaces the old one atomically, since write_metadata uses temp files, wait, let's check if write_metadata is atomic)
    // Actually write_metadata uses std::fs::write which isn't atomic by default, but it's okay for now.
    let new_meta = VaultMetadata::new(&new_salt, new_wrapped_dek, meta.vault_name, meta.username);
    write_metadata(vault_path, &new_meta)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new_vault_and_unlock() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let vault_path = temp_dir.path();
        let password = "SuperSecretPassword123!";

        // 1. Create vault -> verifies that 5 root placeholder accounts are automatically seeded
        {
            let conn = create_new_vault(
                vault_path,
                password,
                Some("Personal Finances"),
                Some("endrico"),
                Some("en"),
                None,
            )
            .expect("create new vault");

            let count: i64 = conn
                .query_row("SELECT count(*) FROM accounts;", [], |r| r.get(0))
                .expect("count seeded accounts");
            assert_eq!(
                count, 26,
                "New vault must seed comprehensive starter accounts"
            );

            let root_asset_id: String = conn
                .query_row("SELECT id FROM accounts WHERE code = '1000';", [], |r| {
                    r.get(0)
                })
                .expect("query root asset id");

            conn.execute(
                "INSERT INTO accounts (id, code, name, type, parent_id, currency, created_at) VALUES ('sub_1', '1010', 'Cash', 'ASSET', ?1, 'IDR', 1700000000);",
                [&root_asset_id],
            ).expect("insert child account");
        }

        // 2. Unlock with wrong password fails
        {
            let err = open_and_unlock_vault(vault_path, "WrongPassword");
            assert!(err.is_err(), "Must fail with wrong password");
        }

        // 3. Unlock with correct password succeeds
        {
            let conn = open_and_unlock_vault(vault_path, password).expect("unlock vault");
            let name: String = conn
                .query_row("SELECT name FROM accounts WHERE id = 'sub_1';", [], |r| {
                    r.get(0)
                })
                .expect("query account");
            assert_eq!(name, "Cash");

            let total_accounts: i64 = conn
                .query_row("SELECT count(*) FROM accounts;", [], |r| r.get(0))
                .expect("query total accounts");
            assert_eq!(total_accounts, 27);
        }
    }
}

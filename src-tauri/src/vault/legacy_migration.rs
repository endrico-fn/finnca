use super::metadata::{write_metadata, VaultMetadata, DB_FILE};
use crate::accounts::models::{Account, AccountType};
use crate::accounts::repository as accounts_repo;
use crate::config::{DATA_FILE, KEY_FILE};
use crate::crypto::envelope::{generate_dek, wrap_dek};
use crate::crypto::kdf::derive_kek;
use crate::crypto::legacy::{scrypt_decrypt, x25519_decrypt};
use crate::db::open_vault_db;
use crate::ledger::dto::PostingInput;
use crate::ledger::models::ReconcileState;
use crate::ledger::service as ledger_service;
use crate::shared::AppError;
use rand::RngCore;
use rusqlite::Connection;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct LegacyAccount {
    pub id: String,
    pub code: String,
    pub name: String,
    #[serde(rename = "type")]
    pub acc_type: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub placeholder: bool,
    #[serde(default)]
    pub hidden: bool,
    pub color: Option<String>,
    pub note: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "interestRate")]
    pub interest_rate: Option<f64>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

fn default_currency() -> String {
    "IDR".to_string()
}

#[derive(Debug, Deserialize)]
pub struct LegacySplit {
    pub id: Option<String>,
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub action: Option<String>,
    #[serde(default)]
    pub reconcile: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LegacyTransaction {
    pub id: String,
    pub date: String,
    pub description: String,
    pub notes: Option<String>,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(rename = "fxRateAtTransaction")]
    pub fx_rate_at_transaction: Option<i64>,
    pub splits: Vec<LegacySplit>,
}

#[derive(Debug, Deserialize)]
pub struct LegacyVaultData {
    pub version: u32,
    pub accounts: Vec<LegacyAccount>,
    pub transactions: Vec<LegacyTransaction>,
    #[serde(rename = "fxRate", default = "default_fx_rate")]
    pub fx_rate: i64,
}

fn default_fx_rate() -> i64 {
    crate::ledger::currency::DEFAULT_FX_RATE
}

pub fn is_legacy_vault(vault_path: &Path) -> bool {
    let has_key = vault_path.join(KEY_FILE).exists();
    let has_data = vault_path.join(DATA_FILE).exists();
    let has_db = vault_path.join(DB_FILE).exists();
    has_key && has_data && !has_db
}

pub fn migrate_legacy_vault(vault_path: &Path, password: &str) -> Result<Connection, AppError> {
    let key_path = vault_path.join(KEY_FILE);
    let data_path = vault_path.join(DATA_FILE);
    let db_path = vault_path.join(DB_FILE);

    if !key_path.exists() || !data_path.exists() {
        return Err(AppError::NotFound(format!(
            "Legacy vault files not found in {}",
            vault_path.display()
        )));
    }

    // 1. Decrypt legacy key using password (scrypt)
    let encrypted_key = fs::read(&key_path)?;
    let serialized_key_bytes = scrypt_decrypt(password, &encrypted_key)
        .map_err(|e| AppError::Crypto(format!("Failed to decrypt vault.key: {e}")))?;
    let serialized_key = String::from_utf8(serialized_key_bytes)
        .map_err(|_| AppError::Crypto("vault.key UTF-8 decoding failed".into()))?;
    let identity: age::x25519::Identity = serialized_key
        .parse()
        .map_err(|_| AppError::Crypto("Failed to parse x25519 identity".into()))?;

    // 2. Decrypt legacy vault.age using x25519 identity
    let encrypted_data = fs::read(&data_path)?;
    let plaintext_bytes = x25519_decrypt(&identity, &encrypted_data)
        .map_err(|e| AppError::Crypto(format!("Failed to decrypt vault.age: {e}")))?;

    let legacy_vault: LegacyVaultData = serde_json::from_slice(&plaintext_bytes)
        .map_err(|e| AppError::Crypto(format!("Failed to parse legacy vault JSON: {e}")))?;

    // 3. Backup legacy files (Backup-Before-Write)
    let timestamp = chrono::Utc::now().timestamp();
    let backup_path = vault_path.join(format!("{DATA_FILE}.bak.{timestamp}"));
    fs::copy(&data_path, &backup_path)?;
    let migrated_copy = vault_path.join(format!("{DATA_FILE}.migrated"));
    let _ = fs::copy(&data_path, &migrated_copy);

    // 4. Cryptographic Envelope setup for SQLite
    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    let dek = generate_dek();
    let kek = derive_kek(password, &salt)?;
    let wrapped_dek = wrap_dek(&kek, &dek)?;
    let metadata = VaultMetadata::new(&salt, wrapped_dek, None, None);

    // 5. Open and migrate new SQLite database
    let mut conn = match open_vault_db(&db_path, &dek) {
        Ok(c) => c,
        Err(e) => {
            let _ = fs::remove_file(&db_path);
            return Err(e);
        }
    };

    // 6. Populate database within error-handled boundary
    let populate_res = populate_and_verify(&mut conn, &legacy_vault, vault_path, &metadata);

    match populate_res {
        Ok(()) => Ok(conn),
        Err(e) => {
            // Drop connection and remove partially initialized vault.db (Rollback guarantee)
            drop(conn);
            let _ = fs::remove_file(&db_path);
            Err(e)
        }
    }
}

fn populate_and_verify(
    conn: &mut Connection,
    legacy: &LegacyVaultData,
    vault_path: &Path,
    metadata: &VaultMetadata,
) -> Result<(), AppError> {
    // 1. Insert Accounts
    for acc in &legacy.accounts {
        let acc_type = AccountType::from_str(&acc.acc_type)
            .map_err(|e| AppError::InvalidInput(format!("Invalid account type: {e}")))?;

        let created_at = acc
            .created_at
            .as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis())
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

        let account = Account {
            id: acc.id.clone(),
            code: acc.code.clone(),
            name: acc.name.clone(),
            account_type: acc_type,
            parent_id: acc.parent_id.clone(),
            currency: acc.currency.clone(),
            placeholder: acc.placeholder,
            hidden: acc.hidden,
            color: acc.color.clone(),
            note: acc.note.clone(),
            description: acc.description.clone(),
            interest_rate: acc.interest_rate,
            created_at,
        };

        accounts_repo::insert(conn, &account)?;
    }

    // 2. Insert Transactions
    for tx in &legacy.transactions {
        let postings: Vec<PostingInput> = tx
            .splits
            .iter()
            .map(|s| {
                let rec = s
                    .reconcile
                    .as_deref()
                    .and_then(|r| ReconcileState::from_str(r).ok());
                PostingInput {
                    id: s.id.clone(),
                    account_id: s.account_id.clone(),
                    amount: s.amount,
                    memo: s.memo.clone(),
                    action: s.action.clone(),
                    reconcile: rec,
                    ..Default::default()
                }
            })
            .collect();

        ledger_service::post_journal_entry(
            conn,
            crate::ledger::dto::CreateJournalEntryInput {
                id: Some(tx.id.clone()),
                date: tx.date.clone(),
                description: tx.description.clone(),
                notes: tx.notes.clone(),
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: Some(tx.currency.clone()),
                fx_rate: tx.fx_rate_at_transaction,
                postings,
            },
            "migration",
        )?;
    }

    // 3. Verify Integrity
    let db_accounts = accounts_repo::list_all(conn)?;
    if db_accounts.len() != legacy.accounts.len() {
        return Err(AppError::Conflict(format!(
            "Migration count mismatch: expected {} accounts, got {}",
            legacy.accounts.len(),
            db_accounts.len()
        )));
    }

    let direct_balances = accounts_repo::get_all_direct_balances(conn)?;

    // Compute expected direct balance per account from JSON splits
    let mut expected_direct: HashMap<String, i64> = HashMap::new();
    for tx in &legacy.transactions {
        for sp in &tx.splits {
            *expected_direct.entry(sp.account_id.clone()).or_insert(0) += sp.amount;
        }
    }

    for (acc_id, exp_bal) in expected_direct {
        let actual_bal = direct_balances.get(&acc_id).copied().unwrap_or(0);
        if actual_bal != exp_bal {
            return Err(AppError::Conflict(format!(
                "Balance mismatch during migration for account '{acc_id}': expected {exp_bal}, got {actual_bal}"
            )));
        }
    }

    // 4. Commit metadata
    write_metadata(vault_path, metadata)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::legacy::{scrypt_encrypt, x25519_encrypt};
    use age::secrecy::ExposeSecret;

    #[test]
    fn test_legacy_migration_roundtrip_with_verification() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let vault_path = temp_dir.path();
        let password = "TestPassword123!";

        // 1. Prepare legacy test files
        let identity = age::x25519::Identity::generate();
        let key_str = identity.to_string();
        let encrypted_key =
            scrypt_encrypt(password, key_str.expose_secret().as_bytes()).expect("encrypt key");
        fs::write(vault_path.join(KEY_FILE), encrypted_key).expect("write key");

        let legacy_json = r#"{
            "version": 2,
            "fxRate": 16000,
            "accounts": [
                {
                    "id": "acc-cash",
                    "code": "1001",
                    "name": "Cash",
                    "type": "ASSET",
                    "currency": "IDR",
                    "placeholder": false
                },
                {
                    "id": "acc-equity",
                    "code": "3001",
                    "name": "Equity",
                    "type": "EQUITY",
                    "currency": "IDR",
                    "placeholder": false
                }
            ],
            "transactions": [
                {
                    "id": "tx-1",
                    "date": "2026-09-14",
                    "description": "Initial Capital",
                    "currency": "IDR",
                    "splits": [
                        { "id": "sp-1", "accountId": "acc-cash", "amount": 1000000 },
                        { "id": "sp-2", "accountId": "acc-equity", "amount": -1000000 }
                    ]
                }
            ]
        }"#;

        let encrypted_data =
            x25519_encrypt(&identity, legacy_json.as_bytes()).expect("encrypt data");
        fs::write(vault_path.join(DATA_FILE), encrypted_data).expect("write data");

        // Verify detection
        assert!(is_legacy_vault(vault_path));

        // 2. Run migration
        let conn = migrate_legacy_vault(vault_path, password).expect("migrate legacy vault");

        // 3. Verify SQLite data is live
        let accounts = accounts_repo::list_all(&conn).expect("list accounts");
        assert_eq!(accounts.len(), 2);

        let balances = accounts_repo::get_all_direct_balances(&conn).expect("balances");
        assert_eq!(balances.get("acc-cash").copied(), Some(1000000));
        assert_eq!(balances.get("acc-equity").copied(), Some(-1000000));

        // 4. Verify backup and metadata files exist
        assert!(vault_path.join(format!("{DATA_FILE}.migrated")).exists());
        assert!(vault_path.join(crate::vault::metadata::META_FILE).exists());
        assert!(vault_path.join(DB_FILE).exists());
        assert!(!is_legacy_vault(vault_path));
    }
}

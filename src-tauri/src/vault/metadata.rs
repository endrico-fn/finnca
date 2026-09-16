use crate::crypto::envelope::WrappedKey;
use crate::crypto::kdf::{ARGON2_M_COST, ARGON2_P_COST, ARGON2_T_COST};
use crate::shared::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const META_FILE: &str = "vault.meta.json";
pub const DB_FILE: &str = "vault.db";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KdfParams {
    pub algorithm: String,
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
    pub salt: String, // Hex-encoded 16-byte salt
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultMetadata {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vault_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub kdf: KdfParams,
    pub wrapped_dek: WrappedKey,
    pub created_at: i64,
    pub updated_at: i64,
}

impl VaultMetadata {
    pub fn new(
        salt: &[u8; 16],
        wrapped_dek: WrappedKey,
        vault_name: Option<String>,
        username: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            version: 2,
            vault_name,
            username,
            kdf: KdfParams {
                algorithm: "argon2id".to_string(),
                m_cost: ARGON2_M_COST,
                t_cost: ARGON2_T_COST,
                p_cost: ARGON2_P_COST,
                salt: hex::encode(salt),
            },
            wrapped_dek,
            created_at: now,
            updated_at: now,
        }
    }
}

pub fn read_metadata(vault_path: &Path) -> Result<VaultMetadata, AppError> {
    let meta_path = vault_path.join(META_FILE);
    if !meta_path.exists() {
        return Err(AppError::NotFound(format!(
            "Metadata file not found in {}",
            vault_path.display()
        )));
    }

    let bytes = fs::read(&meta_path)?;
    let meta: VaultMetadata = serde_json::from_slice(&bytes)
        .map_err(|e| AppError::Crypto(format!("Corrupted metadata file: {e}")))?;

    Ok(meta)
}

pub fn write_metadata(vault_path: &Path, meta: &VaultMetadata) -> Result<(), AppError> {
    let meta_path = vault_path.join(META_FILE);
    let serialized = serde_json::to_vec_pretty(meta)
        .map_err(|e| AppError::Crypto(format!("Failed to serialize metadata: {e}")))?;

    crate::atomic_write(&meta_path, &serialized)
        .map_err(|e| AppError::Io(std::io::Error::other(e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_roundtrip() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let salt = [42u8; 16];
        let wrapped_dek = WrappedKey {
            nonce: "0102030405060708090a0b0c".into(),
            ciphertext: "abcdef123456".into(),
        };

        let meta = VaultMetadata::new(
            &salt,
            wrapped_dek,
            Some("Personal Finances".into()),
            Some("endrico".into()),
        );
        write_metadata(temp_dir.path(), &meta).expect("write metadata");

        let loaded = read_metadata(temp_dir.path()).expect("read metadata");
        assert_eq!(meta, loaded);
    }
}

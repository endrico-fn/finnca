#![allow(dead_code)]
use crate::shared::AppError;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WrappedKey {
    pub nonce: String,
    pub ciphertext: String,
}

pub fn generate_dek() -> Zeroizing<[u8; 32]> {
    let mut dek = Zeroizing::new([0u8; 32]);
    rand::rngs::OsRng.fill_bytes(dek.as_mut());
    dek
}

pub fn wrap_dek(kek: &[u8; 32], dek: &[u8; 32]) -> Result<WrappedKey, AppError> {
    let cipher = ChaCha20Poly1305::new(kek.into());

    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, dek.as_ref())
        .map_err(|e| AppError::Crypto(format!("Failed to wrap database key: {e}")))?;

    Ok(WrappedKey {
        nonce: hex::encode(nonce_bytes),
        ciphertext: hex::encode(ciphertext),
    })
}

pub fn unwrap_dek(kek: &[u8; 32], wrapped: &WrappedKey) -> Result<Zeroizing<[u8; 32]>, AppError> {
    let cipher = ChaCha20Poly1305::new(kek.into());

    let nonce_bytes = hex::decode(&wrapped.nonce)
        .map_err(|e| AppError::Crypto(format!("Invalid nonce format: {e}")))?;
    if nonce_bytes.len() != 12 {
        return Err(AppError::Crypto(
            "Invalid nonce length for ChaCha20-Poly1305".into(),
        ));
    }
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext_bytes = hex::decode(&wrapped.ciphertext)
        .map_err(|e| AppError::Crypto(format!("Invalid ciphertext format: {e}")))?;

    let decrypted = cipher
        .decrypt(nonce, ciphertext_bytes.as_ref())
        .map_err(|_| AppError::Crypto("Invalid password or corrupted vault key envelope".into()))?;

    if decrypted.len() != 32 {
        return Err(AppError::Crypto(
            "Decrypted database key has invalid length".into(),
        ));
    }

    let mut dek = Zeroizing::new([0u8; 32]);
    dek.copy_from_slice(&decrypted);
    Ok(dek)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_roundtrip() {
        let kek = [7u8; 32];
        let dek = generate_dek();

        let wrapped = wrap_dek(&kek, &dek).expect("wrap failed");
        let unwrapped = unwrap_dek(&kek, &wrapped).expect("unwrap failed");

        assert_eq!(dek.as_slice(), unwrapped.as_slice());
    }

    #[test]
    fn test_envelope_wrong_password_fails() {
        let kek = [7u8; 32];
        let wrong_kek = [8u8; 32];
        let dek = generate_dek();

        let wrapped = wrap_dek(&kek, &dek).expect("wrap failed");
        let result = unwrap_dek(&wrong_kek, &wrapped);

        assert!(result.is_err());
    }
}

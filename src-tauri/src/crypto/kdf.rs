use crate::shared::AppError;
use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::Zeroizing;

pub const ARGON2_M_COST: u32 = 65536; // 64 MiB
pub const ARGON2_T_COST: u32 = 3;
pub const ARGON2_P_COST: u32 = 4;

pub fn derive_kek(password: &str, salt: &[u8; 16]) -> Result<Zeroizing<[u8; 32]>, AppError> {
    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32))
        .map_err(|e| AppError::Crypto(format!("Invalid Argon2 parameters: {e}")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; 32]);

    argon2
        .hash_password_into(password.as_bytes(), salt, key.as_mut())
        .map_err(|e| AppError::Crypto(format!("Argon2 key derivation failed: {e}")))?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2id_derivation_deterministic() {
        let salt = [42u8; 16];
        let key1 = derive_kek("master_password", &salt).expect("derivation 1 failed");
        let key2 = derive_kek("master_password", &salt).expect("derivation 2 failed");
        assert_eq!(key1.as_slice(), key2.as_slice());

        let key_diff = derive_kek("other_password", &salt).expect("derivation 3 failed");
        assert_ne!(key1.as_slice(), key_diff.as_slice());
    }
}

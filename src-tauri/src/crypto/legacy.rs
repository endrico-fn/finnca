use std::io::{Read, Write};

#[allow(dead_code)]
pub fn scrypt_encrypt(password: &str, plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let recipient = age::scrypt::Recipient::new(password.into());
    let encryptor =
        age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
            .map_err(|e| format!("failed to create encryptor: {e}"))?;
    let mut out = Vec::new();
    let mut writer = encryptor
        .wrap_output(&mut out)
        .map_err(|e| format!("failed to write encryption header: {e}"))?;
    writer
        .write_all(plaintext)
        .map_err(|e| format!("failed to write plaintext: {e}"))?;
    writer
        .finish()
        .map_err(|e| format!("failed to finalize encryption: {e}"))?;
    Ok(out)
}

pub fn scrypt_decrypt(password: &str, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    let identity = age::scrypt::Identity::new(password.into());
    let decryptor = age::Decryptor::new_buffered(ciphertext)
        .map_err(|e| format!("invalid file format: {e}"))?;
    let reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(|_| "incorrect password or corrupted file".to_string())?;
    let mut out = Vec::new();
    reader
        .take(crate::MAX_PLAINTEXT_BYTES as u64)
        .read_to_end(&mut out)
        .map_err(|e| format!("failed to read plaintext: {e}"))?;
    if out.len() >= crate::MAX_PLAINTEXT_BYTES {
        return Err("vault data too large".to_string());
    }
    Ok(out)
}

#[allow(dead_code)]
pub fn x25519_encrypt(
    identity: &age::x25519::Identity,
    plaintext: &[u8],
) -> Result<Vec<u8>, String> {
    let recipient = identity.to_public();
    let encryptor =
        age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
            .map_err(|e| format!("failed to create encryptor: {e}"))?;
    let mut out = Vec::new();
    let mut writer = encryptor
        .wrap_output(&mut out)
        .map_err(|e| format!("failed to write encryption header: {e}"))?;
    writer
        .write_all(plaintext)
        .map_err(|e| format!("failed to write plaintext: {e}"))?;
    writer
        .finish()
        .map_err(|e| format!("failed to finalize encryption: {e}"))?;
    Ok(out)
}

pub fn x25519_decrypt(
    identity: &age::x25519::Identity,
    ciphertext: &[u8],
) -> Result<Vec<u8>, String> {
    let decryptor = age::Decryptor::new_buffered(ciphertext)
        .map_err(|e| format!("invalid file format: {e}"))?;
    let reader = decryptor
        .decrypt(std::iter::once(identity as &dyn age::Identity))
        .map_err(|e| format!("failed to decrypt data: {e}"))?;
    let mut out = Vec::new();
    reader
        .take(crate::MAX_PLAINTEXT_BYTES as u64)
        .read_to_end(&mut out)
        .map_err(|e| format!("failed to read plaintext: {e}"))?;
    if out.len() >= crate::MAX_PLAINTEXT_BYTES {
        return Err("vault data too large".to_string());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrypt_roundtrip() {
        let password = "test-secret-123";
        let plaintext = b"secret financial data";
        let encrypted = scrypt_encrypt(password, plaintext).expect("encryption failed");
        assert_ne!(&encrypted[..], &plaintext[..]);
        let decrypted = scrypt_decrypt(password, &encrypted).expect("decryption failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn scrypt_wrong_password_fails() {
        let password = "test-secret-123";
        let plaintext = b"secret financial data";
        let encrypted = scrypt_encrypt(password, plaintext).expect("encryption failed");
        assert!(scrypt_decrypt("wrong-password", &encrypted).is_err());
    }

    #[test]
    fn x25519_roundtrip() {
        let identity = age::x25519::Identity::generate();
        let plaintext = b"transaction record";
        let encrypted = x25519_encrypt(&identity, plaintext).expect("encryption failed");
        let decrypted = x25519_decrypt(&identity, &encrypted).expect("decryption failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn x25519_wrong_identity_fails() {
        let identity = age::x25519::Identity::generate();
        let other = age::x25519::Identity::generate();
        let encrypted = x25519_encrypt(&identity, b"secret").expect("encryption failed");
        assert!(x25519_decrypt(&other, &encrypted).is_err());
    }

    #[test]
    fn key_serialization_roundtrip() {
        use age::secrecy::ExposeSecret;

        let identity = age::x25519::Identity::generate();
        let secret = identity.to_string();
        let serialized = secret.expose_secret();
        let parsed: age::x25519::Identity = serialized.parse().expect("parse failed");
        let secret2 = parsed.to_string();
        assert_eq!(secret2.expose_secret(), serialized);
    }
}

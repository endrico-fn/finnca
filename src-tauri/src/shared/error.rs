use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database operation failed: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Cryptographic operation failed: {0}")]
    Crypto(String),

    #[error("Vault is locked or uninitialized")]
    VaultLocked,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource conflict: {0}")]
    Conflict(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "ERR_DATABASE",
            AppError::Crypto(_) => "ERR_CRYPTO",
            AppError::VaultLocked => "ERR_VAULT_LOCKED",
            AppError::InvalidInput(_) => "ERR_INVALID_INPUT",
            AppError::NotFound(_) => "ERR_NOT_FOUND",
            AppError::Conflict(_) => "ERR_CONFLICT",
            AppError::Io(_) => "ERR_IO",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

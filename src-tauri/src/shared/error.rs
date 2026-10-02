use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database operation failed")]
    Database(#[from] rusqlite::Error),

    #[error("Cryptographic operation failed")]
    Crypto(String),

    #[error("Vault is locked or uninitialized")]
    VaultLocked,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource conflict: {0}")]
    Conflict(String),

    #[error("Accounting period is locked: {0}")]
    PeriodLocked(String),

    #[error("I/O operation failed")]
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
            AppError::PeriodLocked(_) => "ERR_PERIOD_LOCKED",
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

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct AppErrorDto {
    pub code: String,
    pub message: String,
}

impl specta::Type for AppError {
    fn definition(types: &mut specta::Types) -> specta::datatype::DataType {
        <AppErrorDto as specta::Type>::definition(types)
    }
}

use crate::shared::AppError;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Session {
    pub vault_path: PathBuf,
    pub vault_name: String,
    pub db: Option<Arc<Mutex<Connection>>>,
}

pub struct AppState {
    pub session: Mutex<Option<Session>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(None),
        }
    }

    pub fn set_session(&self, session: Session) {
        match self.session.lock() {
            Ok(mut guard) => *guard = Some(session),
            Err(poisoned) => *poisoned.into_inner() = Some(session),
        }
    }

    pub fn clear_session(&self) {
        match self.session.lock() {
            Ok(mut guard) => *guard = None,
            Err(poisoned) => *poisoned.into_inner() = None,
        }
    }

    pub fn get_db(&self) -> Result<Arc<Mutex<Connection>>, AppError> {
        let guard = self.session.lock().map_err(|_| AppError::VaultLocked)?;
        if let Some(session) = guard.as_ref() {
            session.db.clone().ok_or(AppError::VaultLocked)
        } else {
            Err(AppError::VaultLocked)
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

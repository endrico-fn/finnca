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
    pub pending_import_path: Mutex<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(None),
            pending_import_path: Mutex::new(None),
        }
    }

    pub fn with_pending_import_path(path: Option<String>) -> Self {
        Self {
            session: Mutex::new(None),
            pending_import_path: Mutex::new(path),
        }
    }

    pub fn set_pending_import_path(&self, path: Option<String>) {
        match self.pending_import_path.lock() {
            Ok(mut guard) => *guard = path,
            Err(poisoned) => *poisoned.into_inner() = path,
        }
    }

    pub fn get_pending_import_path(&self) -> Option<String> {
        match self.pending_import_path.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    pub fn take_pending_import_path(&self) -> Option<String> {
        match self.pending_import_path.lock() {
            Ok(mut guard) => guard.take(),
            Err(poisoned) => poisoned.into_inner().take(),
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

    pub fn is_unlocked(&self) -> bool {
        match self.session.lock() {
            Ok(guard) => guard.is_some(),
            Err(poisoned) => poisoned.into_inner().is_some(),
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

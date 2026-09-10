use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Clone)]
pub struct Session {
    pub identity: age::x25519::Identity,
    pub vault_path: PathBuf,
    pub vault_name: String,
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
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

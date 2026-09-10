use crate::config;
use crate::state::AppState;
use crate::*;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn rename_user(
    app: AppHandle,
    state: State<'_, AppState>,
    username: String,
) -> Result<AppStateView, String> {
    let name = username.trim().to_string();
    if name.is_empty() {
        return Err("username cannot be empty".into());
    }
    if name.len() < 2 {
        return Err("username too short".into());
    }
    let mut config = config::load(&app)?;
    if config.vault.is_none() {
        return Err("vault not configured".into());
    }
    config.username = Some(name.clone());

    // Also update known_vaults to keep registry in sync
    if let Some(v) = &config.vault {
        if let Some(entry) = config.known_vaults.iter_mut().find(|k| k.path == v.path) {
            entry.username = name.clone();
        }
    }

    config::save(&app, &config)?;
    view(&app, &state)
}

#[tauri::command]
pub fn rename_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<AppStateView, String> {
    let new_name = name.trim().to_string();
    if new_name.is_empty() {
        return Err("vault name cannot be empty".into());
    }
    let mut config = config::load(&app)?;
    let vault = config.vault.as_mut().ok_or("vault not configured")?;
    vault.name = new_name.clone();

    // Also update known_vaults to keep registry in sync
    let v_path = vault.path.clone();
    if let Some(entry) = config.known_vaults.iter_mut().find(|k| k.path == v_path) {
        entry.name = new_name.clone();
    }

    config::save(&app, &config)?;

    if let Ok(mut guard) = lock_session(&state) {
        if let Some(ref mut session) = *guard {
            session.vault_name = new_name;
        }
    }

    view(&app, &state)
}

use crate::config;
use crate::state::{AppState, Session};
use crate::{view, AppStateView};
use std::path::Path;
use tauri::{AppHandle, State};

#[tauri::command]
#[specta::specta]
pub fn get_app_state(app: AppHandle, state: State<'_, AppState>) -> Result<AppStateView, String> {
    view(&app, &state)
}

#[tauri::command]
#[specta::specta]
pub async fn unlock(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
) -> Result<AppStateView, String> {
    let mut config = config::load(&app)?;
    let vault = config
        .vault
        .as_ref()
        .ok_or("vault not created — start from setup")?;
    let vault_path = Path::new(&vault.path);

    let vault_path_clone = vault_path.to_path_buf();
    let conn = tauri::async_runtime::spawn_blocking(move || {
        crate::vault::manager::open_and_unlock_vault(&vault_path_clone, &password)
    })
    .await
    .map_err(|e| format!("Unlock task failed: {e}"))?
    .map_err(|e| format!("Failed to unlock vault: {e}"))?;

    #[cfg(target_os = "linux")]
    if config.settings.auto_lock_mode == "on-reboot" {
        if let Ok(boot_id) = std::fs::read_to_string("/proc/sys/kernel/random/boot_id") {
            config.settings.boot_id = Some(boot_id.trim().to_string());
            let _ = config::save(&app, &config);
        }
    }

    state.set_session(Session {
        vault_path: vault_path.to_path_buf(),
        vault_name: vault.name.clone(),
        db: Some(std::sync::Arc::new(std::sync::Mutex::new(conn))),
    });
    crate::updater::watchdog::mark_confirmed_stable();
    view(&app, &state)
}

#[tauri::command]
#[specta::specta]
pub fn lock(app: AppHandle, state: State<'_, AppState>) -> Result<AppStateView, String> {
    state.clear_session();
    view(&app, &state)
}

#[tauri::command]
#[specta::specta]
pub async fn change_password(
    app: AppHandle,
    state: State<'_, AppState>,
    old_password: String,
    new_password: String,
) -> Result<AppStateView, String> {
    if new_password.trim().len() < 8 {
        return Err("password must be at least 8 characters".into());
    }
    let config = config::load(&app)?;
    let vault = config.vault.as_ref().ok_or("vault not configured")?;
    let vault_path = Path::new(&vault.path).to_path_buf();

    tauri::async_runtime::spawn_blocking(move || {
        crate::vault::manager::change_vault_password(&vault_path, &old_password, &new_password)
    })
    .await
    .map_err(|e| format!("Password change task failed: {e}"))?
    .map_err(|e| e.to_string())?;

    view(&app, &state)
}

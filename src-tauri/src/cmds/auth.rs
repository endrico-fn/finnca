use crate::config;
use crate::config::KEY_FILE;
use crate::state::AppState;
use crate::*;
use std::path::Path;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_app_state(app: AppHandle, state: State<'_, AppState>) -> Result<AppStateView, String> {
    view(&app, &state)
}

#[tauri::command]
pub async fn unlock(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
) -> Result<AppStateView, String> {
    let config = config::load(&app)?;
    let vault = config.vault.ok_or("vault not created — start from setup")?;
    let key_path = Path::new(&vault.path).join(KEY_FILE);
    if !key_path.exists() {
        return Err(format!("vault.key not found in {}", vault.path));
    }
    let encrypted = read_bounded(&key_path, MAX_KEY_READ_BYTES, "vault.key")?;
    if encrypted.is_empty() {
        return Err("vault.key empty — corrupted, restore from backup".into());
    }
    let serialized_bytes = scrypt_decrypt_blocking(password, encrypted).await?;
    let serialized =
        String::from_utf8(serialized_bytes).map_err(|_| "vault.key corrupted".to_string())?;
    let identity: age::x25519::Identity = serialized
        .parse()
        .map_err(|_| "vault.key corrupted".to_string())?;

    state.set_session(Session {
        identity,
        vault_path: Path::new(&vault.path).to_path_buf(),
        vault_name: vault.name.clone(),
    });
    view(&app, &state)
}

#[tauri::command]
pub fn lock(app: AppHandle, state: State<'_, AppState>) -> Result<AppStateView, String> {
    state.clear_session();
    view(&app, &state)
}

#[tauri::command]
pub fn get_boot_id() -> Result<String, String> {
    // Linux boot_id — changes on every reboot, used for "on-reboot" fast-unlock
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .map(|s| s.trim().to_string())
            .map_err(|e| format!("failed to read boot_id: {e}"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err("boot_id fast-unlock is currently only supported on Linux".to_string())
    }
}

#[tauri::command]
pub fn set_auto_lock_mode(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: String,
) -> Result<AppStateView, String> {
    match mode.as_str() {
        "always" | "on-reboot" => {}
        _ => return Err("unknown auto-lock mode".into()),
    }
    let mut config = config::load(&app)?;
    config.settings.auto_lock_mode = mode.clone();
    // When switching to on-reboot, persist current boot_id for fast-unlock comparison
    if mode == "on-reboot" {
        if let Ok(boot_id) = std::fs::read_to_string("/proc/sys/kernel/random/boot_id") {
            config.settings.boot_id = Some(boot_id.trim().to_string());
        }
    } else {
        config.settings.boot_id = None;
    }
    config::save(&app, &config)?;
    view(&app, &state)
}

#[tauri::command]
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
    let key_path = Path::new(&vault.path).join(KEY_FILE);
    if !key_path.exists() {
        return Err("vault.key not found in vault location".into());
    }

    let encrypted = read_bounded(&key_path, MAX_KEY_READ_BYTES, "vault.key")?;
    let serialized_bytes = scrypt_decrypt_blocking(old_password, encrypted)
        .await
        .map_err(|_| "Incorrect current password.".to_string())?;

    let new_encrypted = scrypt_encrypt_blocking(new_password, serialized_bytes).await?;
    atomic_write(&key_path, &new_encrypted)?;

    view(&app, &state)
}

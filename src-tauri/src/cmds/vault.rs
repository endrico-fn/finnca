use crate::config;
use crate::config::{DATA_FILE, KEY_FILE};
use crate::state::AppState;
use crate::*;
use std::fs;
use std::path::Path;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn create_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    path: String,
) -> Result<AppStateView, String> {
    if name.trim().is_empty() {
        return Err("vault name cannot be empty".into());
    }

    let raw_path = Path::new(&path);
    if !raw_path.exists() || !raw_path.is_dir() {
        return Err("storage location folder not found".into());
    }

    // Slugify vault name: e.g. "Personal Vault" -> "personal-vault"
    let slug: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    let folder_name = format!("finnca-{}", if slug.is_empty() { "vault" } else { &slug });

    // If chosen path already is named finnca-<something>, use it directly; otherwise create subfolder
    let vault_path = if raw_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with("finnca-"))
        .unwrap_or(false)
    {
        raw_path.to_path_buf()
    } else {
        raw_path.join(&folder_name)
    };

    if !vault_path.exists() {
        fs::create_dir_all(&vault_path)
            .map_err(|e| format!("failed to create vault folder: {e}"))?;
    }

    validate_vault_dir(&vault_path)?;

    let identity = age::x25519::Identity::generate();
    let session = Session {
        identity,
        vault_path,
        vault_name: name.trim().to_string(),
    };
    state.set_session(session);
    view(&app, &state)
}

#[tauri::command]
pub async fn create_account(
    app: AppHandle,
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<AppStateView, String> {
    if username.trim().is_empty() {
        return Err("username cannot be empty".into());
    }
    if password.len() < 8 {
        return Err("password must be at least 8 characters".into());
    }

    let session = lock_session(&state)?
        .take()
        .ok_or("run create_vault first")?;

    let secret = session.identity.to_string();
    let serialized = secret.expose_secret();
    let encrypted = scrypt_encrypt_blocking(password, serialized.as_bytes().to_vec()).await?;
    let key_path = session.vault_path.join(KEY_FILE);
    atomic_write(&key_path, &encrypted).map_err(|e| format!("failed to write vault.key: {e}"))?;

    let mut config = config::load(&app)?;
    config.vault = Some(config::VaultInfo {
        name: session.vault_name.clone(),
        path: session.vault_path.to_string_lossy().into_owned(),
    });
    config.username = Some(username.trim().to_string());
    config::save(&app, &config)?;

    state.set_session(session);
    view(&app, &state)
}

#[tauri::command]
pub async fn import_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> Result<AppStateView, String> {
    if password.len() < 8 {
        return Err("password must be at least 8 characters".into());
    }

    let vault_path = Path::new(&path).to_path_buf();
    if !vault_path.exists() || !vault_path.is_dir() {
        return Err("vault folder not found".into());
    }
    // Spec: import folder utuh — must contain existing vault.key (e.g. finnca-username.bak/)
    let key_path = vault_path.join(KEY_FILE);
    if !key_path.exists() {
        return Err("vault.key not found — select a folder that contains an existing finnca vault (finnca-*.bak)".into());
    }
    // Verify password can decrypt existing vault.key before touching config
    let encrypted = read_bounded(&key_path, MAX_KEY_READ_BYTES, "vault.key")?;
    if encrypted.is_empty() {
        return Err("vault.key empty — corrupted vault".into());
    }
    let serialized_bytes = scrypt_decrypt_blocking(password, encrypted)
        .await
        .map_err(|_| "incorrect password — cannot import vault".to_string())?;
    let serialized =
        String::from_utf8(serialized_bytes).map_err(|_| "vault.key corrupted".to_string())?;
    let identity: age::x25519::Identity = serialized
        .parse()
        .map_err(|_| "vault.key corrupted".to_string())?;

    // If vault.age exists, verify identity can decrypt it (integrity check)
    let data_path = vault_path.join(DATA_FILE);
    if data_path.exists() {
        if let Ok(enc) = fs::read(&data_path) {
            if !enc.is_empty() {
                let _ = crypto::x25519_decrypt(&identity, &enc).map_err(|_| {
                    "vault data cannot be decrypted with this key — corrupted vault".to_string()
                })?;
            }
        }
    }

    // Derive vault name & username from folder name if possible
    let folder_name = vault_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("vault");
    // e.g. finnca-username.bak -> username, finnca-personal-vault -> Personal Vault
    let raw_name = folder_name
        .strip_prefix("finnca-")
        .unwrap_or(folder_name)
        .trim_end_matches(".bak");
    let vault_name = if raw_name.is_empty() {
        folder_name.to_string()
    } else {
        raw_name.to_string()
    };
    // Try to preserve existing username from config if vault path matches, else use vault_name
    let mut config = config::load(&app)?;
    let username = config
        .username
        .clone()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| vault_name.clone());

    config.vault = Some(config::VaultInfo {
        name: vault_name.clone(),
        path: vault_path.to_string_lossy().into_owned(),
    });
    config.username = Some(username);
    config::save(&app, &config)?;

    state.set_session(Session {
        identity,
        vault_path,
        vault_name,
    });
    view(&app, &state)
}

fn require_session(state: &AppState) -> Result<Session, String> {
    match state.session.lock() {
        Ok(guard) => guard
            .clone()
            .ok_or_else(|| "vault locked — unlock first".to_string()),
        Err(_) => {
            state.clear_session();
            Err("session recovered after panic — unlock again".to_string())
        }
    }
}

#[tauri::command]
pub fn read_vault_data(state: State<'_, AppState>) -> Result<String, String> {
    let session = require_session(&state)?;
    let data_path = session.vault_path.join(DATA_FILE);
    if !data_path.exists() {
        return Ok("[]".into());
    }
    let encrypted = read_bounded(&data_path, MAX_VAULT_READ_BYTES, "vault.age")?;
    if encrypted.is_empty() {
        return Err("vault.age empty — corrupted, restore from vault.age.bak".into());
    }
    let plaintext = crypto::x25519_decrypt(&session.identity, &encrypted)?;
    if plaintext.len() > MAX_PLAINTEXT_BYTES {
        return Err("vault data too large".into());
    }
    String::from_utf8(plaintext).map_err(|e| format!("vault data corrupted: {e}"))
}

#[tauri::command]
pub fn write_vault_data(state: State<'_, AppState>, records: String) -> Result<(), String> {
    let session = require_session(&state)?;
    let plaintext = records.into_bytes();
    // prevent huge payloads
    if plaintext.len() > crate::MAX_PLAINTEXT_BYTES {
        return Err("vault data too large".into());
    }
    let encrypted = crypto::x25519_encrypt(&session.identity, &plaintext)?;
    let data_path = session.vault_path.join(DATA_FILE);
    let bak = data_path.with_extension("age.bak");

    // Backup previous state first for crash-safety, then atomic write unique tmp
    if data_path.exists() {
        fs::copy(&data_path, &bak)
            .map_err(|e| format!("failed to create vault.age backup: {}", e))?;
    }

    atomic_write(&data_path, &encrypted).map_err(|e| format!("failed to write vault.age: {e}"))
}

#[tauri::command]
pub async fn delete_vault_and_account(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
) -> Result<AppStateView, String> {
    let config = config::load(&app)?;
    let vault = config.vault.as_ref().ok_or("vault not found")?;
    let key_path = Path::new(&vault.path).join(KEY_FILE);
    if !key_path.exists() {
        return Err("vault.key not found in vault location".into());
    }

    // 1. Verify password by attempting to decrypt vault.key
    let encrypted = read_bounded(&key_path, MAX_KEY_READ_BYTES, "vault.key")?;
    let _ = scrypt_decrypt_blocking(password, encrypted)
        .await
        .map_err(|_| "Incorrect password — vault deletion cancelled.".to_string())?;

    // 2. Clear session in RAM
    state.clear_session();

    // 3. Remove vault files securely
    let vault_dir = Path::new(&vault.path);
    if vault_dir.exists() && vault_dir.is_dir() {
        let files_to_delete = [
            KEY_FILE,
            DATA_FILE,
            "vault.age.bak",
            "vault.age.tmp",
            "vault.key.tmp",
        ];

        for file in files_to_delete {
            let path = vault_dir.join(file);
            if path.exists() {
                fs::remove_file(&path).map_err(|e| format!("Failed to delete {}: {}", file, e))?;
            }
        }

        // Only remove the directory if it is completely empty.
        let _ = fs::remove_dir(vault_dir);
    }

    // 4. Reset config to clean unconfigured state
    let mut config = config::load(&app)?;
    config.vault = None;
    config.username = None;
    config.known_vaults.retain(|v| v.path != vault.path);
    config::save(&app, &config)?;

    view(&app, &state)
}

#[tauri::command]
pub fn open_vault_folder(app: AppHandle) -> Result<(), String> {
    let config = config::load(&app)?;
    let vault = config.vault.ok_or("vault not configured")?;
    let path = vault.path;
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open folder: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open folder: {e}"))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open folder: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub fn export_text_file(
    state: State<'_, AppState>,
    path: String,
    contents: String,
) -> Result<(), String> {
    require_session(&state)?;
    if path.trim().is_empty() {
        return Err("path cannot be empty".into());
    }
    fs::write(&path, contents.as_bytes()).map_err(|e| format!("failed to write file: {e}"))
}

#[tauri::command]
pub fn get_known_vaults_cmd(app: AppHandle) -> Result<Vec<config::VaultRegistryEntry>, String> {
    let cfg = config::load(&app)?;
    Ok(cfg.known_vaults)
}

#[tauri::command]
pub fn remember_known_vault_cmd(
    app: AppHandle,
    entry: config::VaultRegistryEntry,
) -> Result<(), String> {
    let mut cfg = config::load(&app)?;
    let idx = cfg
        .known_vaults
        .iter()
        .position(|v| v.id == entry.id || v.path == entry.path);
    if let Some(i) = idx {
        cfg.known_vaults[i] = entry;
    } else {
        cfg.known_vaults.push(entry);
    }
    config::save(&app, &cfg)
}

#[tauri::command]
pub fn forget_known_vault_cmd(app: AppHandle, id_or_path: String) -> Result<(), String> {
    let mut cfg = config::load(&app)?;
    cfg.known_vaults
        .retain(|v| v.id != id_or_path && v.path != id_or_path);
    config::save(&app, &cfg)
}

#[tauri::command]
pub fn set_active_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<AppStateView, String> {
    let mut config = config::load(&app)?;
    // find vault in known_vaults by path or legacy id
    let entry = config
        .known_vaults
        .iter()
        .find(|v| v.path == path || v.id == path)
        .ok_or("Vault not found in registry")?;
    config.vault = Some(config::VaultInfo {
        name: entry.name.clone(),
        path: entry.path.clone(),
    });
    config.username = Some(entry.username.clone());
    config::save(&app, &config)?;
    let same_vault = state.session.lock().map_or(false, |guard| {
        guard
            .as_ref()
            .is_some_and(|s| s.vault_path.to_string_lossy() == entry.path)
    });
    if !same_vault {
        state.clear_session(); // clear session to lock vault when switching
    }
    view(&app, &state)
}

#[tauri::command]
pub fn export_vault_backup_folder(
    state: State<'_, AppState>,
    dest_dir: String,
) -> Result<(), String> {
    let session = require_session(&state)?;
    let dest_path = Path::new(&dest_dir);
    if !dest_path.exists() || !dest_path.is_dir() {
        return Err("Destination is not a valid directory".into());
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup_folder_name = format!(
        "finnca-{}-backup-{}",
        session.vault_name.replace(' ', "-").to_lowercase(),
        timestamp
    );
    let target = dest_path.join(backup_folder_name);

    std::fs::create_dir_all(&target).map_err(|e| format!("Failed to create backup folder: {e}"))?;

    let src_key = session.vault_path.join(KEY_FILE);
    let dest_key = target.join(KEY_FILE);
    if src_key.exists() {
        std::fs::copy(&src_key, &dest_key).map_err(|e| format!("Failed to copy vault.key: {e}"))?;
    }

    let src_data = session.vault_path.join(DATA_FILE);
    let dest_data = target.join(DATA_FILE);
    if src_data.exists() {
        std::fs::copy(&src_data, &dest_data)
            .map_err(|e| format!("Failed to copy vault.age: {e}"))?;
    }

    Ok(())
}

pub(crate) fn validate_vault_dir(path: &Path) -> Result<(), String> {
    if !path.exists() || !path.is_dir() {
        return Err("vault folder not found".into());
    }
    let key_path = path.join(KEY_FILE);
    if key_path.exists() {
        return Err("folder already contains a finnca vault — use Import legacy vault or choose another folder".into());
    }
    std::fs::write(path.join(".finnca-test"), b"ok")
        .map_err(|e| format!("folder not writable: {e}"))?;
    let _ = std::fs::remove_file(path.join(".finnca-test"));
    Ok(())
}

#[tauri::command]
pub fn write_file_raw(
    state: State<'_, AppState>,
    path: String,
    contents: Vec<u8>,
) -> Result<(), String> {
    require_session(&state)?;
    if path.trim().is_empty() {
        return Err("path cannot be empty".into());
    }
    crate::atomic_write(Path::new(&path), &contents).map_err(|e| e.to_string())
}

use crate::config;
use crate::config::{DATA_FILE, KEY_FILE};
use crate::state::{AppState, Session};
use crate::{lock_session, view, AppStateView};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::{AppHandle, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInspectionResult {
    pub status: String,
    pub vault_name: Option<String>,
    pub username: Option<String>,
    pub is_valid: bool,
    pub message: String,
}

pub(crate) fn require_session(state: &AppState) -> Result<Session, String> {
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

pub(crate) fn validate_vault_dir(path: &Path) -> Result<(), String> {
    if !path.exists() || !path.is_dir() {
        return Err("vault folder not found".into());
    }
    if path.join(KEY_FILE).exists()
        || path.join("vault.meta.json").exists()
        || path.join("vault.db").exists()
    {
        return Err(
            "folder already contains a finnca vault — use Import vault or choose another folder"
                .into(),
        );
    }
    std::fs::write(path.join(".finnca-test"), b"ok")
        .map_err(|e| format!("folder not writable: {e}"))?;
    let _ = std::fs::remove_file(path.join(".finnca-test"));
    Ok(())
}

#[tauri::command]
pub fn inspect_vault_folder(path: String) -> Result<VaultInspectionResult, String> {
    let p = Path::new(&path);
    if !p.exists() || !p.is_dir() {
        return Ok(VaultInspectionResult {
            status: "not_found".into(),
            vault_name: None,
            username: None,
            is_valid: false,
            message: "Folder does not exist".into(),
        });
    }

    let meta_path = p.join("vault.meta.json");
    let db_path = p.join("vault.db");

    if meta_path.exists() && db_path.exists() {
        if let Ok(meta) = crate::vault::metadata::read_metadata(p) {
            let name = meta.vault_name.clone().unwrap_or_else(|| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Finnca Vault")
                    .strip_prefix("finnca-")
                    .unwrap_or("Finnca Vault")
                    .to_string()
            });
            return Ok(VaultInspectionResult {
                status: "valid_sqlite".into(),
                vault_name: Some(name),
                username: meta.username,
                is_valid: true,
                message: "Finnca SQLite Vault detected (Argon2id + SQLCipher)".into(),
            });
        }
    }

    let key_path = p.join("vault.key");
    let age_path = p.join("vault.age");
    if key_path.exists() && age_path.exists() {
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Legacy Vault")
            .strip_prefix("finnca-")
            .unwrap_or("Legacy Vault")
            .to_string();
        return Ok(VaultInspectionResult {
            status: "valid_legacy".into(),
            vault_name: Some(name),
            username: None,
            is_valid: true,
            message: "Legacy vault detected (will be auto-upgraded to SQLite)".into(),
        });
    }

    let is_empty = p
        .read_dir()
        .map(|mut i| i.next().is_none())
        .unwrap_or(false);
    if is_empty {
        return Ok(VaultInspectionResult {
            status: "empty".into(),
            vault_name: None,
            username: None,
            is_valid: false,
            message: "Selected directory is empty".into(),
        });
    }

    Ok(VaultInspectionResult {
        status: "unrecognized".into(),
        vault_name: None,
        username: None,
        is_valid: false,
        message: "Directory does not contain valid Finnca vault files".into(),
    })
}

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

    let session = Session {
        vault_path,
        vault_name: name.trim().to_string(),
        db: None,
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
    template_language: Option<String>,
) -> Result<AppStateView, String> {
    if username.trim().is_empty() {
        return Err("username cannot be empty".into());
    }
    if password.len() < 8 {
        return Err("password must be at least 8 characters".into());
    }

    let mut session = lock_session(&state)?
        .take()
        .ok_or("run create_vault first")?;

    let vault_path = session.vault_path.clone();
    let vault_name = session.vault_name.clone();
    let uname = username.trim().to_string();
    let t_lang = template_language;

    let conn = tauri::async_runtime::spawn_blocking(move || {
        crate::vault::manager::create_new_vault(
            &vault_path,
            &password,
            Some(&vault_name),
            Some(&uname),
            t_lang.as_deref(),
        )
    })
    .await
    .map_err(|e| format!("Create vault task failed: {e}"))?
    .map_err(|e| format!("Failed to create vault: {e}"))?;

    session.db = Some(std::sync::Arc::new(std::sync::Mutex::new(conn)));

    let mut config = config::load(&app)?;
    config.vault = Some(config::VaultInfo {
        name: session.vault_name.clone(),
        path: session.vault_path.to_string_lossy().into_owned(),
    });
    config.username = Some(username.trim().to_string());

    let v_path = session.vault_path.to_string_lossy().into_owned();
    if let Some(entry) = config.known_vaults.iter_mut().find(|k| k.path == v_path) {
        entry.name = session.vault_name.clone();
        entry.username = username.trim().to_string();
    } else {
        config.known_vaults.push(config::VaultRegistryEntry {
            id: v_path.clone(),
            name: session.vault_name.clone(),
            path: v_path,
            username: username.trim().to_string(),
            last_opened_at: Some(chrono::Utc::now().to_rfc3339()),
        });
    }
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

    let vault_path_clone = vault_path.clone();
    let conn = tauri::async_runtime::spawn_blocking(move || {
        crate::vault::manager::open_and_unlock_vault(&vault_path_clone, &password)
    })
    .await
    .map_err(|e| format!("Unlock task failed: {e}"))?
    .map_err(|e| format!("incorrect password or invalid vault: {e}"))?;

    let folder_name = vault_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("vault");

    let base_name = folder_name.strip_prefix("finnca-").unwrap_or(folder_name);
    let base_name = base_name.strip_suffix(".bak").unwrap_or(base_name);
    let fallback_vault_name = base_name
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let (vault_name, username) =
        if let Ok(meta) = crate::vault::metadata::read_metadata(&vault_path) {
            let name = meta.vault_name.unwrap_or(fallback_vault_name);
            let user = meta.username.unwrap_or_else(|| name.clone());
            (name, user)
        } else {
            let user = fallback_vault_name.clone();
            (fallback_vault_name, user)
        };

    let mut config = config::load(&app)?;
    config.vault = Some(config::VaultInfo {
        name: vault_name.clone(),
        path: vault_path.to_string_lossy().into_owned(),
    });
    config.username = Some(username.clone());

    let v_path = vault_path.to_string_lossy().into_owned();
    if let Some(entry) = config.known_vaults.iter_mut().find(|k| k.path == v_path) {
        entry.name = vault_name.clone();
        entry.username = username.clone();
    } else {
        config.known_vaults.push(config::VaultRegistryEntry {
            id: v_path.clone(),
            name: vault_name.clone(),
            path: v_path,
            username: username.clone(),
            last_opened_at: Some(chrono::Utc::now().to_rfc3339()),
        });
    }
    config::save(&app, &config)?;

    state.set_session(Session {
        vault_path,
        vault_name,
        db: Some(std::sync::Arc::new(std::sync::Mutex::new(conn))),
    });

    view(&app, &state)
}

#[tauri::command]
pub async fn delete_vault_and_account(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
) -> Result<AppStateView, String> {
    let config = config::load(&app)?;
    let vault = config.vault.as_ref().ok_or("vault not found")?;
    let vault_path = Path::new(&vault.path).to_path_buf();

    let vault_path_clone = vault_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::vault::manager::open_and_unlock_vault(&vault_path_clone, &password)
    })
    .await
    .map_err(|e| format!("Unlock task failed: {e}"))?
    .map_err(|_| "Incorrect password — vault deletion cancelled.".to_string())?;

    state.clear_session();

    if vault_path.exists() && vault_path.is_dir() {
        let files_to_delete = [
            KEY_FILE,
            DATA_FILE,
            "vault.age.bak",
            "vault.age.tmp",
            "vault.key.tmp",
            "vault.db",
            "vault.db-shm",
            "vault.db-wal",
            "vault.meta.json",
            "vault.meta.json.tmp",
        ];

        for file in files_to_delete {
            let path = vault_path.join(file);
            if path.exists() {
                let _ = fs::remove_file(&path);
            }
        }

        let _ = fs::remove_dir(vault_path);
    }

    let mut config = config::load(&app)?;
    config.known_vaults.retain(|v| v.path != vault.path);

    if let Some(next_vault) = config.known_vaults.first() {
        config.vault = Some(config::VaultInfo {
            name: next_vault.name.clone(),
            path: next_vault.path.clone(),
        });
        config.username = Some(next_vault.username.clone());
    } else {
        config.vault = None;
        config.username = None;
    }
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
    let same_vault = state.session.lock().is_ok_and(|guard| {
        guard
            .as_ref()
            .is_some_and(|s| s.vault_path.to_string_lossy() == entry.path)
    });
    if !same_vault {
        state.clear_session();
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

    // Checkpoint SQLite WAL if active connection exists
    if let Some(ref db_mutex) = session.db {
        if let Ok(conn) = db_mutex.lock() {
            let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        }
    }

    // 1. Copy SQLite database & metadata
    let src_meta = session.vault_path.join("vault.meta.json");
    let dest_meta = target.join("vault.meta.json");
    if src_meta.exists() {
        std::fs::copy(&src_meta, &dest_meta)
            .map_err(|e| format!("Failed to copy vault.meta.json: {e}"))?;
    }

    let src_db = session.vault_path.join("vault.db");
    let dest_db = target.join("vault.db");
    if src_db.exists() {
        std::fs::copy(&src_db, &dest_db).map_err(|e| format!("Failed to copy vault.db: {e}"))?;
    }

    // 2. Legacy fallback files if present
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

    if let Some(v) = &config.vault {
        let v_path = Path::new(&v.path);
        if let Ok(mut meta) = crate::vault::metadata::read_metadata(v_path) {
            meta.username = Some(name.clone());
            let _ = crate::vault::metadata::write_metadata(v_path, &meta);
        }
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

    let v_path_str = vault.path.clone();
    let v_path = Path::new(&v_path_str);
    if let Ok(mut meta) = crate::vault::metadata::read_metadata(v_path) {
        meta.vault_name = Some(new_name.clone());
        let _ = crate::vault::metadata::write_metadata(v_path, &meta);
    }

    if let Some(entry) = config
        .known_vaults
        .iter_mut()
        .find(|k| k.path == v_path_str)
    {
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

use crate::config;
use crate::config::{DATA_FILE, KEY_FILE};
use crate::state::{AppState, Session};
use crate::{lock_session, view, AppStateView};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};
use tauri::{AppHandle, State};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
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

pub(crate) fn validate_safe_export_target(
    path_str: &str,
    session: &Session,
    content_len: usize,
) -> Result<PathBuf, String> {
    if content_len > crate::MAX_PLAINTEXT_BYTES {
        return Err(format!(
            "Payload size ({} bytes) exceeds safety limit ({} bytes)",
            content_len,
            crate::MAX_PLAINTEXT_BYTES
        ));
    }

    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return Err("Path cannot be empty".into());
    }
    if trimmed.contains('\0') {
        return Err("Path contains invalid null byte".into());
    }

    let path = Path::new(trimmed);
    if !path.is_absolute() {
        return Err("Target path must be an absolute path".into());
    }

    for comp in path.components() {
        match comp {
            Component::ParentDir => return Err("Path traversal '..' is strictly forbidden".into()),
            Component::Normal(os_str) => {
                let name = os_str.to_string_lossy();
                if name.starts_with('.')
                    && (name.contains("ssh")
                        || name.contains("gnupg")
                        || name.contains("bash")
                        || name.contains("profile")
                        || name.contains("zsh")
                        || name.contains("git")
                        || name.contains("aws")
                        || name.contains("env")
                        || name.contains("config"))
                {
                    return Err(format!(
                        "Writing to sensitive dotfile/directory '{name}' is forbidden"
                    ));
                }
            }
            _ => {}
        }
    }

    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
        let lower = file_name.to_lowercase();
        if lower.starts_with("vault.")
            || lower.starts_with(".finnca")
            || lower == KEY_FILE
            || lower == DATA_FILE
        {
            return Err(format!(
                "Direct write to protected vault file '{file_name}' is forbidden"
            ));
        }

        let dangerous_exts = [
            "exe", "bat", "cmd", "sh", "bash", "bin", "elf", "so", "dll", "dylib", "com", "scr",
            "msi", "vbs", "ps1", "service",
        ];
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if dangerous_exts.contains(&ext.to_lowercase().as_str()) {
                return Err(format!(
                    "Writing to executable file extension '.{ext}' is forbidden"
                ));
            }
        }
    } else {
        return Err("Invalid file name".into());
    }

    // Check if target is a symlink
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            return Err("Writing through symlinks is strictly forbidden".into());
        }
    }

    let parent = path.parent().ok_or("Target path has no parent directory")?;
    if !parent.exists() || !parent.is_dir() {
        return Err("Target parent directory does not exist or is not a directory".into());
    }

    let canonical_parent = parent
        .canonicalize()
        .map_err(|e| format!("Cannot resolve parent directory: {e}"))?;
    let parent_str = canonical_parent.to_string_lossy();

    #[cfg(unix)]
    {
        if parent_str == "/" {
            return Err("Writing directly to root directory '/' is forbidden".into());
        }
        let forbidden_roots = [
            "/etc", "/bin", "/sbin", "/usr", "/boot", "/dev", "/proc", "/sys", "/lib", "/lib64",
            "/root",
        ];
        for f in &forbidden_roots {
            if parent_str == *f || parent_str.starts_with(&format!("{f}/")) {
                return Err(format!("Writing into system directory '{f}' is forbidden"));
            }
        }
    }

    #[cfg(windows)]
    {
        let lower_parent = parent_str.to_lowercase();
        if lower_parent == "c:\\"
            || lower_parent.starts_with("c:\\windows")
            || lower_parent.starts_with("c:\\program files")
        {
            return Err("Writing into system directories is forbidden".into());
        }
    }

    // Check active vault folder integrity
    if let Ok(canonical_vault) = session.vault_path.canonicalize() {
        if canonical_parent == canonical_vault || canonical_parent.starts_with(&canonical_vault) {
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                let lower = file_name.to_lowercase();
                if lower.starts_with("vault.")
                    || lower.starts_with(".finnca")
                    || lower == KEY_FILE
                    || lower == DATA_FILE
                {
                    return Err(format!(
                        "Direct write to protected vault file '{file_name}' inside vault directory is forbidden"
                    ));
                }
            }
        }
    }

    Ok(path.to_path_buf())
}

#[tauri::command]
#[specta::specta]
pub fn inspect_vault_folder(path: String) -> Result<VaultInspectionResult, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Ok(VaultInspectionResult {
            status: "not_found".into(),
            vault_name: None,
            username: None,
            is_valid: false,
            message: "Path does not exist".into(),
        });
    }

    if p.is_file() {
        let is_finnca = p
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("finnca"))
            .unwrap_or(false);

        if is_finnca {
            let name = p
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("Finnca Archive")
                .strip_prefix("finnca-")
                .unwrap_or("Finnca Archive")
                .to_string();
            return Ok(VaultInspectionResult {
                status: "valid_archive".into(),
                vault_name: Some(name),
                username: None,
                is_valid: true,
                message: "Finnca Vault Archive (.finnca) detected".into(),
            });
        }

        return Ok(VaultInspectionResult {
            status: "invalid_file".into(),
            vault_name: None,
            username: None,
            is_valid: false,
            message: "Target is a file, not a vault directory or .finnca archive".into(),
        });
    }

    if !p.is_dir() {
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
#[specta::specta]
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
#[specta::specta]
pub async fn create_account(
    app: AppHandle,
    state: State<'_, AppState>,
    username: String,
    password: String,
    template_language: Option<String>,
    account_profile: Option<String>,
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
    let acc_prof = account_profile;

    let conn = tauri::async_runtime::spawn_blocking(move || {
        crate::vault::manager::create_new_vault(
            &vault_path,
            &password,
            Some(&vault_name),
            Some(&uname),
            t_lang.as_deref(),
            acc_prof.as_deref(),
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

    #[cfg(target_os = "linux")]
    if config.settings.auto_lock_mode == "on-reboot" {
        if let Ok(boot_id) = std::fs::read_to_string("/proc/sys/kernel/random/boot_id") {
            config.settings.boot_id = Some(boot_id.trim().to_string());
        }
    }

    config::save(&app, &config)?;

    state.set_session(session);

    view(&app, &state)
}

pub(crate) fn unpack_finnca_archive(
    app: &AppHandle,
    archive_path: &Path,
) -> Result<PathBuf, String> {
    let stem = archive_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("vault");
    let slug = stem.strip_prefix("finnca-").unwrap_or(stem);
    let folder_name = format!("finnca-{}", if slug.is_empty() { "vault" } else { slug });

    let mut target_dir = archive_path
        .parent()
        .map(|p| p.join(&folder_name))
        .unwrap_or_else(|| PathBuf::from(&folder_name));

    if fs::create_dir_all(&target_dir).is_err() {
        use tauri::Manager;
        let config_dir = app
            .path()
            .app_config_dir()
            .map_err(|e| format!("Failed to resolve app config dir: {e}"))?;
        target_dir = config_dir.join(&folder_name);
        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create vault destination directory: {e}"))?;
    }

    let file =
        fs::File::open(archive_path).map_err(|e| format!("Failed to open archive file: {e}"))?;

    let is_zip = if let Ok(mut zip) = zip::ZipArchive::new(file) {
        for i in 0..zip.len() {
            let mut entry = zip
                .by_index(i)
                .map_err(|e| format!("Corrupt zip archive entry: {e}"))?;
            let enclosed_name = match entry.enclosed_name() {
                Some(p) => p.to_owned(),
                None => continue,
            };
            let outpath = target_dir.join(&enclosed_name);
            if entry.is_dir() {
                fs::create_dir_all(&outpath)
                    .map_err(|e| format!("Failed to create folder: {e}"))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)
                            .map_err(|e| format!("Failed to create parent directory: {e}"))?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)
                    .map_err(|e| format!("Failed to create file {}: {e}", outpath.display()))?;
                std::io::copy(&mut entry, &mut outfile)
                    .map_err(|e| format!("Failed to extract {}: {e}", outpath.display()))?;
            }
        }
        true
    } else {
        false
    };

    if !is_zip {
        return Err(
            "Invalid archive: file is not a valid .finnca archive (must be a zip container)".into(),
        );
    }

    if target_dir.join("vault.db").exists() || target_dir.join(KEY_FILE).exists() {
        return Ok(target_dir);
    }

    if let Ok(entries) = fs::read_dir(&target_dir) {
        let subfolders: Vec<_> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        if subfolders.len() == 1
            && (subfolders[0].join("vault.db").exists() || subfolders[0].join(KEY_FILE).exists())
        {
            return Ok(subfolders[0].clone());
        }
    }

    Err("Archive does not contain valid Finnca vault data (vault.db)".into())
}

#[tauri::command]
#[specta::specta]
pub async fn import_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> Result<AppStateView, String> {
    if password.len() < 8 {
        return Err("password must be at least 8 characters".into());
    }

    let raw_path = Path::new(&path);
    if !raw_path.exists() {
        return Err("vault path not found".into());
    }

    let vault_path = if raw_path.is_file() {
        let is_finnca = raw_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("finnca") || ext.eq_ignore_ascii_case("zip"))
            .unwrap_or(false);

        if !is_finnca {
            return Err("Target file is not a .finnca vault archive".into());
        }

        unpack_finnca_archive(&app, raw_path)?
    } else if raw_path.is_dir() {
        raw_path.to_path_buf()
    } else {
        return Err("vault path is neither a file nor a directory".into());
    };

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
#[specta::specta]
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
#[specta::specta]
pub fn open_vault_folder(app: AppHandle) -> Result<(), String> {
    let config = config::load(&app)?;
    let vault = config.vault.ok_or("vault not configured")?;
    let path = vault.path;
    let p = Path::new(&path);
    if !p.exists() || !p.is_dir() {
        return Err("Vault directory does not exist or is not a directory".into());
    }
    let canonical = p
        .canonicalize()
        .map_err(|e| format!("Invalid vault path: {e}"))?;
    let canonical_str = canonical.to_string_lossy();

    if canonical_str.starts_with('-') {
        return Err("Invalid vault directory path".into());
    }

    #[cfg(unix)]
    {
        let forbidden = [
            "/etc", "/root", "/boot", "/sys", "/proc", "/bin", "/sbin", "/usr", "/dev",
        ];
        if canonical_str == "/"
            || forbidden
                .iter()
                .any(|f| canonical_str == *f || canonical_str.starts_with(&format!("{f}/")))
        {
            return Err("Cannot open system root folder".into());
        }
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg("--")
            .arg(&*canonical_str)
            .spawn()
            .map_err(|e| format!("failed to open folder: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("--")
            .arg(&*canonical_str)
            .spawn()
            .map_err(|e| format!("failed to open folder: {e}"))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&*canonical_str)
            .spawn()
            .map_err(|e| format!("failed to open folder: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn export_text_file(
    state: State<'_, AppState>,
    path: String,
    contents: String,
) -> Result<(), String> {
    let session = require_session(&state)?;
    let safe_path = validate_safe_export_target(&path, &session, contents.len())?;
    crate::atomic_write(&safe_path, contents.as_bytes())
        .map_err(|e| format!("failed to write file: {e}"))
}

#[tauri::command]
#[specta::specta]
pub fn get_known_vaults_cmd(app: AppHandle) -> Result<Vec<config::VaultRegistryEntry>, String> {
    let cfg = config::load(&app)?;
    Ok(cfg.known_vaults)
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
pub fn forget_known_vault_cmd(app: AppHandle, id_or_path: String) -> Result<(), String> {
    let mut cfg = config::load(&app)?;
    cfg.known_vaults
        .retain(|v| v.id != id_or_path && v.path != id_or_path);
    config::save(&app, &cfg)
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
pub fn export_vault_backup_folder(
    state: State<'_, AppState>,
    dest_dir: String,
) -> Result<(), String> {
    let session = require_session(&state)?;
    let dest_path = Path::new(&dest_dir);
    if !dest_path.exists() || !dest_path.is_dir() {
        return Err("Destination is not a valid directory".into());
    }

    let canonical_dest = dest_path
        .canonicalize()
        .map_err(|e| format!("Invalid destination directory: {e}"))?;
    #[cfg(unix)]
    {
        let dest_str = canonical_dest.to_string_lossy();
        let forbidden = [
            "/etc", "/root", "/boot", "/sys", "/proc", "/bin", "/sbin", "/usr", "/dev",
        ];
        if dest_str == "/"
            || forbidden
                .iter()
                .any(|f| dest_str == *f || dest_str.starts_with(&format!("{f}/")))
        {
            return Err("Exporting backup into system directories is forbidden".into());
        }
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
    let target = dest_path.join(&backup_folder_name);

    std::fs::create_dir_all(&target).map_err(|e| format!("Failed to create backup folder: {e}"))?;

    // Checkpoint SQLite WAL and hold lock throughout copy to guarantee atomic snapshot
    let _lock_guard = if let Some(ref db_mutex) = session.db {
        let conn = db_mutex
            .lock()
            .map_err(|_| "Database is locked by another operation".to_string())?;
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| format!("WAL checkpoint failed before backup: {e}"))?;
        Some(conn)
    } else {
        None
    };

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

    // 3. Create .finnca portable archive in destination directory
    let archive_path = dest_path.join(format!("{backup_folder_name}.finnca"));
    if let Ok(archive_file) = std::fs::File::create(&archive_path) {
        let mut zip = zip::ZipWriter::new(archive_file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        if dest_meta.exists() {
            if let Ok(data) = std::fs::read(&dest_meta) {
                let _ = zip.start_file("vault.meta.json", options);
                let _ = std::io::Write::write_all(&mut zip, &data);
            }
        }
        if dest_db.exists() {
            if let Ok(data) = std::fs::read(&dest_db) {
                let _ = zip.start_file("vault.db", options);
                let _ = std::io::Write::write_all(&mut zip, &data);
            }
        }
        if dest_key.exists() {
            if let Ok(data) = std::fs::read(&dest_key) {
                let _ = zip.start_file(KEY_FILE, options);
                let _ = std::io::Write::write_all(&mut zip, &data);
            }
        }
        if dest_data.exists() {
            if let Ok(data) = std::fs::read(&dest_data) {
                let _ = zip.start_file(DATA_FILE, options);
                let _ = std::io::Write::write_all(&mut zip, &data);
            }
        }
        let _ = zip.finish();
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn write_file_raw(
    state: State<'_, AppState>,
    path: String,
    contents: Vec<u8>,
) -> Result<(), String> {
    let session = require_session(&state)?;
    let safe_path = validate_safe_export_target(&path, &session, contents.len())?;
    crate::atomic_write(&safe_path, &contents).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
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

#[tauri::command]
#[specta::specta]
pub fn get_pending_import_path(state: State<'_, AppState>) -> Option<String> {
    state.get_pending_import_path()
}

#[tauri::command]
#[specta::specta]
pub fn clear_pending_import_path(state: State<'_, AppState>) {
    state.set_pending_import_path(None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_safe_export_target_rejects_relative_and_traversal() {
        let temp = tempfile::tempdir().unwrap();
        let session = Session {
            vault_path: temp.path().to_path_buf(),
            vault_name: "test".into(),
            db: None,
        };

        // Relative path rejected
        assert!(validate_safe_export_target("export.json", &session, 10).is_err());

        // Path traversal rejected
        let traversal = temp.path().join("../../../etc/passwd");
        assert!(validate_safe_export_target(&traversal.to_string_lossy(), &session, 10).is_err());

        // Dangerous extensions rejected
        let dangerous = temp.path().join("script.sh");
        assert!(validate_safe_export_target(&dangerous.to_string_lossy(), &session, 10).is_err());

        // Protected vault files rejected
        let protected = temp.path().join("vault.db");
        assert!(validate_safe_export_target(&protected.to_string_lossy(), &session, 10).is_err());

        let protected_key = temp.path().join("vault.key");
        assert!(
            validate_safe_export_target(&protected_key.to_string_lossy(), &session, 10).is_err()
        );

        // Null bytes rejected
        let null_byte = format!("{}/test\0.json", temp.path().display());
        assert!(validate_safe_export_target(&null_byte, &session, 10).is_err());

        // Symlink rejected (on unix)
        #[cfg(unix)]
        {
            let target_file = temp.path().join("real_target.json");
            std::fs::write(&target_file, b"content").unwrap();
            let symlink_file = temp.path().join("link_target.json");
            std::os::unix::fs::symlink(&target_file, &symlink_file).unwrap();
            assert!(
                validate_safe_export_target(&symlink_file.to_string_lossy(), &session, 10).is_err()
            );
        }

        // Valid file accepted
        let valid = temp.path().join("backup.json");
        assert!(validate_safe_export_target(&valid.to_string_lossy(), &session, 10).is_ok());
    }

    #[test]
    fn test_inspect_vault_folder_archive() {
        let temp = tempfile::tempdir().unwrap();

        // Nonexistent path
        let non_existent = temp.path().join("does_not_exist.finnca");
        let res = inspect_vault_folder(non_existent.to_string_lossy().into()).unwrap();
        assert!(!res.is_valid);
        assert_eq!(res.status, "not_found");

        // Non-finnca file
        let txt_file = temp.path().join("note.txt");
        std::fs::write(&txt_file, b"hello").unwrap();
        let res_txt = inspect_vault_folder(txt_file.to_string_lossy().into()).unwrap();
        assert!(!res_txt.is_valid);
        assert_eq!(res_txt.status, "invalid_file");

        // .finnca archive file
        let finnca_file = temp.path().join("finnca-personal-finance.finnca");
        std::fs::write(&finnca_file, b"PK\x03\x04archive").unwrap();
        let res_finnca = inspect_vault_folder(finnca_file.to_string_lossy().into()).unwrap();
        assert!(res_finnca.is_valid);
        assert_eq!(res_finnca.status, "valid_archive");
        assert_eq!(res_finnca.vault_name, Some("personal-finance".to_string()));
    }
}

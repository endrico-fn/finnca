use crate::config::KEY_FILE;
use tauri::AppHandle;
pub mod cmds;
mod config;
mod crypto;
mod state;

use std::fs;
use std::path::Path;

use age::secrecy::ExposeSecret;
use serde::Serialize;
use state::{AppState, Session};

const MAX_VAULT_READ_BYTES: u64 = 15 * 1024 * 1024;
const MAX_KEY_READ_BYTES: u64 = 1024 * 1024;
pub(crate) const MAX_PLAINTEXT_BYTES: usize = 11 * 1024 * 1024;

pub(crate) fn read_bounded(path: &Path, max: u64, label: &str) -> Result<Vec<u8>, String> {
    let meta = fs::metadata(path).map_err(|e| format!("failed to read {label}: {e}"))?;
    if meta.len() > max {
        return Err(format!(
            "{label} too large ({:.1}MB max)",
            max as f64 / 1048576.0
        ));
    }
    let bytes = fs::read(path).map_err(|e| format!("failed to read {label}: {e}"))?;
    if bytes.len() as u64 > max {
        return Err(format!(
            "{label} too large ({:.1}MB max)",
            max as f64 / 1048576.0
        ));
    }
    Ok(bytes)
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension(format!(
        "{}.tmp-{}-{}",
        path.extension().and_then(|s| s.to_str()).unwrap_or("tmp"),
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    {
        use std::io::Write;
        let mut file =
            fs::File::create(&tmp).map_err(|e| format!("failed to create temp file: {e}"))?;
        file.write_all(bytes)
            .map_err(|e| format!("failed to write temp file: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("failed to sync temp file: {e}"))?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("failed to write file: {e}")
    })?;

    #[cfg(unix)]
    if let Some(parent) = path.parent() {
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
    }

    Ok(())
}

pub(crate) async fn scrypt_encrypt_blocking(
    password: String,
    plaintext: Vec<u8>,
) -> Result<Vec<u8>, String> {
    tauri::async_runtime::spawn_blocking(move || crypto::scrypt_encrypt(&password, &plaintext))
        .await
        .map_err(|e| format!("encryption task failed: {e}"))?
}

pub(crate) async fn scrypt_decrypt_blocking(
    password: String,
    ciphertext: Vec<u8>,
) -> Result<Vec<u8>, String> {
    tauri::async_runtime::spawn_blocking(move || crypto::scrypt_decrypt(&password, &ciphertext))
        .await
        .map_err(|e| format!("decryption task failed: {e}"))?
}

pub(crate) fn lock_session(
    state: &AppState,
) -> Result<std::sync::MutexGuard<'_, Option<Session>>, String> {
    match state.session.lock() {
        Ok(guard) => Ok(guard),
        Err(_) => {
            state.clear_session();
            Err("session recovered after panic — please retry".to_string())
        }
    }
}

#[derive(Serialize)]
pub struct AppStateView {
    configured: bool,
    unlocked: bool,
    username: Option<String>,
    vault_name: Option<String>,
    vault_path: Option<String>,
    settings: config::Settings,
}

pub(crate) fn view(app: &AppHandle, state: &AppState) -> Result<AppStateView, String> {
    let mut config = config::load(app)?;
    let mut configured = config.vault.is_some() && config.username.is_some();

    // Auto-detect if user deleted the vault folder manually on disk
    if let Some(vault) = &config.vault {
        let key_path = Path::new(&vault.path).join(KEY_FILE);
        if !key_path.exists() {
            configured = false;
            config.vault = None;
            config.username = None;
            let _ = config::save(app, &config);
            state.clear_session();
        }
    }

    let unlocked = lock_session(state)?.is_some();
    Ok(AppStateView {
        configured,
        unlocked,
        username: config.username,
        vault_name: config.vault.as_ref().map(|v| v.name.clone()),
        vault_path: config.vault.as_ref().map(|v| v.path.clone()),
        settings: config.settings,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            cmds::auth::get_app_state,
            cmds::auth::unlock,
            cmds::auth::lock,
            cmds::auth::get_boot_id,
            cmds::auth::set_auto_lock_mode,
            cmds::auth::change_password,
            cmds::vault::create_vault,
            cmds::vault::create_account,
            cmds::vault::import_vault,
            cmds::vault::read_vault_data,
            cmds::vault::write_vault_data,
            cmds::vault::delete_vault_and_account,
            cmds::vault::open_vault_folder,
            cmds::vault::export_text_file,
            cmds::vault::get_known_vaults_cmd,
            cmds::vault::remember_known_vault_cmd,
            cmds::vault::forget_known_vault_cmd,
            cmds::vault::set_active_vault,
            cmds::vault::export_vault_backup_folder,
            cmds::user::rename_user,
            cmds::user::rename_vault,
            cmds::vault::write_file_raw
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

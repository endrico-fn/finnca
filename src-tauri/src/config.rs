use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub auto_lock_mode: String,
    /// Linux boot_id at time mode was set to on-reboot — for fast-unlock comparison
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot_id: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_lock_mode: "always".into(),
            boot_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultRegistryEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub username: String,
    #[serde(default)]
    pub last_opened_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub vault: Option<VaultInfo>,
    pub username: Option<String>,
    pub settings: Settings,
    #[serde(default)]
    pub known_vaults: Vec<VaultRegistryEntry>,
}

pub fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    // Fixed flat path: ~/.config/finnca/finnca.json (not com.finnca.app)
    let base = app
        .path()
        .config_dir()
        .map_err(|_| "Failed to resolve system config directory".to_string())?;
    Ok(base.join("finnca").join("finnca.json"))
}

fn legacy_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|p| p.join("finnca.json"))
}

pub fn load(app: &AppHandle) -> Result<AppConfig, String> {
    let path = config_path(app)?;
    if !path.exists() {
        // migrate from legacy com.finnca.app path if present
        if let Some(legacy) = legacy_path(app) {
            if legacy.exists() {
                if let Ok(raw) = std::fs::read_to_string(&legacy) {
                    if let Ok(cfg) = serde_json::from_str::<AppConfig>(&raw) {
                        // lazily migrate to new location
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let _ = std::fs::write(&path, &raw);
                        return Ok(cfg);
                    }
                }
            }
        }
        return Ok(AppConfig::default());
    }
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("failed to read config: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("config corrupted: {e}"))
}

pub fn save(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("failed to create config directory: {e}"))?;
    }
    let raw = serde_json::to_string_pretty(config)
        .map_err(|e| format!("failed to serialize config: {e}"))?;
    crate::atomic_write(&path, raw.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

pub const KEY_FILE: &str = "vault.key";
pub const DATA_FILE: &str = "vault.age";

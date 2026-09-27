use crate::config;
use crate::state::AppState;
use crate::{view, AppStateView};
use tauri::{AppHandle, State};

#[tauri::command]
#[specta::specta]
pub fn get_boot_id() -> Result<String, String> {
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
#[specta::specta]
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

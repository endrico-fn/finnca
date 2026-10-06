#[tauri::command]
#[specta::specta]
pub fn arm_update_watchdog_cmd(target_version: String) -> Result<(), String> {
    let watchdog = crate::updater::default_watchdog();
    watchdog.arm_watchdog(&target_version)
}

#[tauri::command]
#[specta::specta]
pub fn disarm_update_watchdog_cmd() -> Result<(), String> {
    let watchdog = crate::updater::default_watchdog();
    watchdog.handle_clean_exit();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_rollback_notice_cmd() -> Result<Option<String>, String> {
    let watchdog = crate::updater::default_watchdog();
    watchdog.get_and_clear_rollback_notice()
}

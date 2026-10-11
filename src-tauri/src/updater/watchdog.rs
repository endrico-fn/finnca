use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const BOOT_STATE_FILE: &str = "update_pending.json";
pub const LEGACY_BOOT_STATE_FILE: &str = ".boot_state";
pub const ROLLBACK_NOTICE_FILE: &str = "rollback_notice.json";
pub const MAX_CRASH_ATTEMPTS: u32 = 2;
pub const STABILITY_THRESHOLD_SECS: u64 = 15;

static GUARD_ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BootState {
    pub target_version: String,
    pub timestamp_unix: u64,
    pub attempts: u32,
    #[serde(default)]
    pub confirmed_stable: bool,
}

pub struct UpdateWatchdog {
    pub config_dir: PathBuf,
    pub current_exe: PathBuf,
    pub stability_threshold_secs: u64,
}

impl UpdateWatchdog {
    pub fn new(config_dir: PathBuf, current_exe: PathBuf) -> Self {
        Self {
            config_dir,
            current_exe,
            stability_threshold_secs: STABILITY_THRESHOLD_SECS,
        }
    }

    pub fn with_threshold(
        config_dir: PathBuf,
        current_exe: PathBuf,
        stability_threshold_secs: u64,
    ) -> Self {
        Self {
            config_dir,
            current_exe,
            stability_threshold_secs,
        }
    }

    /// Resolves actual writable binary path.
    /// On Linux AppImage, the target is the outer .AppImage file specified by $APPIMAGE,
    /// rather than the read-only squashfs mount at /tmp/.mount_XXXX/.
    pub fn resolve_target_binary(&self) -> PathBuf {
        #[cfg(target_os = "linux")]
        {
            if let Ok(appimage_path) = std::env::var("APPIMAGE") {
                let p = PathBuf::from(appimage_path);
                if p.exists() {
                    return p;
                }
            }
        }
        self.current_exe.clone()
    }

    pub fn state_path(&self) -> PathBuf {
        let primary = self.config_dir.join(BOOT_STATE_FILE);
        if !primary.exists() {
            let legacy = self.config_dir.join(LEGACY_BOOT_STATE_FILE);
            if legacy.exists() {
                return legacy;
            }
        }
        primary
    }

    pub fn backup_exe_path(&self) -> PathBuf {
        let target = self.resolve_target_binary();
        let mut backup = target.clone();
        backup.set_extension("backup");
        backup
    }

    /// Arm the watchdog before replacing executable during an update.
    pub fn arm_watchdog(&self, target_version: &str) -> Result<(), String> {
        let target_exe = self.resolve_target_binary();
        let backup = self.backup_exe_path();

        if target_exe.exists() {
            fs::copy(&target_exe, &backup)
                .map_err(|e| format!("Failed to create binary backup at '{backup:?}': {e}"))?;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let state = BootState {
            target_version: target_version.to_string(),
            timestamp_unix: now,
            attempts: 0,
            confirmed_stable: false,
        };

        let raw = serde_json::to_string_pretty(&state)
            .map_err(|e| format!("Failed to serialize boot state: {e}"))?;

        if !self.config_dir.exists() {
            let _ = fs::create_dir_all(&self.config_dir);
        }

        let state_file = self.config_dir.join(BOOT_STATE_FILE);
        fs::write(&state_file, raw)
            .map_err(|e| format!("Failed to write boot state to '{state_file:?}': {e}"))?;

        Ok(())
    }

    /// Inspect boot state during early initialization.
    /// If repeated crashes (>= 2) are detected, perform atomic rollback.
    pub fn inspect_and_guard_startup(&self) -> Result<bool, String> {
        let state_file = self.state_path();
        if !state_file.exists() {
            return Ok(false);
        }

        let raw = fs::read_to_string(&state_file)
            .map_err(|e| format!("Failed to read boot state: {e}"))?;

        let mut state: BootState = match serde_json::from_str(&raw) {
            Ok(s) => s,
            Err(_) => BootState {
                target_version: "unknown".into(),
                timestamp_unix: 0,
                attempts: 0,
                confirmed_stable: false,
            },
        };

        state.attempts += 1;

        if state.attempts >= MAX_CRASH_ATTEMPTS {
            eprintln!(
                "[WATCHDOG ALERT] Repeated startup crash detected ({} attempts). Initiating automatic atomic rollback!",
                state.attempts
            );
            self.execute_atomic_rollback(&state.target_version)?;
            return Ok(true);
        }

        let updated_raw = serde_json::to_string_pretty(&state)
            .map_err(|e| format!("Failed to serialize updated boot state: {e}"))?;

        fs::write(&state_file, updated_raw)
            .map_err(|e| format!("Failed to persist boot state attempts: {e}"))?;

        self.spawn_stability_monitor();
        Ok(false)
    }

    /// Atomic cross-platform rollback:
    /// - Linux: Unlink old target binary to avoid ETXTBSY, then rename backup to target.
    /// - Windows: Rename target binary to corrupt.old, then rename backup to target.
    /// - AppImage: Operates on outer $APPIMAGE path.
    pub fn execute_atomic_rollback(&self, failed_version: &str) -> Result<(), String> {
        let target_exe = self.resolve_target_binary();
        let backup = self.backup_exe_path();

        if backup.exists() {
            #[cfg(unix)]
            {
                let _ = fs::remove_file(&target_exe);
                fs::rename(&backup, &target_exe).map_err(|e| {
                    format!("Failed to restore backup binary '{backup:?}' -> '{target_exe:?}': {e}")
                })?;
            }

            #[cfg(windows)]
            {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let corrupt_swap = target_exe.with_extension(format!("corrupt.{now}.old"));
                let _ = fs::rename(&target_exe, &corrupt_swap);
                fs::rename(&backup, &target_exe).map_err(|e| {
                    format!("Failed to restore backup binary '{backup:?}' -> '{target_exe:?}': {e}")
                })?;
            }

            let _ = fs::remove_file(&backup);
        }

        let primary_state = self.config_dir.join(BOOT_STATE_FILE);
        let legacy_state = self.config_dir.join(LEGACY_BOOT_STATE_FILE);
        let _ = fs::remove_file(primary_state);
        let _ = fs::remove_file(legacy_state);

        // Record rollback notice for user notification on subsequent boot
        let notice_file = self.config_dir.join(ROLLBACK_NOTICE_FILE);
        let notice = serde_json::json!({
            "target_version": failed_version,
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        });
        let _ = fs::write(&notice_file, notice.to_string());

        eprintln!("[WATCHDOG SUCCESS] Rollback to previous binary successfully completed.");
        Ok(())
    }

    /// Read and consume pending rollback notice
    pub fn get_and_clear_rollback_notice(&self) -> Result<Option<String>, String> {
        let notice_file = self.config_dir.join(ROLLBACK_NOTICE_FILE);
        if notice_file.exists() {
            let content = fs::read_to_string(&notice_file).unwrap_or_default();
            let _ = fs::remove_file(&notice_file);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = val.get("target_version").and_then(|v| v.as_str()) {
                    return Ok(Some(v.to_string()));
                }
            }
            return Ok(Some("unknown".into()));
        }
        Ok(None)
    }

    /// Early clean exit hook: If user exits normally within < 15 seconds,
    /// disarm watchdog so clean brief sessions are never counted as crashes.
    pub fn handle_clean_exit(&self) {
        let primary_state = self.config_dir.join(BOOT_STATE_FILE);
        let legacy_state = self.config_dir.join(LEGACY_BOOT_STATE_FILE);
        let backup_file = self.backup_exe_path();

        if primary_state.exists() || legacy_state.exists() {
            let _ = fs::remove_file(primary_state);
            let _ = fs::remove_file(legacy_state);
            let _ = fs::remove_file(backup_file);
            println!("[WATCHDOG] Application clean exit detected. Update state disarmed.");
        }
    }

    /// Mark confirmed stable (e.g. called upon first successful vault unlock)
    pub fn mark_confirmed_stable(&self) {
        let primary_state = self.config_dir.join(BOOT_STATE_FILE);
        let legacy_state = self.config_dir.join(LEGACY_BOOT_STATE_FILE);
        let backup_file = self.backup_exe_path();

        if primary_state.exists() || legacy_state.exists() {
            let _ = fs::remove_file(primary_state);
            let _ = fs::remove_file(legacy_state);
            let _ = fs::remove_file(backup_file);
            println!("[WATCHDOG] Vault unlock verified. Update confirmed permanent.");
        }
    }

    /// Background stability monitor: If application runs stably for threshold seconds,
    /// clear update marker and remove backup binary.
    fn spawn_stability_monitor(&self) {
        let state_file = self.state_path();
        let backup_file = self.backup_exe_path();
        let threshold = self.stability_threshold_secs;

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(threshold));
            if state_file.exists() {
                let _ = fs::remove_file(&state_file);
                let _ = fs::remove_file(&backup_file);
                println!(
                    "[WATCHDOG] Application ran stably for {}s. Update confirmed permanent.",
                    threshold
                );
            }
        });
    }
}

pub fn default_watchdog() -> UpdateWatchdog {
    let config_dir = dirs::config_dir()
        .map(|d| d.join("finnca"))
        .unwrap_or_else(|| PathBuf::from(".config/finnca"));
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("finnca"));
    UpdateWatchdog::new(config_dir, current_exe)
}

pub fn init_and_guard_startup() {
    if GUARD_ONCE.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let watchdog = default_watchdog();
    if let Ok(true) = watchdog.inspect_and_guard_startup() {
        let target_exe = watchdog.resolve_target_binary();
        if target_exe.exists() {
            eprintln!("[WATCHDOG] Relaunching restored binary: {:?}", target_exe);
            let _ = std::process::Command::new(&target_exe)
                .args(std::env::args().skip(1))
                .spawn();
            std::process::exit(0);
        }
    }
}

pub fn handle_clean_exit() {
    let watchdog = default_watchdog();
    watchdog.handle_clean_exit();
}

pub fn mark_confirmed_stable() {
    let watchdog = default_watchdog();
    watchdog.mark_confirmed_stable();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_arm_and_clean_exit() {
        let temp = tempfile::tempdir().unwrap();
        let config_dir = temp.path().join("config");
        let exe_dir = temp.path().join("bin");
        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&exe_dir).unwrap();

        let exe_path = exe_dir.join("finnca_bin");
        fs::write(&exe_path, "binary-v1").unwrap();

        let watchdog = UpdateWatchdog::new(config_dir.clone(), exe_path.clone());

        // 1. Arm watchdog
        watchdog.arm_watchdog("0.2.0").unwrap();

        let state_file = config_dir.join(BOOT_STATE_FILE);
        let backup_file = exe_path.with_extension("backup");
        assert!(state_file.exists());
        assert!(backup_file.exists());
        assert_eq!(fs::read_to_string(&backup_file).unwrap(), "binary-v1");

        // 2. Clean exit disarms watchdog
        watchdog.handle_clean_exit();
        assert!(!state_file.exists());
        assert!(!backup_file.exists());
    }

    #[test]
    fn test_watchdog_crash_loop_triggers_rollback() {
        let temp = tempfile::tempdir().unwrap();
        let config_dir = temp.path().join("config");
        let exe_dir = temp.path().join("bin");
        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&exe_dir).unwrap();

        let exe_path = exe_dir.join("finnca_bin");
        fs::write(&exe_path, "good-binary-v1").unwrap();

        let watchdog = UpdateWatchdog::new(config_dir.clone(), exe_path.clone());

        // Arm watchdog
        watchdog.arm_watchdog("0.2.0").unwrap();

        // Simulate new bad binary installed
        fs::write(&exe_path, "broken-binary-v2").unwrap();

        // Attempt 1 (e.g. boot 1 crashes before 15s)
        let rolled_back_1 = watchdog.inspect_and_guard_startup().unwrap();
        assert!(!rolled_back_1);
        let state_raw = fs::read_to_string(config_dir.join(BOOT_STATE_FILE)).unwrap();
        let state: BootState = serde_json::from_str(&state_raw).unwrap();
        assert_eq!(state.attempts, 1);
        assert_eq!(fs::read_to_string(&exe_path).unwrap(), "broken-binary-v2");

        // Attempt 2 (boot 2 crashes again -> attempts reaches 2 -> trigger rollback!)
        let rolled_back_2 = watchdog.inspect_and_guard_startup().unwrap();
        assert!(rolled_back_2);

        // Binary restored to good version!
        assert_eq!(fs::read_to_string(&exe_path).unwrap(), "good-binary-v1");
        // State file cleaned up
        assert!(!config_dir.join(BOOT_STATE_FILE).exists());

        // Rollback notice recorded and consumable exactly once
        let notice = watchdog.get_and_clear_rollback_notice().unwrap();
        assert_eq!(notice, Some("0.2.0".to_string()));
        let notice_second = watchdog.get_and_clear_rollback_notice().unwrap();
        assert_eq!(notice_second, None);
    }

    #[test]
    fn test_watchdog_mark_confirmed_stable() {
        let temp = tempfile::tempdir().unwrap();
        let config_dir = temp.path().join("config");
        let exe_dir = temp.path().join("bin");
        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&exe_dir).unwrap();

        let exe_path = exe_dir.join("finnca_bin");
        fs::write(&exe_path, "good-binary").unwrap();

        let watchdog = UpdateWatchdog::new(config_dir.clone(), exe_path.clone());
        watchdog.arm_watchdog("0.3.0").unwrap();

        let state_file = config_dir.join(BOOT_STATE_FILE);
        let backup_file = exe_path.with_extension("backup");
        assert!(state_file.exists());
        assert!(backup_file.exists());

        // Vault unlock triggers immediate confirmation
        watchdog.mark_confirmed_stable();
        assert!(!state_file.exists());
        assert!(!backup_file.exists());
    }
}

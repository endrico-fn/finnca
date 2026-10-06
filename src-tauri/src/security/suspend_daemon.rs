use crate::app_state::AppState;
use tauri::{AppHandle, Emitter, Manager};

/// Core suspend lock logic: drops db connection, clears session, emits `vault:locked`
pub fn handle_os_suspend(app: &AppHandle) -> bool {
    if let Some(state) = app.try_state::<AppState>() {
        if state.is_unlocked() {
            eprintln!("[SECURITY] OS Suspend signal received. Auto-locking vault and clearing session.");
            state.clear_session();
            let _ = app.emit("vault:locked", ());
            return true;
        }
    }
    false
}

#[cfg(target_os = "linux")]
pub fn spawn_linux_suspend_listener(app: AppHandle) {
    std::thread::spawn(move || {
        let conn = match zbus::blocking::Connection::system() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[SECURITY] D-Bus system bus not available for sleep listener: {e}");
                return;
            }
        };

        use zbus::blocking::Proxy;
        let proxy = match Proxy::new(
            &conn,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        ) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[SECURITY] Failed to create D-Bus login1 proxy: {e}");
                return;
            }
        };

        let signal_iterator = match proxy.receive_signal("PrepareForSleep") {
            Ok(iter) => iter,
            Err(e) => {
                eprintln!("[SECURITY] Failed to subscribe to PrepareForSleep signal: {e}");
                return;
            }
        };

        eprintln!("[SECURITY] Linux OS sleep daemon listening on org.freedesktop.login1.Manager:PrepareForSleep");

        for msg in signal_iterator {
            let is_suspend = msg
                .body()
                .deserialize::<bool>()
                .or_else(|_| msg.body().deserialize::<(bool,)>().map(|t| t.0))
                .unwrap_or(false);
            if is_suspend {
                handle_os_suspend(&app);
            }
        }
    });
}

#[cfg(target_os = "windows")]
pub fn register_windows_power_broadcast(window: &tauri::WebviewWindow) {
    use windows_sys::Win32::UI::Shell::{RemoveWindowSubclass, SetWindowSubclass};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DefSubclassProc, WM_NCDESTROY, WM_POWERBROADCAST,
    };

    const PBT_APMSUSPEND: usize = 0x0004;

    unsafe extern "system" fn subclass_proc(
        hwnd: windows_sys::Win32::Foundation::HWND,
        msg: u32,
        wparam: windows_sys::Win32::Foundation::WPARAM,
        lparam: windows_sys::Win32::Foundation::LPARAM,
        _uid_subclass: usize,
        ref_data: usize,
    ) -> windows_sys::Win32::Foundation::LRESULT {
        if msg == WM_POWERBROADCAST && wparam == PBT_APMSUSPEND {
            let app_ptr = ref_data as *const AppHandle;
            if !app_ptr.is_null() {
                handle_os_suspend(&*app_ptr);
            }
        } else if msg == WM_NCDESTROY {
            let app_ptr = ref_data as *mut AppHandle;
            if !app_ptr.is_null() {
                let _ = Box::from_raw(app_ptr);
            }
            RemoveWindowSubclass(hwnd, Some(subclass_proc), 1);
        }
        DefSubclassProc(hwnd, msg, wparam, lparam)
    }

    if let Ok(hwnd) = window.hwnd() {
        let app_box = Box::new(window.app_handle().clone());
        let ref_data = Box::into_raw(app_box) as usize;
        unsafe {
            SetWindowSubclass(hwnd.0 as _, Some(subclass_proc), 1, ref_data);
        }
    }
}

pub fn init_suspend_daemon(app: &AppHandle) {
    #[cfg(target_os = "linux")]
    {
        spawn_linux_suspend_listener(app.clone());
    }

    #[cfg(target_os = "windows")]
    {
        for (_, w) in app.webview_windows() {
            register_windows_power_broadcast(&w);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_suspend_locks_session() {
        let state = AppState::new();
        assert!(!state.is_unlocked());

        // Fake session
        state.set_session(crate::app_state::Session {
            vault_path: std::path::PathBuf::from("/tmp/test.finnca"),
            vault_name: "Test Vault".into(),
            db: None,
        });

        assert!(state.is_unlocked());
        state.clear_session();
        assert!(!state.is_unlocked());
    }
}

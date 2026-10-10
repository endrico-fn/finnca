// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("GTK_OVERLAY_SCROLLING").is_none() {
            std::env::set_var("GTK_OVERLAY_SCROLLING", "0");
        }
        if std::env::var_os("GTK_USE_PORTAL").is_none() {
            std::env::set_var("GTK_USE_PORTAL", "1");
        }
    }
    finnca_lib::run()
}

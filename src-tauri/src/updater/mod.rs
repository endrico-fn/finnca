pub mod commands;
pub mod watchdog;

pub use watchdog::{
    default_watchdog, handle_clean_exit, init_and_guard_startup, BootState, UpdateWatchdog,
};

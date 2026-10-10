use tauri::AppHandle;
pub mod accounts;
pub mod app_config;
pub mod app_state;
pub mod audit;
pub mod auth;
pub mod budget;
mod crypto;
pub mod db;
pub mod diagnostics;
pub mod ledger;
pub mod plan;
pub mod plugins;
pub mod reconcile;
pub mod report;
pub mod security;
pub mod shared;
pub mod updater;
pub mod vault;

pub use app_config as config;
pub use app_state as state;

use std::fs;
use std::path::Path;

use app_state::{AppState, Session};
use serde::Serialize;

pub(crate) const MAX_PLAINTEXT_BYTES: usize = 11 * 1024 * 1024;

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
    fs::write(&tmp, bytes).map_err(|e| format!("failed to write temp file: {e}"))?;
    fs::rename(&tmp, path).map_err(|e| format!("failed to commit file: {e}"))?;
    Ok(())
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

#[derive(Serialize, specta::Type)]
pub struct AppStateView {
    pub configured: bool,
    pub unlocked: bool,
    pub username: Option<String>,
    pub vault_name: Option<String>,
    pub vault_path: Option<String>,
    pub settings: config::Settings,
    pub pending_import_path: Option<String>,
}

pub(crate) fn view(app: &AppHandle, state: &AppState) -> Result<AppStateView, String> {
    let config = config::load(app)?;
    let configured = config.vault.is_some() && config.username.is_some();
    let unlocked = lock_session(state)?.is_some();
    let pending_import_path = state.get_pending_import_path();
    Ok(AppStateView {
        configured,
        unlocked,
        username: config.username,
        vault_name: config.vault.as_ref().map(|v| v.name.clone()),
        vault_path: config.vault.as_ref().map(|v| v.path.clone()),
        settings: config.settings,
        pending_import_path,
    })
}

pub(crate) fn percent_decode_str(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let mut chars = s.as_bytes().iter().copied();
    while let Some(b) = chars.next() {
        if b == b'%' {
            if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                if let Ok(val) =
                    u8::from_str_radix(std::str::from_utf8(&[h1, h2]).unwrap_or(""), 16)
                {
                    bytes.push(val);
                    continue;
                }
            }
        }
        bytes.push(b);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

pub(crate) fn find_finnca_file_arg<I, S>(args: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter().find_map(|arg| {
        let mut s = arg.as_ref().trim();
        if s.starts_with('-') {
            return None;
        }
        if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
            s = &s[1..s.len() - 1];
            s = s.trim();
        }
        let clean = if let Some(stripped) = s.strip_prefix("file://") {
            #[cfg(windows)]
            let path_part = stripped.strip_prefix('/').unwrap_or(stripped);
            #[cfg(not(windows))]
            let path_part = stripped;
            percent_decode_str(path_part)
        } else {
            s.to_string()
        };
        if clean.to_ascii_lowercase().ends_with(".finnca") {
            Some(clean)
        } else {
            None
        }
    })
}

pub fn create_specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .dangerously_cast_bigints_to_number()
        .disable_serde_phases()
        .commands(tauri_specta::collect_commands![
            auth::commands::get_app_state,
            auth::commands::unlock,
            auth::commands::lock,
            auth::commands::change_password,
            security::commands::get_boot_id,
            security::commands::set_auto_lock_mode,
            security::commands::simulate_os_suspend_cmd,
            updater::commands::arm_update_watchdog_cmd,
            updater::commands::disarm_update_watchdog_cmd,
            updater::commands::get_rollback_notice_cmd,
            vault::commands::create_vault,
            vault::commands::create_account,
            vault::commands::import_vault,
            vault::commands::inspect_vault_folder,
            vault::commands::get_pending_import_path,
            vault::commands::clear_pending_import_path,
            vault::commands::delete_vault_and_account,
            vault::commands::open_vault_folder,
            vault::commands::export_text_file,
            vault::commands::get_known_vaults_cmd,
            vault::commands::remember_known_vault_cmd,
            vault::commands::forget_known_vault_cmd,
            vault::commands::set_active_vault,
            vault::commands::export_vault_backup_folder,
            vault::commands::rename_user,
            vault::commands::rename_vault,
            vault::commands::write_file_raw,
            accounts::commands::create_account_cmd,
            accounts::commands::update_account_cmd,
            accounts::commands::delete_account_cmd,
            accounts::commands::get_account_cmd,
            accounts::commands::list_accounts_cmd,
            accounts::commands::seed_root_accounts_cmd,
            accounts::commands::seed_starter_accounts_cmd,
            ledger::commands::post_journal_entry_cmd,
            ledger::commands::update_journal_entry_cmd,
            ledger::commands::delete_journal_entry_cmd,
            ledger::commands::get_journal_entry_cmd,
            ledger::commands::list_journal_entries_cmd,
            ledger::commands::get_account_ledger_cmd,
            ledger::commands::get_ledger_totals_cmd,
            ledger::commands::get_dashboard_metrics_cmd,
            ledger::commands::diagnose_vault_health_cmd,
            ledger::commands::get_closing_date_cmd,
            ledger::commands::set_closing_date_cmd,
            ledger::commands::export_beancount_cmd,
            budget::commands::upsert_budget_cmd,
            budget::commands::delete_budget_cmd,
            budget::commands::get_budget_summary_cmd,
            audit::commands::get_audit_log_cmd,
            audit::commands::get_entity_audit_log_cmd,
            audit::commands::verify_audit_log_integrity_cmd,
            plan::commands::create_plan_cmd,
            plan::commands::update_plan_cmd,
            plan::commands::delete_plan_cmd,
            plan::commands::get_plan_cmd,
            plan::commands::list_plans_cmd,
            plan::commands::list_plans_with_progress_cmd,
            plan::commands::record_plan_installment_cmd,
            plan::commands::get_due_recurring_plans_cmd,
            plan::commands::post_due_recurring_batch_cmd,
            reconcile::commands::get_reconciliation_status_cmd,
            reconcile::commands::set_posting_reconciled_cmd,
            reconcile::commands::bulk_set_postings_reconciled_cmd,
            reconcile::commands::finish_reconciliation_cmd,
            reconcile::commands::match_statement_cmd,
            reconcile::commands::read_statement_file_cmd,
            reconcile::commands::list_reconcile_rules_cmd,
            reconcile::commands::create_reconcile_rule_cmd,
            reconcile::commands::delete_reconcile_rule_cmd,
            reconcile::commands::evaluate_reconcile_rules_cmd,
            report::commands::get_profit_loss_report_cmd,
            report::commands::get_balance_sheet_report_cmd,
            report::commands::get_cash_flow_report_cmd,
            report::commands::get_trial_balance_report_cmd,
            report::commands::get_fx_revaluation_report_cmd,
            report::commands::get_historical_trends_report_cmd,
            report::commands::get_monthly_cashflow_summary_cmd,
            plugins::commands::list_plugins_cmd,
            plugins::commands::execute_statement_parser_cmd
        ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    diagnostics::init_crash_reporter();
    updater::watchdog::init_and_guard_startup();

    let builder = create_specta_builder();

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default().header("/* eslint-disable */\n"),
            "../src/lib/core/ipc/bindings.gen.ts",
        )
        .expect("error exporting specta typescript bindings");

    let initial_file = find_finnca_file_arg(std::env::args().skip(1));
    let initial_file_clone = initial_file.clone();
    let app_state = AppState::with_pending_import_path(initial_file);

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            use tauri::{Emitter, Manager};
            if let Some(file_arg) = find_finnca_file_arg(argv.iter().skip(1)) {
                if let Some(state) = app.try_state::<AppState>() {
                    state.set_pending_import_path(Some(file_arg.clone()));
                }
                let _ = app.emit("finnca:import-file", file_arg);
            }
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .build(),
        )
        .manage(app_state)
        .setup(move |app| {
            use tauri::Emitter;
            security::suspend_daemon::init_suspend_daemon(app.handle());
            if let Some(path) = initial_file_clone {
                let _ = app.emit("finnca:import-file", path);
            }
            Ok(())
        })
        .invoke_handler(builder.invoke_handler())
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                updater::watchdog::handle_clean_exit();
            }
        });
}

#[cfg(test)]
mod specta_export_test {
    use super::*;

    #[test]
    fn test_export_specta_bindings() {
        let builder = create_specta_builder();
        builder
            .export(
                specta_typescript::Typescript::default().header("/* eslint-disable */\n"),
                "../src/lib/core/ipc/bindings.gen.ts",
            )
            .expect("error exporting specta typescript bindings in test");
    }

    #[test]
    fn test_find_finnca_file_arg() {
        let args = vec!["finnca", "--debug", "/home/user/vault.finnca"];
        assert_eq!(
            find_finnca_file_arg(args),
            Some("/home/user/vault.finnca".to_string())
        );

        let no_match = vec!["finnca", "--debug", "some_file.txt"];
        assert_eq!(find_finnca_file_arg(no_match), None);

        let flag_ends_with_finnca = vec!["finnca", "--out=test.finnca", "-f=data.finnca"];
        assert_eq!(find_finnca_file_arg(flag_ends_with_finnca), None);

        let quoted = vec!["finnca", "\"C:\\Users\\John Doe\\My Vault.finnca\""];
        assert_eq!(
            find_finnca_file_arg(quoted),
            Some("C:\\Users\\John Doe\\My Vault.finnca".to_string())
        );

        let uri = vec!["finnca", "file:///home/user/My%20Vault.finnca"];
        assert_eq!(
            find_finnca_file_arg(uri),
            Some("/home/user/My Vault.finnca".to_string())
        );

        let case_insensitive = vec!["C:\\Users\\Doc\\Backup.FINNCA"];
        assert_eq!(
            find_finnca_file_arg(case_insensitive),
            Some("C:\\Users\\Doc\\Backup.FINNCA".to_string())
        );
    }
}

use tauri::AppHandle;
pub mod accounts;
pub mod app_config;
pub mod app_state;
pub mod audit;
pub mod auth;
pub mod budget;
mod crypto;
pub mod db;
pub mod ledger;
pub mod plan;
pub mod reconcile;
pub mod report;
pub mod security;
pub mod shared;
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

#[derive(Serialize)]
pub struct AppStateView {
    pub configured: bool,
    pub unlocked: bool,
    pub username: Option<String>,
    pub vault_name: Option<String>,
    pub vault_path: Option<String>,
    pub settings: config::Settings,
}

pub(crate) fn view(app: &AppHandle, state: &AppState) -> Result<AppStateView, String> {
    let config = config::load(app)?;
    let configured = config.vault.is_some() && config.username.is_some();
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
            auth::commands::get_app_state,
            auth::commands::unlock,
            auth::commands::lock,
            auth::commands::change_password,
            security::commands::get_boot_id,
            security::commands::set_auto_lock_mode,
            vault::commands::create_vault,
            vault::commands::create_account,
            vault::commands::import_vault,
            vault::commands::inspect_vault_folder,
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
            budget::commands::upsert_budget_cmd,
            budget::commands::delete_budget_cmd,
            budget::commands::get_budget_summary_cmd,
            audit::commands::get_audit_log_cmd,
            audit::commands::get_entity_audit_log_cmd,
            plan::commands::create_plan_cmd,
            plan::commands::update_plan_cmd,
            plan::commands::delete_plan_cmd,
            plan::commands::get_plan_cmd,
            plan::commands::list_plans_cmd,
            plan::commands::list_plans_with_progress_cmd,
            reconcile::commands::get_reconciliation_status_cmd,
            reconcile::commands::set_posting_reconciled_cmd,
            reconcile::commands::bulk_set_postings_reconciled_cmd,
            reconcile::commands::finish_reconciliation_cmd,
            reconcile::commands::match_statement_cmd,
            reconcile::commands::read_statement_file_cmd,
            report::commands::get_profit_loss_report_cmd,
            report::commands::get_balance_sheet_report_cmd,
            report::commands::get_cash_flow_report_cmd,
            report::commands::get_trial_balance_report_cmd,
            report::commands::get_fx_revaluation_report_cmd,
            report::commands::get_historical_trends_report_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

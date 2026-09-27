use super::dto::{
    BalanceSheetReport, CashFlowReport, FxRevaluationReport, HistoricalTrendsReport,
    MonthlyCashflowPoint, ProfitLossReport, TrialBalanceReport,
};
use super::generators::{
    balance_sheet, cash_flow, fx_revaluation, historical_trends, monthly_cashflow, profit_loss,
    trial_balance,
};
use crate::shared::AppError;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn get_profit_loss_report_cmd(
    state: State<'_, AppState>,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<ProfitLossReport, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    profit_loss::generate(&conn, from_date.as_deref(), to_date.as_deref())
}

#[tauri::command]
#[specta::specta]
pub fn get_balance_sheet_report_cmd(
    state: State<'_, AppState>,
    as_of_date: Option<String>,
) -> Result<BalanceSheetReport, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    balance_sheet::generate(&conn, as_of_date.as_deref())
}

#[tauri::command]
#[specta::specta]
pub fn get_cash_flow_report_cmd(
    state: State<'_, AppState>,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<CashFlowReport, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    cash_flow::generate(&conn, from_date.as_deref(), to_date.as_deref())
}

#[tauri::command]
#[specta::specta]
pub fn get_trial_balance_report_cmd(
    state: State<'_, AppState>,
    as_of_date: Option<String>,
) -> Result<TrialBalanceReport, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    trial_balance::generate(&conn, as_of_date.as_deref())
}

#[tauri::command]
#[specta::specta]
pub fn get_fx_revaluation_report_cmd(
    state: State<'_, AppState>,
    as_of_date: Option<String>,
    fx_rate: Option<i64>,
) -> Result<FxRevaluationReport, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    fx_revaluation::generate(&conn, as_of_date.as_deref(), fx_rate.unwrap_or(0))
}

#[tauri::command]
#[specta::specta]
pub fn get_historical_trends_report_cmd(
    state: State<'_, AppState>,
    from_date: String,
    to_date: String,
    fx_rate: Option<i64>,
) -> Result<HistoricalTrendsReport, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    historical_trends::generate(&conn, &from_date, &to_date, fx_rate.unwrap_or(0))
}

#[tauri::command]
#[specta::specta]
pub fn get_monthly_cashflow_summary_cmd(
    state: State<'_, AppState>,
    months: Option<u32>,
) -> Result<Vec<MonthlyCashflowPoint>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    monthly_cashflow::generate(&conn, months.unwrap_or(6))
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AccountReportRow {
    pub account_id: String,
    pub code: String,
    pub name: String,
    pub currency: String,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ProfitLossReport {
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub income_rows: Vec<AccountReportRow>,
    pub expense_rows: Vec<AccountReportRow>,
    pub total_income: i64,
    pub total_expenses: i64,
    pub net_income: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BalanceSheetReport {
    pub as_of_date: Option<String>,
    pub asset_rows: Vec<AccountReportRow>,
    pub liability_rows: Vec<AccountReportRow>,
    pub equity_rows: Vec<AccountReportRow>,
    pub total_assets: i64,
    pub total_liabilities: i64,
    pub total_equity: i64,
    pub net_income: i64,
    pub discrepancy: i64,
    pub is_balanced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CashFlowActivityRow {
    pub date: String,
    pub category: String,
    pub description: String,
    pub account_name: String,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CashFlowReport {
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub starting_cash: i64,
    pub operating_cash_flow: i64,
    pub investing_cash_flow: i64,
    pub financing_cash_flow: i64,
    pub net_cash_change: i64,
    pub ending_cash: i64,
    pub operating_rows: Vec<CashFlowActivityRow>,
    pub investing_rows: Vec<CashFlowActivityRow>,
    pub financing_rows: Vec<CashFlowActivityRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct TrialBalanceRow {
    pub account_id: String,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub debit: i64,
    pub credit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct TrialBalanceReport {
    pub as_of_date: Option<String>,
    pub rows: Vec<TrialBalanceRow>,
    pub total_debit: i64,
    pub total_credit: i64,
    pub is_balanced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct FxRevaluationItem {
    pub account_id: String,
    pub code: String,
    pub name: String,
    pub currency: String,
    pub native_balance: i64,
    pub cost_basis_idr: i64,
    pub current_value_idr: i64,
    pub unrealized_gain_idr: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct FxRevaluationReport {
    pub as_of_date: Option<String>,
    pub current_fx_rate: i64,
    pub items: Vec<FxRevaluationItem>,
    pub total_cost_basis_idr: i64,
    pub total_current_value_idr: i64,
    pub total_unrealized_gain_idr: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DailyTrendPoint {
    pub date: String,
    pub net_worth: i64,
    pub assets: i64,
    pub liabilities: i64,
    pub liquid_cash: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct HistoricalTrendsReport {
    pub from_date: String,
    pub to_date: String,
    pub points: Vec<DailyTrendPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct MonthlyCashflowPoint {
    pub month: String,
    pub income: i64,
    pub expense: i64,
    pub net: i64,
}

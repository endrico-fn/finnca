use super::models::ReconcileState;
use crate::accounts::models::AccountType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct PostingInput {
    #[serde(default)]
    pub id: Option<String>,
    pub account_id: String,
    pub amount: i64,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub reconcile: Option<ReconcileState>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub fx_rate: Option<i64>,
    #[serde(default)]
    pub cost_amount: Option<i64>,
}

impl PostingInput {
    pub fn new(account_id: impl Into<String>, amount: i64) -> Self {
        Self {
            id: None,
            account_id: account_id.into(),
            amount,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct CreateJournalEntryInput {
    #[serde(default)]
    pub id: Option<String>,
    pub date: String,
    pub description: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub reference_no: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub plan_id: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub fx_rate: Option<i64>,
    pub postings: Vec<PostingInput>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct UpdateJournalEntryInput {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub notes: Option<Option<String>>,
    #[serde(default)]
    pub reference_no: Option<Option<String>>,
    #[serde(default)]
    pub due_date: Option<Option<String>>,
    #[serde(default)]
    pub plan_id: Option<Option<String>>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub fx_rate: Option<i64>,
    #[serde(default)]
    pub postings: Option<Vec<PostingInput>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct PostingView {
    pub id: String,
    pub entry_id: String,
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
    pub account_type: AccountType,
    pub amount: i64,
    pub memo: Option<String>,
    pub action: Option<String>,
    pub reconcile: ReconcileState,
    pub reconciled_at: Option<i64>,
    pub currency: String,
    pub fx_rate: Option<i64>,
    pub cost_amount: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct JournalEntryView {
    pub id: String,
    pub date: String,
    pub description: String,
    pub notes: Option<String>,
    pub reference_no: Option<String>,
    pub due_date: Option<String>,
    pub plan_id: Option<String>,
    pub currency: String,
    pub fx_rate: i64,
    pub posted_at: i64,
    pub postings: Vec<PostingView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct LedgerTotalsView {
    pub total_assets: i64,
    pub total_liabilities: i64,
    pub total_equity: i64,
    pub total_income: i64,
    pub total_expenses: i64,
    pub net_income: i64,
    pub balance_sheet_discrepancy: i64,
    pub is_balance_sheet_aligned: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct AccountRunningLedgerItem {
    pub posting_id: String,
    pub entry_id: String,
    pub date: String,
    pub description: String,
    pub amount: i64,
    pub running_balance: i64,
    pub reconcile: ReconcileState,
    pub memo: Option<String>,
    pub offset_account: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct DashboardMetricsView {
    pub total_assets: i64,
    pub total_liabilities: i64,
    pub net_worth: i64,
    pub monthly_burn_30d: i64,
    pub runway_months: f64,
    pub this_month_income: i64,
    pub this_month_expense: i64,
    pub this_month_net: i64,
    pub overdue_invoices_count: i64,
    pub overdue_invoices_amount: i64,
    pub upcoming_due_count: i64,
    pub upcoming_due_amount: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct VaultHealthReport {
    pub is_healthy: bool,
    pub sqlite_integrity: String,
    pub foreign_keys_ok: bool,
    pub unbalanced_entries_count: usize,
    pub orphan_postings_count: usize,
    pub placeholder_postings_count: usize,
    pub issues: Vec<String>,
}

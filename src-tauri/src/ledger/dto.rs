use super::models::ReconcileState;
use crate::accounts::models::AccountType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PostingInput {
    pub id: Option<String>,
    pub account_id: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub action: Option<String>,
    pub reconcile: Option<ReconcileState>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateJournalEntryInput {
    pub id: Option<String>,
    pub date: String,
    pub description: String,
    pub notes: Option<String>,
    pub currency: Option<String>,
    pub fx_rate: Option<i64>,
    pub postings: Vec<PostingInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateJournalEntryInput {
    pub date: Option<String>,
    pub description: Option<String>,
    pub notes: Option<Option<String>>,
    pub currency: Option<String>,
    pub fx_rate: Option<i64>,
    pub postings: Option<Vec<PostingInput>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEntryView {
    pub id: String,
    pub date: String,
    pub description: String,
    pub notes: Option<String>,
    pub currency: String,
    pub fx_rate: i64,
    pub posted_at: i64,
    pub postings: Vec<PostingView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountRunningLedgerItem {
    pub entry_id: String,
    pub date: String,
    pub description: String,
    pub amount: i64,
    pub running_balance: i64,
    pub reconcile: ReconcileState,
}

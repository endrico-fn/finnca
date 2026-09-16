use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UpsertBudgetInput {
    pub id: Option<String>,
    pub month: String,
    pub account_id: String,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct EnvelopeView {
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
    pub assigned: i64,
    pub activity: i64,
    pub available: i64,
}

#[derive(Debug, Serialize)]
pub struct BudgetMonthSummary {
    pub month: String,
    pub envelopes: Vec<EnvelopeView>,
    pub total_assigned: i64,
    pub total_activity: i64,
    pub to_be_budgeted: i64,
}

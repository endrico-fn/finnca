use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BudgetAllocation {
    pub id: String,
    pub month: String,
    pub account_id: String,
    pub amount: i64,
    pub created_at: i64,
}

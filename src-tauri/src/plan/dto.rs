use super::models::{PaymentPlan, PlanFrequency, PlanStatus, PlanType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CreatePlanInput {
    pub title: String,
    pub plan_type: PlanType,
    #[serde(default)]
    pub status: Option<PlanStatus>,
    pub total_amount: i64,
    pub installment_amount: i64,
    pub frequency: PlanFrequency,
    pub start_date: String,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub day_of_month: Option<u32>,
    pub from_account_id: String,
    pub to_account_id: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub last_posted_date: Option<String>,
    #[serde(default)]
    pub auto_post: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdatePlanInput {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub plan_type: Option<PlanType>,
    #[serde(default)]
    pub status: Option<PlanStatus>,
    #[serde(default)]
    pub total_amount: Option<i64>,
    #[serde(default)]
    pub installment_amount: Option<i64>,
    #[serde(default)]
    pub frequency: Option<PlanFrequency>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub due_date: Option<Option<String>>,
    #[serde(default)]
    pub day_of_month: Option<Option<u32>>,
    #[serde(default)]
    pub from_account_id: Option<String>,
    #[serde(default)]
    pub to_account_id: Option<String>,
    #[serde(default)]
    pub notes: Option<Option<String>>,
    #[serde(default)]
    pub last_posted_date: Option<Option<String>>,
    #[serde(default)]
    pub auto_post: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PlanProgressView {
    pub plan: PaymentPlan,
    pub paid_amount: i64,
    pub remaining_amount: i64,
    pub progress_percent: i64,
    pub is_settled: bool,
    pub installments_paid_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct RecordInstallmentInput {
    pub plan_id: String,
    pub date: String,
    pub amount: i64,
    #[serde(default)]
    pub reference_no: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DueRecurringPlanView {
    pub plan_id: String,
    pub title: String,
    pub plan_type: PlanType,
    pub amount: i64,
    pub currency: String,
    pub from_account_id: String,
    pub from_account_name: String,
    pub to_account_id: String,
    pub to_account_name: String,
    pub next_due_date: String,
    pub is_overdue: bool,
    pub days_diff: i64,
    pub last_posted_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PostDueRecurringBatchInput {
    pub plan_ids: Vec<String>,
    #[serde(default)]
    pub date: Option<String>,
}

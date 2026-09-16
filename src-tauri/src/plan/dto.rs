use super::models::{PaymentPlan, PlanFrequency, PlanStatus, PlanType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreatePlanInput {
    pub title: String,
    pub plan_type: PlanType,
    pub status: Option<PlanStatus>,
    pub total_amount: i64,
    pub installment_amount: i64,
    pub frequency: PlanFrequency,
    pub start_date: String,
    pub due_date: Option<String>,
    pub day_of_month: Option<u32>,
    pub from_account_id: String,
    pub to_account_id: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlanInput {
    pub title: Option<String>,
    pub plan_type: Option<PlanType>,
    pub status: Option<PlanStatus>,
    pub total_amount: Option<i64>,
    pub installment_amount: Option<i64>,
    pub frequency: Option<PlanFrequency>,
    pub start_date: Option<String>,
    pub due_date: Option<Option<String>>,
    pub day_of_month: Option<Option<u32>>,
    pub from_account_id: Option<String>,
    pub to_account_id: Option<String>,
    pub notes: Option<Option<String>>,
}

#[derive(Debug, Serialize)]
pub struct PlanProgressView {
    pub plan: PaymentPlan,
    pub paid_amount: i64,
    pub remaining_amount: i64,
    pub progress_percent: i64,
    pub is_settled: bool,
    pub installments_paid_count: i64,
}

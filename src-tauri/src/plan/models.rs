use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PlanType {
    Receivable,
    Payable,
    Recurring,
}

impl PlanType {
    pub fn as_str(self) -> &'static str {
        match self {
            PlanType::Receivable => "RECEIVABLE",
            PlanType::Payable => "PAYABLE",
            PlanType::Recurring => "RECURRING",
        }
    }
}

impl FromStr for PlanType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "RECEIVABLE" => Ok(PlanType::Receivable),
            "PAYABLE" => Ok(PlanType::Payable),
            "RECURRING" => Ok(PlanType::Recurring),
            _ => Err(format!("Unknown plan type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PlanStatus {
    Active,
    Completed,
    Overdue,
    Archived,
}

impl PlanStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            PlanStatus::Active => "ACTIVE",
            PlanStatus::Completed => "COMPLETED",
            PlanStatus::Overdue => "OVERDUE",
            PlanStatus::Archived => "ARCHIVED",
        }
    }
}

impl FromStr for PlanStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ACTIVE" => Ok(PlanStatus::Active),
            "COMPLETED" => Ok(PlanStatus::Completed),
            "OVERDUE" => Ok(PlanStatus::Overdue),
            "ARCHIVED" => Ok(PlanStatus::Archived),
            _ => Err(format!("Unknown plan status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PlanFrequency {
    Daily,
    Weekly,
    Monthly,
}

impl PlanFrequency {
    pub fn as_str(self) -> &'static str {
        match self {
            PlanFrequency::Daily => "DAILY",
            PlanFrequency::Weekly => "WEEKLY",
            PlanFrequency::Monthly => "MONTHLY",
        }
    }
}

impl FromStr for PlanFrequency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DAILY" => Ok(PlanFrequency::Daily),
            "WEEKLY" => Ok(PlanFrequency::Weekly),
            "MONTHLY" => Ok(PlanFrequency::Monthly),
            _ => Err(format!("Unknown plan frequency: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentPlan {
    pub id: String,
    pub title: String,
    pub plan_type: PlanType,
    pub status: PlanStatus,
    pub total_amount: i64,
    pub installment_amount: i64,
    pub frequency: PlanFrequency,
    pub start_date: String,
    pub due_date: Option<String>,
    pub day_of_month: Option<u32>,
    pub from_account_id: String,
    pub to_account_id: String,
    pub notes: Option<String>,
    pub created_at: i64,
}

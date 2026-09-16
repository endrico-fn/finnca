use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconcileState {
    #[serde(rename = "n")]
    None,
    #[serde(rename = "c")]
    Cleared,
    #[serde(rename = "y")]
    Reconciled,
}

impl ReconcileState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReconcileState::None => "n",
            ReconcileState::Cleared => "c",
            ReconcileState::Reconciled => "y",
        }
    }
}

impl FromStr for ReconcileState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "n" => Ok(ReconcileState::None),
            "c" => Ok(ReconcileState::Cleared),
            "y" => Ok(ReconcileState::Reconciled),
            _ => Err(format!("Invalid reconcile state: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: String,
    pub date: String,
    pub description: String,
    pub notes: Option<String>,
    pub currency: String,
    pub fx_rate: i64,
    pub posted_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Posting {
    pub id: String,
    pub entry_id: String,
    pub account_id: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub action: Option<String>,
    pub reconcile: ReconcileState,
    pub reconciled_at: Option<i64>,
}

use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "UPPERCASE")]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Asset => "ASSET",
            AccountType::Liability => "LIABILITY",
            AccountType::Equity => "EQUITY",
            AccountType::Income => "INCOME",
            AccountType::Expense => "EXPENSE",
        }
    }

    pub fn is_debit_normal(&self) -> bool {
        matches!(self, AccountType::Asset | AccountType::Expense)
    }
}

impl FromStr for AccountType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ASSET" => Ok(AccountType::Asset),
            "LIABILITY" => Ok(AccountType::Liability),
            "EQUITY" => Ok(AccountType::Equity),
            "INCOME" => Ok(AccountType::Income),
            "EXPENSE" => Ok(AccountType::Expense),
            _ => Err(format!("Unknown account type: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Account {
    pub id: String,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub parent_id: Option<String>,
    pub currency: String,
    pub placeholder: bool,
    pub hidden: bool,
    pub color: Option<String>,
    pub note: Option<String>,
    pub description: Option<String>,
    pub interest_rate: Option<f64>,
    pub created_at: i64,
}

use super::models::{Account, AccountType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAccountInput {
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub parent_id: Option<String>,
    pub currency: Option<String>,
    pub placeholder: Option<bool>,
    pub hidden: Option<bool>,
    pub color: Option<String>,
    pub note: Option<String>,
    pub description: Option<String>,
    pub interest_rate: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAccountInput {
    pub code: Option<String>,
    pub name: Option<String>,
    pub account_type: Option<AccountType>,
    pub parent_id: Option<Option<String>>,
    pub currency: Option<String>,
    pub placeholder: Option<bool>,
    pub hidden: Option<bool>,
    pub color: Option<Option<String>>,
    pub note: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub interest_rate: Option<Option<f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountBalanceView {
    pub account: Account,
    pub direct_balance: i64,
    pub recursive_balance: i64,
}

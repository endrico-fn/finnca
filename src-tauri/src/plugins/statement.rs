use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ParseStatementInput {
    pub parser_id: String,
    pub file_name: String,
    pub file_bytes_base64: String,
    pub account_currency: String,
    pub account_is_debit_normal: bool,
    #[serde(default)]
    pub options: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ParsedStatementRow {
    /// Format tanggal terstandarisasi ISO-8601: "YYYY-MM-DD"
    pub date: String,
    /// Nominal dalam integer minor units (positif untuk uang masuk, negatif untuk uang keluar)
    pub amount: i64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub reference_no: Option<String>,
    #[serde(default)]
    pub balance_after: Option<i64>,
    #[serde(default)]
    pub payee_or_payer: Option<String>,
    #[serde(default)]
    pub category_hint: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct ParsedStatementOutput {
    #[serde(default)]
    pub bank_name: Option<String>,
    #[serde(default)]
    pub statement_account_number: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub opening_balance: Option<i64>,
    #[serde(default)]
    pub closing_balance: Option<i64>,
    #[serde(default)]
    pub rows: Vec<ParsedStatementRow>,
}

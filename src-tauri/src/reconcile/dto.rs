use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PostingReconcileView {
    pub posting_id: String,
    pub entry_id: String,
    pub date: String,
    pub description: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub reconciled: String,
    pub reconciled_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ReconciliationStatusView {
    pub account_id: String,
    pub reconciled_balance: i64,
    pub cleared_balance: i64,
    pub uncleared_balance: i64,
    pub total_balance: i64,
    pub uncleared_postings: Vec<PostingReconcileView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ReconcileRule {
    pub id: String,
    pub pattern: String,
    pub is_regex: bool,
    pub match_type: String,
    pub account_id: String,
    pub priority: i64,
    pub description_override: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ReconcileRuleWithAccount {
    pub id: String,
    pub pattern: String,
    pub is_regex: bool,
    pub match_type: String,
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub priority: i64,
    pub description_override: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CreateReconcileRuleInput {
    pub pattern: String,
    pub is_regex: bool,
    pub match_type: Option<String>,
    pub account_id: String,
    pub priority: Option<i64>,
    pub description_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct MatchedRuleSuggestion {
    pub rule_id: String,
    pub pattern: String,
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
    pub description_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct StatementRow {
    pub date: String,
    pub amount: i64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub rule_match: Option<MatchedRuleSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct MatchResult {
    pub statement_index: usize,
    pub posting_id: String,
    pub matched_amount: i64,
    pub confidence: u8,
    pub days_diff: i64,
    pub matched_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct MatchStatementOutput {
    pub matches: Vec<MatchResult>,
    pub unmatched_statement_indices: Vec<usize>,
    pub unmatched_statement_rows: Vec<StatementRow>,
    pub matched_count: usize,
    pub unmatched_count: usize,
}

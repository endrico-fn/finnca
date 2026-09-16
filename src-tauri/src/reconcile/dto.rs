use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationStatusView {
    pub account_id: String,
    pub reconciled_balance: i64,
    pub cleared_balance: i64,
    pub uncleared_balance: i64,
    pub total_balance: i64,
    pub uncleared_postings: Vec<PostingReconcileView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementRow {
    pub date: String,
    pub amount: i64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub statement_index: usize,
    pub posting_id: String,
    pub matched_amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchStatementOutput {
    pub matches: Vec<MatchResult>,
    pub unmatched_statement_indices: Vec<usize>,
    pub matched_count: usize,
    pub unmatched_count: usize,
}

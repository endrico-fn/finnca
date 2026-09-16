use super::dto::{MatchResult, MatchStatementOutput, PostingReconcileView, StatementRow};
use chrono::NaiveDate;
use std::collections::HashSet;

pub fn days_diff(d1: &str, d2: &str) -> Option<i64> {
    let date1 = NaiveDate::parse_from_str(d1.trim(), "%Y-%m-%d").ok()?;
    let date2 = NaiveDate::parse_from_str(d2.trim(), "%Y-%m-%d").ok()?;
    Some((date1 - date2).num_days().abs())
}

pub fn match_statements_against_postings(
    statements: &[StatementRow],
    postings: &[PostingReconcileView],
    max_day_diff: i64,
) -> MatchStatementOutput {
    let mut matched_posting_ids = HashSet::new();
    let mut matches = Vec::new();
    let mut unmatched_statement_indices = Vec::new();

    for (idx, stmt) in statements.iter().enumerate() {
        let mut found = false;

        for p in postings {
            if matched_posting_ids.contains(&p.posting_id) {
                continue;
            }

            if p.reconciled == "y" {
                continue;
            }

            let amt_match = stmt.amount.abs() == p.amount.abs();
            let date_match =
                days_diff(&stmt.date, &p.date).is_some_and(|diff| diff <= max_day_diff);

            if amt_match && date_match {
                matched_posting_ids.insert(p.posting_id.clone());
                matches.push(MatchResult {
                    statement_index: idx,
                    posting_id: p.posting_id.clone(),
                    matched_amount: p.amount,
                });
                found = true;
                break;
            }
        }

        if !found {
            unmatched_statement_indices.push(idx);
        }
    }

    let matched_count = matches.len();
    let unmatched_count = unmatched_statement_indices.len();

    MatchStatementOutput {
        matches,
        unmatched_statement_indices,
        matched_count,
        unmatched_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_logic() {
        let stmts = vec![
            StatementRow {
                date: "2025-01-10".into(),
                amount: 150000,
                description: Some("Coffee".into()),
            },
            StatementRow {
                date: "2025-01-12".into(),
                amount: 500000,
                description: Some("Groceries".into()),
            },
        ];

        let postings = vec![
            PostingReconcileView {
                posting_id: "post_1".into(),
                entry_id: "entry_1".into(),
                date: "2025-01-11".into(),
                description: "Coffee Shop".into(),
                amount: -150000,
                memo: None,
                reconciled: "n".into(),
                reconciled_at: None,
            },
            PostingReconcileView {
                posting_id: "post_2".into(),
                entry_id: "entry_2".into(),
                date: "2025-01-20".into(), // too far away
                description: "Groceries".into(),
                amount: -500000,
                memo: None,
                reconciled: "n".into(),
                reconciled_at: None,
            },
        ];

        let res = match_statements_against_postings(&stmts, &postings, 4);
        assert_eq!(res.matched_count, 1);
        assert_eq!(res.unmatched_count, 1);
        assert_eq!(res.matches[0].posting_id, "post_1");
        assert_eq!(res.unmatched_statement_indices, vec![1]);
    }
}

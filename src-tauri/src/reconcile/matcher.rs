use super::dto::{MatchResult, MatchStatementOutput, PostingReconcileView, StatementRow};
use chrono::NaiveDate;
use std::collections::HashSet;

pub fn parse_flexible_date(s: &str) -> Option<NaiveDate> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Try primary ISO first
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Some(d);
    }
    // Try standard financial statement formats (Indonesian/European, US, and variants)
    let formats = [
        "%d/%m/%Y", "%d-%m-%Y", "%Y/%m/%d", "%m/%d/%Y", "%d.%m.%Y", "%Y%m%d",
    ];
    for fmt in formats {
        if let Ok(d) = NaiveDate::parse_from_str(trimmed, fmt) {
            return Some(d);
        }
    }
    None
}

pub fn days_diff(d1: &str, d2: &str) -> Option<i64> {
    let date1 = parse_flexible_date(d1)?;
    let date2 = parse_flexible_date(d2)?;
    Some((date1 - date2).num_days().abs())
}

fn has_description_overlap(stmt_desc: Option<&str>, posting_desc: &str) -> bool {
    let s_desc = match stmt_desc {
        Some(d) if !d.trim().is_empty() => d.to_lowercase(),
        _ => return false,
    };
    let p_desc = posting_desc.to_lowercase();

    let s_words: HashSet<&str> = s_desc
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3)
        .collect();

    for w in p_desc
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3)
    {
        if s_words.contains(w) {
            return true;
        }
    }
    false
}

struct Candidate {
    statement_index: usize,
    posting_id: String,
    matched_amount: i64,
    confidence: u8,
    days_diff: i64,
    matched_description: Option<String>,
}

pub fn match_statements_against_postings(
    statements: &[StatementRow],
    postings: &[PostingReconcileView],
    max_day_diff: i64,
) -> MatchStatementOutput {
    let mut candidates = Vec::new();

    for (stmt_idx, stmt) in statements.iter().enumerate() {
        for p in postings {
            if p.reconciled == "y" {
                continue;
            }

            if stmt.amount != p.amount {
                continue;
            }

            if let Some(diff) = days_diff(&stmt.date, &p.date) {
                if diff <= max_day_diff {
                    let base_score: u8 = match diff {
                        0 => 90,
                        1 => 85,
                        2 => 80,
                        3 => 75,
                        4 => 70,
                        _ => 60,
                    };

                    let text_match =
                        has_description_overlap(stmt.description.as_deref(), &p.description);
                    let confidence = if text_match {
                        (base_score + 10).min(100)
                    } else {
                        base_score
                    };

                    candidates.push(Candidate {
                        statement_index: stmt_idx,
                        posting_id: p.posting_id.clone(),
                        matched_amount: p.amount,
                        confidence,
                        days_diff: diff,
                        matched_description: Some(p.description.clone()),
                    });
                }
            }
        }
    }

    // Sort by confidence DESC, then days_diff ASC to match highest-quality pairs first
    candidates.sort_by(|a, b| {
        b.confidence
            .cmp(&a.confidence)
            .then_with(|| a.days_diff.cmp(&b.days_diff))
    });

    let mut matched_statement_indices = HashSet::new();
    let mut matched_posting_ids = HashSet::new();
    let mut matches = Vec::new();

    for cand in candidates {
        if !matched_statement_indices.contains(&cand.statement_index)
            && !matched_posting_ids.contains(&cand.posting_id)
        {
            matched_statement_indices.insert(cand.statement_index);
            matched_posting_ids.insert(cand.posting_id.clone());

            matches.push(MatchResult {
                statement_index: cand.statement_index,
                posting_id: cand.posting_id,
                matched_amount: cand.matched_amount,
                confidence: cand.confidence,
                days_diff: cand.days_diff,
                matched_description: cand.matched_description,
            });
        }
    }

    let mut unmatched_statement_indices = Vec::new();
    let mut unmatched_statement_rows = Vec::new();

    for (idx, stmt) in statements.iter().enumerate() {
        if !matched_statement_indices.contains(&idx) {
            unmatched_statement_indices.push(idx);
            unmatched_statement_rows.push(stmt.clone());
        }
    }

    let matched_count = matches.len();
    let unmatched_count = unmatched_statement_indices.len();

    MatchStatementOutput {
        matches,
        unmatched_statement_indices,
        unmatched_statement_rows,
        matched_count,
        unmatched_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_logic_with_confidence_and_flexible_dates() {
        let stmts = vec![
            StatementRow {
                date: "10/01/2025".into(), // DD/MM/YYYY Indonesian bank format
                amount: -150000,
                description: Some("Coffee Shop Central".into()),
                rule_match: None,
            },
            StatementRow {
                date: "2025-01-12".into(),
                amount: -500000,
                description: Some("Groceries Supermarket".into()),
                rule_match: None,
            },
        ];

        let postings = vec![
            PostingReconcileView {
                posting_id: "post_1".into(),
                entry_id: "entry_1".into(),
                date: "2025-01-10".into(),
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
        assert_eq!(res.matches[0].confidence, 100); // 90 base + 10 text match
        assert_eq!(res.unmatched_statement_indices, vec![1]);
        assert_eq!(res.unmatched_statement_rows.len(), 1);
        assert_eq!(res.unmatched_statement_rows[0].amount, -500000);
    }

    #[test]
    fn test_directional_mismatch_does_not_match() {
        let stmts = vec![StatementRow {
            date: "2025-01-10".into(),
            amount: 150000, // Deposit (inflow)
            description: Some("Deposit".into()),
            rule_match: None,
        }];

        let postings = vec![PostingReconcileView {
            posting_id: "post_1".into(),
            entry_id: "entry_1".into(),
            date: "2025-01-10".into(),
            description: "Expense".into(),
            amount: -150000, // Outflow
            memo: None,
            reconciled: "n".into(),
            reconciled_at: None,
        }];

        let res = match_statements_against_postings(&stmts, &postings, 4);
        assert_eq!(res.matched_count, 0);
        assert_eq!(res.unmatched_count, 1);
    }
}

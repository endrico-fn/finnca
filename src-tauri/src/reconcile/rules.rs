use super::dto::{
    CreateReconcileRuleInput, MatchedRuleSuggestion, ReconcileRuleWithAccount, StatementRow,
};
use crate::shared::AppError;
use regex::RegexBuilder;
use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn list_rules(conn: &Connection) -> Result<Vec<ReconcileRuleWithAccount>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.pattern, r.is_regex, r.match_type, r.account_id,
                a.code, a.name, a.type, r.priority, r.description_override, r.created_at
         FROM reconcile_rules r
         JOIN accounts a ON a.id = r.account_id
         ORDER BY r.priority DESC, r.created_at ASC;",
    )?;

    let rows = stmt.query_map([], |row| {
        let is_regex_int: i64 = row.get(2)?;
        Ok(ReconcileRuleWithAccount {
            id: row.get(0)?,
            pattern: row.get(1)?,
            is_regex: is_regex_int != 0,
            match_type: row.get(3)?,
            account_id: row.get(4)?,
            account_code: row.get(5)?,
            account_name: row.get(6)?,
            account_type: row.get(7)?,
            priority: row.get(8)?,
            description_override: row.get(9)?,
            created_at: row.get(10)?,
        })
    })?;

    let mut rules = Vec::new();
    for r in rows {
        rules.push(r?);
    }
    Ok(rules)
}

pub fn create_rule(
    conn: &Connection,
    input: &CreateReconcileRuleInput,
) -> Result<ReconcileRuleWithAccount, AppError> {
    let pattern = input.pattern.trim();
    if pattern.is_empty() {
        return Err(AppError::InvalidInput(
            "rule pattern cannot be empty".into(),
        ));
    }

    if input.is_regex {
        if let Err(e) = RegexBuilder::new(pattern).case_insensitive(true).build() {
            return Err(AppError::InvalidInput(format!(
                "invalid regex pattern: {e}"
            )));
        }
    }

    let match_type = input
        .match_type
        .as_deref()
        .unwrap_or("ANY")
        .trim()
        .to_uppercase();
    if !matches!(match_type.as_str(), "ANY" | "INFLOW" | "OUTFLOW") {
        return Err(AppError::InvalidInput(
            "match_type must be 'ANY', 'INFLOW', or 'OUTFLOW'".into(),
        ));
    }

    let mut acc_stmt = conn.prepare("SELECT code, name, type FROM accounts WHERE id = ?1;")?;
    let (account_code, account_name, account_type): (String, String, String) = acc_stmt
        .query_row(params![input.account_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map_err(|_| AppError::NotFound(format!("account id={}", input.account_id)))?;

    let id = ulid::Ulid::new().to_string();
    let priority = input.priority.unwrap_or(0);
    let created_at = now_unix();
    let is_regex_int = if input.is_regex { 1 } else { 0 };

    conn.execute(
        "INSERT INTO reconcile_rules (id, pattern, is_regex, match_type, account_id, priority, description_override, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
        params![
            id,
            pattern,
            is_regex_int,
            match_type,
            input.account_id,
            priority,
            input.description_override,
            created_at,
        ],
    )?;

    Ok(ReconcileRuleWithAccount {
        id,
        pattern: pattern.to_string(),
        is_regex: input.is_regex,
        match_type,
        account_id: input.account_id.clone(),
        account_code,
        account_name,
        account_type,
        priority,
        description_override: input.description_override.clone(),
        created_at,
    })
}

pub fn delete_rule(conn: &Connection, rule_id: &str) -> Result<(), AppError> {
    let affected = conn.execute(
        "DELETE FROM reconcile_rules WHERE id = ?1;",
        params![rule_id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("reconcile rule id={rule_id}")));
    }
    Ok(())
}

pub fn find_matching_rule(
    stmt: &StatementRow,
    rules: &[ReconcileRuleWithAccount],
) -> Option<MatchedRuleSuggestion> {
    let desc = match stmt.description.as_deref() {
        Some(d) if !d.trim().is_empty() => d.trim(),
        _ => return None,
    };

    let desc_lower = desc.to_lowercase();

    for rule in rules {
        if rule.match_type == "INFLOW" && stmt.amount <= 0 {
            continue;
        }
        if rule.match_type == "OUTFLOW" && stmt.amount >= 0 {
            continue;
        }

        let is_match = if rule.is_regex {
            RegexBuilder::new(&rule.pattern)
                .case_insensitive(true)
                .build()
                .map(|re| re.is_match(desc))
                .unwrap_or(false)
        } else {
            desc_lower.contains(&rule.pattern.to_lowercase())
        };

        if is_match {
            return Some(MatchedRuleSuggestion {
                rule_id: rule.id.clone(),
                pattern: rule.pattern.clone(),
                account_id: rule.account_id.clone(),
                account_code: rule.account_code.clone(),
                account_name: rule.account_name.clone(),
                description_override: rule.description_override.clone(),
            });
        }
    }

    None
}

pub fn apply_rules_to_unmatched_statements(
    statements: &mut [StatementRow],
    rules: &[ReconcileRuleWithAccount],
) {
    for stmt in statements.iter_mut() {
        stmt.rule_match = find_matching_rule(stmt, rules);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        run_migrations(&mut conn).unwrap();

        conn.execute(
            "INSERT INTO accounts (id, code, name, type, created_at)
             VALUES ('acc_exp_listrik', '5101', 'Beban Listrik PLN', 'EXPENSE', 1000);",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO accounts (id, code, name, type, created_at)
             VALUES ('acc_inc_bunga', '4102', 'Pendapatan Bunga Bank', 'INCOME', 1000);",
            [],
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_create_list_delete_rule() {
        let conn = setup_test_db();

        let input = CreateReconcileRuleInput {
            pattern: "PLN POSTPAID".into(),
            is_regex: false,
            match_type: Some("OUTFLOW".into()),
            account_id: "acc_exp_listrik".into(),
            priority: Some(10),
            description_override: Some("Pembayaran Tagihan Listrik PLN".into()),
        };

        let created = create_rule(&conn, &input).expect("create rule");
        assert_eq!(created.pattern, "PLN POSTPAID");
        assert_eq!(created.account_code, "5101");
        assert_eq!(created.account_name, "Beban Listrik PLN");
        assert_eq!(created.priority, 10);

        let rules = list_rules(&conn).expect("list rules");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, created.id);

        delete_rule(&conn, &created.id).expect("delete rule");
        let rules_after = list_rules(&conn).expect("list rules after delete");
        assert_eq!(rules_after.len(), 0);
    }

    #[test]
    fn test_rule_matching_priority_and_direction() {
        let rules = vec![
            ReconcileRuleWithAccount {
                id: "rule_1".into(),
                pattern: "PLN".into(),
                is_regex: false,
                match_type: "OUTFLOW".into(),
                account_id: "acc_exp_listrik".into(),
                account_code: "5101".into(),
                account_name: "Beban Listrik PLN".into(),
                account_type: "EXPENSE".into(),
                priority: 10,
                description_override: None,
                created_at: 100,
            },
            ReconcileRuleWithAccount {
                id: "rule_2".into(),
                pattern: "^BUNGA.*".into(),
                is_regex: true,
                match_type: "INFLOW".into(),
                account_id: "acc_inc_bunga".into(),
                account_code: "4102".into(),
                account_name: "Pendapatan Bunga Bank".into(),
                account_type: "INCOME".into(),
                priority: 5,
                description_override: Some("Bunga Tabungan".into()),
                created_at: 200,
            },
        ];

        let mut stmts = vec![
            StatementRow {
                date: "2025-01-01".into(),
                amount: -250000,
                description: Some("TRSF E-BANKING PLN PREPAID 123".into()),
                rule_match: None,
            },
            StatementRow {
                date: "2025-01-02".into(),
                amount: 15400,
                description: Some("BUNGA TABUNGAN GIRO".into()),
                rule_match: None,
            },
            StatementRow {
                date: "2025-01-03".into(),
                amount: 250000,
                description: Some("PLN REFUND".into()),
                rule_match: None,
            },
        ];

        apply_rules_to_unmatched_statements(&mut stmts, &rules);

        assert!(stmts[0].rule_match.is_some());
        let m0 = stmts[0].rule_match.as_ref().unwrap();
        assert_eq!(m0.rule_id, "rule_1");
        assert_eq!(m0.account_id, "acc_exp_listrik");

        assert!(stmts[1].rule_match.is_some());
        let m1 = stmts[1].rule_match.as_ref().unwrap();
        assert_eq!(m1.rule_id, "rule_2");
        assert_eq!(m1.account_id, "acc_inc_bunga");
        assert_eq!(m1.description_override.as_deref(), Some("Bunga Tabungan"));

        assert!(stmts[2].rule_match.is_none());
    }
}

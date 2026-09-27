use super::dto::{
    AccountRunningLedgerItem, CreateJournalEntryInput, DashboardMetricsView, JournalEntryView,
    LedgerTotalsView, PostingInput, UpdateJournalEntryInput, VaultHealthReport,
};
use super::models::{JournalEntry, Posting, ReconcileState};
use super::repository;
use super::validation;
use crate::accounts::repository as accounts_repo;
use crate::audit::{self, models::AuditAction};
use crate::ledger::currency::DEFAULT_FX_RATE;
use crate::shared::{generate_id, AppError};
use rusqlite::Connection;

pub fn post_journal_entry(
    conn: &mut Connection,
    input: CreateJournalEntryInput,
    actor: &str,
) -> Result<JournalEntryView, AppError> {
    // 1. Pure validation
    validation::validate_date_format(&input.date)?;
    let closing_date = repository::get_closing_date(conn)?;
    validation::validate_not_in_locked_period(&input.date, closing_date.as_deref())?;
    validation::validate_postings_balance(&input.postings)?;

    if let Some(fx) = input.fx_rate {
        if fx <= 0 {
            return Err(AppError::InvalidInput(
                "fx_rate must be greater than 0".into(),
            ));
        }
    }

    let description = input.description.trim();
    if description.is_empty() {
        return Err(AppError::InvalidInput(
            "Transaction description cannot be empty".into(),
        ));
    }

    // 2. State & Relational validation
    for posting in &input.postings {
        let acc = accounts_repo::get_by_id(conn, &posting.account_id)?.ok_or_else(|| {
            AppError::NotFound(format!("Account '{}' not found", posting.account_id))
        })?;

        if acc.placeholder {
            return Err(AppError::InvalidInput(format!(
                "Cannot post to placeholder account '{}' ({})",
                acc.name, acc.code
            )));
        }
    }

    let entry_id = input.id.unwrap_or_else(generate_id);
    let now = chrono::Utc::now().timestamp_millis();

    let entry = JournalEntry {
        id: entry_id.clone(),
        date: input.date,
        description: description.to_string(),
        notes: input.notes,
        reference_no: input.reference_no,
        due_date: input.due_date,
        plan_id: input.plan_id,
        currency: input.currency.unwrap_or_else(|| "IDR".to_string()),
        fx_rate: input.fx_rate.unwrap_or(DEFAULT_FX_RATE),
        posted_at: now,
    };

    // 3. Database transaction
    let tx = conn.transaction()?;
    repository::insert_entry(&tx, &entry)?;

    for p in input.postings {
        let posting_id = p.id.unwrap_or_else(generate_id);
        let rec = p.reconcile.unwrap_or(ReconcileState::None);
        let rec_at = if rec == ReconcileState::Reconciled {
            Some(now)
        } else {
            None
        };
        let posting = Posting {
            id: posting_id,
            entry_id: entry_id.clone(),
            account_id: p.account_id,
            amount: p.amount,
            memo: p.memo,
            action: p.action,
            reconcile: rec,
            reconciled_at: rec_at,
            currency: p.currency.unwrap_or_else(|| entry.currency.clone()),
            fx_rate: p.fx_rate.or(Some(entry.fx_rate)),
            cost_amount: p.cost_amount,
        };
        repository::insert_posting(&tx, &posting)?;
    }

    let _ = audit::record(
        &tx,
        actor,
        AuditAction::PostJournalEntry,
        "JOURNAL_ENTRY",
        &entry_id,
        Some(&format!("Posted '{}'", entry.description)),
    );

    tx.commit()?;

    repository::get_entry_by_id(conn, &entry_id)?
        .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
}

pub fn update_journal_entry(
    conn: &mut Connection,
    id: &str,
    input: UpdateJournalEntryInput,
    actor: &str,
) -> Result<JournalEntryView, AppError> {
    let existing = repository::get_entry_by_id(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("Journal entry '{id}' not found")))?;

    let closing_date = repository::get_closing_date(conn)?;
    validation::validate_not_in_locked_period(&existing.date, closing_date.as_deref())?;

    if let Some(date) = &input.date {
        validation::validate_date_format(date)?;
        validation::validate_not_in_locked_period(date, closing_date.as_deref())?;
    }

    if let Some(fx) = input.fx_rate {
        if fx <= 0 {
            return Err(AppError::InvalidInput(
                "fx_rate must be greater than 0".into(),
            ));
        }
    }

    let target_currency = input.currency.as_deref().unwrap_or(&existing.currency);
    let target_fx = input.fx_rate.unwrap_or(existing.fx_rate);

    if let Some(postings) = &input.postings {
        validation::validate_postings_balance_with_context(postings, target_currency, target_fx)?;
        for posting in postings {
            let acc = accounts_repo::get_by_id(conn, &posting.account_id)?.ok_or_else(|| {
                AppError::NotFound(format!("Account '{}' not found", posting.account_id))
            })?;

            if acc.placeholder {
                return Err(AppError::InvalidInput(format!(
                    "Cannot post to placeholder account '{}' ({})",
                    acc.name, acc.code
                )));
            }
        }
    } else if input.currency.is_some() || input.fx_rate.is_some() {
        let existing_inputs: Vec<PostingInput> = existing
            .postings
            .iter()
            .map(|p| PostingInput {
                id: Some(p.id.clone()),
                account_id: p.account_id.clone(),
                amount: p.amount,
                memo: p.memo.clone(),
                action: p.action.clone(),
                reconcile: Some(p.reconcile),
                currency: Some(p.currency.clone()),
                fx_rate: p.fx_rate,
                cost_amount: p.cost_amount,
            })
            .collect();
        validation::validate_postings_balance_with_context(
            &existing_inputs,
            target_currency,
            target_fx,
        )?;
    }

    let updated_entry = JournalEntry {
        id: id.to_string(),
        date: input.date.unwrap_or(existing.date),
        description: input
            .description
            .unwrap_or(existing.description)
            .trim()
            .to_string(),
        notes: match input.notes {
            Some(n) => n,
            None => existing.notes,
        },
        reference_no: match input.reference_no {
            Some(r) => r,
            None => existing.reference_no,
        },
        due_date: match input.due_date {
            Some(d) => d,
            None => existing.due_date,
        },
        plan_id: match input.plan_id {
            Some(p) => p,
            None => existing.plan_id,
        },
        currency: input.currency.unwrap_or(existing.currency),
        fx_rate: input.fx_rate.unwrap_or(existing.fx_rate),
        posted_at: existing.posted_at,
    };

    let tx = conn.transaction()?;
    repository::update_entry(&tx, &updated_entry)?;

    if let Some(new_postings) = input.postings {
        use std::collections::HashMap;
        let existing_map: HashMap<&str, &super::dto::PostingView> = existing
            .postings
            .iter()
            .map(|p| (p.id.as_str(), p))
            .collect();

        for ep in &existing.postings {
            if ep.reconcile == ReconcileState::Reconciled {
                let np_opt = new_postings
                    .iter()
                    .find(|np| np.id.as_deref() == Some(&ep.id));
                match np_opt {
                    None => {
                        return Err(AppError::InvalidInput(format!(
                            "Cannot remove reconciled posting '{}'. Unreconcile it first.",
                            ep.id
                        )));
                    }
                    Some(np) => {
                        let rec = np.reconcile.unwrap_or(ep.reconcile);
                        if rec == ReconcileState::Reconciled
                            && (np.amount != ep.amount || np.account_id != ep.account_id)
                        {
                            return Err(AppError::InvalidInput(format!(
                                "Cannot modify amount or account of reconciled posting '{}'. Unreconcile it first.",
                                ep.id
                            )));
                        }
                    }
                }
            }
        }

        repository::delete_postings_by_entry(&tx, id)?;
        let now = chrono::Utc::now().timestamp_millis();

        for p in new_postings {
            let posting_id = p.id.unwrap_or_else(generate_id);
            let matched_ep = existing_map.get(posting_id.as_str()).copied();

            let rec = p
                .reconcile
                .or_else(|| matched_ep.map(|ep| ep.reconcile))
                .unwrap_or(ReconcileState::None);

            let rec_at = if rec == ReconcileState::Reconciled {
                matched_ep.and_then(|ep| ep.reconciled_at).or(Some(now))
            } else {
                None
            };

            let posting_currency = p
                .currency
                .or_else(|| matched_ep.map(|ep| ep.currency.clone()))
                .unwrap_or_else(|| updated_entry.currency.clone());

            let posting_fx = p
                .fx_rate
                .or_else(|| matched_ep.and_then(|ep| ep.fx_rate))
                .or(Some(updated_entry.fx_rate));

            let posting_cost = p
                .cost_amount
                .or_else(|| matched_ep.and_then(|ep| ep.cost_amount));

            let posting = Posting {
                id: posting_id,
                entry_id: id.to_string(),
                account_id: p.account_id,
                amount: p.amount,
                memo: p.memo,
                action: p.action,
                reconcile: rec,
                reconciled_at: rec_at,
                currency: posting_currency,
                fx_rate: posting_fx,
                cost_amount: posting_cost,
            };
            repository::insert_posting(&tx, &posting)?;
        }
    }

    let _ = audit::record(
        &tx,
        actor,
        AuditAction::UpdateJournalEntry,
        "JOURNAL_ENTRY",
        id,
        Some(&format!("Updated '{}'", updated_entry.description)),
    );

    tx.commit()?;

    repository::get_entry_by_id(conn, id)?
        .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))
}

pub fn delete_journal_entry(conn: &mut Connection, id: &str, actor: &str) -> Result<(), AppError> {
    let existing = repository::get_entry_by_id(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("Journal entry '{id}' not found")))?;

    let closing_date = repository::get_closing_date(conn)?;
    validation::validate_not_in_locked_period(&existing.date, closing_date.as_deref())?;

    let has_reconciled = existing
        .postings
        .iter()
        .any(|p| p.reconcile == ReconcileState::Reconciled);
    if has_reconciled {
        return Err(AppError::InvalidInput(
            "Cannot delete a journal entry with reconciled postings. Unreconcile the entry first."
                .into(),
        ));
    }

    let tx = conn.transaction()?;
    repository::delete_entry(&tx, id)?;
    let _ = audit::record(
        &tx,
        actor,
        AuditAction::DeleteJournalEntry,
        "JOURNAL_ENTRY",
        id,
        Some(&format!("Deleted '{}'", existing.description)),
    );
    tx.commit()?;
    Ok(())
}

pub fn get_journal_entry(conn: &Connection, id: &str) -> Result<JournalEntryView, AppError> {
    repository::get_entry_by_id(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("Journal entry '{id}' not found")))
}

pub fn list_journal_entries(
    conn: &Connection,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<JournalEntryView>, AppError> {
    repository::list_entries(conn, limit, offset)
}

pub fn get_account_running_ledger(
    conn: &Connection,
    account_id: &str,
) -> Result<Vec<AccountRunningLedgerItem>, AppError> {
    repository::get_account_running_ledger(conn, account_id)
}

pub fn get_totals(conn: &Connection, fx_rate: Option<i64>) -> Result<LedgerTotalsView, AppError> {
    let fx = fx_rate.unwrap_or(DEFAULT_FX_RATE);
    repository::calculate_totals(conn, fx)
}

pub fn get_dashboard_metrics(
    conn: &Connection,
    fx_rate: Option<i64>,
    today: Option<String>,
) -> Result<DashboardMetricsView, AppError> {
    let fx = fx_rate.unwrap_or(DEFAULT_FX_RATE);
    let today_str = today.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    repository::calculate_dashboard_metrics(conn, fx, &today_str)
}

pub fn diagnose_vault_health(conn: &Connection) -> Result<VaultHealthReport, AppError> {
    let mut issues = Vec::new();

    // 1. SQLite PRAGMA integrity_check
    let sqlite_integrity: String = conn.query_row("PRAGMA integrity_check;", [], |r| r.get(0))?;
    if sqlite_integrity != "ok" {
        issues.push(format!("SQLite integrity error: {}", sqlite_integrity));
    }

    // 2. PRAGMA foreign_key_check
    let mut fk_stmt = conn.prepare("PRAGMA foreign_key_check;")?;
    let mut fk_issues = 0;
    let fk_rows = fk_stmt.query_map([], |r| {
        let table: String = r.get(0)?;
        let rowid: i64 = r.get(1)?;
        let target: String = r.get(2)?;
        Ok(format!(
            "Foreign key violation in table '{}' rowid {} -> target '{}'",
            table, rowid, target
        ))
    })?;
    for row in fk_rows {
        issues.push(row?);
        fk_issues += 1;
    }
    let foreign_keys_ok = fk_issues == 0;

    // 3. Unbalanced journal entries (sum(amount) != 0)
    let mut unbal_stmt = conn.prepare(
        "SELECT je.id, je.date, je.description, SUM(p.amount) as net
         FROM journal_entries je
         JOIN postings p ON p.entry_id = je.id
         GROUP BY je.id
         HAVING net != 0;",
    )?;
    let mut unbalanced_entries_count = 0;
    let unbal_rows = unbal_stmt.query_map([], |r| {
        let id: String = r.get(0)?;
        let date: String = r.get(1)?;
        let desc: String = r.get(2)?;
        let net: i64 = r.get(3)?;
        Ok(format!(
            "Unbalanced entry '{}' ({}, '{}') has sum {}",
            id, date, desc, net
        ))
    })?;
    for row in unbal_rows {
        issues.push(row?);
        unbalanced_entries_count += 1;
    }

    // 4. Orphan postings without journal_entries
    let mut orphan_stmt = conn.prepare(
        "SELECT p.id, p.entry_id FROM postings p
         LEFT JOIN journal_entries je ON je.id = p.entry_id
         WHERE je.id IS NULL;",
    )?;
    let mut orphan_postings_count = 0;
    let orphan_rows = orphan_stmt.query_map([], |r| {
        let pid: String = r.get(0)?;
        let eid: String = r.get(1)?;
        Ok(format!(
            "Orphan posting '{}' references missing entry '{}'",
            pid, eid
        ))
    })?;
    for row in orphan_rows {
        issues.push(row?);
        orphan_postings_count += 1;
    }

    // 5. Postings targeting placeholder accounts
    let mut ph_stmt = conn.prepare(
        "SELECT p.id, a.code, a.name FROM postings p
         JOIN accounts a ON a.id = p.account_id
         WHERE a.placeholder = 1;",
    )?;
    let mut placeholder_postings_count = 0;
    let ph_rows = ph_stmt.query_map([], |r| {
        let pid: String = r.get(0)?;
        let code: String = r.get(1)?;
        let name: String = r.get(2)?;
        Ok(format!(
            "Posting '{}' is on placeholder account '{}' ({})",
            pid, name, code
        ))
    })?;
    for row in ph_rows {
        issues.push(row?);
        placeholder_postings_count += 1;
    }

    let is_healthy = issues.is_empty();

    Ok(VaultHealthReport {
        is_healthy,
        sqlite_integrity,
        foreign_keys_ok,
        unbalanced_entries_count,
        orphan_postings_count,
        placeholder_postings_count,
        issues,
    })
}

pub fn get_closing_date(conn: &Connection) -> Result<Option<String>, AppError> {
    repository::get_closing_date(conn)
}

pub fn set_closing_date(
    conn: &Connection,
    closing_date: Option<&str>,
    actor: &str,
) -> Result<(), AppError> {
    if let Some(date) = closing_date {
        let date = date.trim();
        if !date.is_empty() {
            validation::validate_date_format(date)?;
        }
    }

    repository::set_closing_date(conn, closing_date)?;

    let detail = match closing_date {
        Some(d) if !d.trim().is_empty() => format!("Closed books through {d}"),
        _ => "Unlocked books (cleared closing date)".to_string(),
    };

    let _ = audit::record(
        conn,
        actor,
        AuditAction::UpdateClosingDate,
        "BOOK_CLOSING",
        "1",
        Some(&detail),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::dto::CreateAccountInput;
    use crate::accounts::models::AccountType;
    use crate::ledger::dto::PostingInput;

    #[test]
    fn test_diagnose_vault_health_on_clean_db() {
        let mut conn = Connection::open_in_memory().expect("open db");
        crate::db::schema::run_migrations(&mut conn).expect("run migrations");
        let report = diagnose_vault_health(&conn).expect("diagnose");
        assert!(report.is_healthy);
        assert_eq!(report.sqlite_integrity, "ok");
        assert!(report.foreign_keys_ok);
        assert_eq!(report.unbalanced_entries_count, 0);
        assert_eq!(report.orphan_postings_count, 0);
        assert_eq!(report.placeholder_postings_count, 0);
        assert!(report.issues.is_empty());
    }

    #[test]
    fn test_closing_date_service_and_locked_period_enforcement() {
        let mut conn = Connection::open_in_memory().expect("open db");
        crate::db::schema::run_migrations(&mut conn).expect("run migrations");

        // Seed 2 active accounts
        let acc1 = crate::accounts::service::create_account(
            &mut conn,
            CreateAccountInput {
                code: "1101".into(),
                name: "Kas Operasional".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                currency: Some("IDR".into()),
                placeholder: Some(false),
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "tester",
        )
        .expect("create acc1");

        let acc2 = crate::accounts::service::create_account(
            &mut conn,
            CreateAccountInput {
                code: "5101".into(),
                name: "Beban Operasional".into(),
                account_type: AccountType::Expense,
                parent_id: None,
                currency: Some("IDR".into()),
                placeholder: Some(false),
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "tester",
        )
        .expect("create acc2");

        // 1. Initial closing date is None
        assert_eq!(get_closing_date(&conn).unwrap(), None);

        // 2. Post a transaction on 2025-10-10 when period is open
        let entry = post_journal_entry(
            &mut conn,
            CreateJournalEntryInput {
                id: None,
                date: "2025-10-10".into(),
                description: "Biaya Operasional Okt".into(),
                notes: None,
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                postings: vec![
                    PostingInput {
                        id: None,
                        account_id: acc1.id.clone(),
                        amount: -500_000,
                        memo: None,
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                    PostingInput {
                        id: None,
                        account_id: acc2.id.clone(),
                        amount: 500_000,
                        memo: None,
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                ],
            },
            "tester",
        )
        .expect("post open entry");

        // 3. Set closing date to 2025-12-31
        set_closing_date(&conn, Some("2025-12-31"), "tester").expect("set lock date");
        assert_eq!(
            get_closing_date(&conn).unwrap(),
            Some("2025-12-31".to_string())
        );

        // 4. Try posting new entry on or before 2025-12-31 -> Rejected with ERR_PERIOD_LOCKED
        let post_err = post_journal_entry(
            &mut conn,
            CreateJournalEntryInput {
                id: None,
                date: "2025-12-31".into(),
                description: "Posting at lock date".into(),
                notes: None,
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                postings: vec![
                    PostingInput {
                        id: None,
                        account_id: acc1.id.clone(),
                        amount: -100,
                        memo: None,
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                    PostingInput {
                        id: None,
                        account_id: acc2.id.clone(),
                        amount: 100,
                        memo: None,
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                ],
            },
            "tester",
        )
        .unwrap_err();
        assert_eq!(post_err.code(), "ERR_PERIOD_LOCKED");

        // 5. Try updating existing entry on 2025-10-10 -> Rejected with ERR_PERIOD_LOCKED
        let update_err = update_journal_entry(
            &mut conn,
            &entry.id,
            UpdateJournalEntryInput {
                date: None,
                description: Some("Attempted update".into()),
                notes: None,
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: None,
                fx_rate: None,
                postings: None,
            },
            "tester",
        )
        .unwrap_err();
        assert_eq!(update_err.code(), "ERR_PERIOD_LOCKED");

        // 6. Try deleting existing entry on 2025-10-10 -> Rejected with ERR_PERIOD_LOCKED
        let delete_err = delete_journal_entry(&mut conn, &entry.id, "tester").unwrap_err();
        assert_eq!(delete_err.code(), "ERR_PERIOD_LOCKED");

        // 7. Post new entry on 2026-01-01 (after lock date) -> Succeeds!
        let new_entry = post_journal_entry(
            &mut conn,
            CreateJournalEntryInput {
                id: None,
                date: "2026-01-01".into(),
                description: "Biaya Jan 2026".into(),
                notes: None,
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                postings: vec![
                    PostingInput {
                        id: None,
                        account_id: acc1.id.clone(),
                        amount: -150_000,
                        memo: None,
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                    PostingInput {
                        id: None,
                        account_id: acc2.id.clone(),
                        amount: 150_000,
                        memo: None,
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                ],
            },
            "tester",
        )
        .expect("post open entry 2026");
        assert_eq!(new_entry.date, "2026-01-01");

        // 8. Try updating new 2026-01-01 entry with date back to 2025-11-01 -> Rejected with ERR_PERIOD_LOCKED
        let backdate_err = update_journal_entry(
            &mut conn,
            &new_entry.id,
            UpdateJournalEntryInput {
                date: Some("2025-11-01".into()),
                description: None,
                notes: None,
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: None,
                fx_rate: None,
                postings: None,
            },
            "tester",
        )
        .unwrap_err();
        assert_eq!(backdate_err.code(), "ERR_PERIOD_LOCKED");

        // 9. Clear closing date
        set_closing_date(&conn, None, "tester").expect("clear lock date");
        assert_eq!(get_closing_date(&conn).unwrap(), None);

        // 10. Now delete the 2025-10-10 entry succeeds
        delete_journal_entry(&mut conn, &entry.id, "tester").expect("delete after unlock");
    }

    #[test]
    fn test_reconciled_integrity_and_delete_protection() {
        let mut conn = Connection::open_in_memory().expect("open db");
        crate::db::schema::run_migrations(&mut conn).expect("run migrations");

        let acc1 = crate::accounts::service::create_account(
            &mut conn,
            CreateAccountInput {
                code: "1101".into(),
                name: "Kas".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                currency: Some("IDR".into()),
                placeholder: Some(false),
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "tester",
        )
        .expect("create acc1");

        let acc2 = crate::accounts::service::create_account(
            &mut conn,
            CreateAccountInput {
                code: "5101".into(),
                name: "Beban".into(),
                account_type: AccountType::Expense,
                parent_id: None,
                currency: Some("IDR".into()),
                placeholder: Some(false),
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "tester",
        )
        .expect("create acc2");

        let entry = post_journal_entry(
            &mut conn,
            CreateJournalEntryInput {
                id: None,
                date: "2026-03-01".into(),
                description: "Test Reconciled".into(),
                notes: None,
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                postings: vec![
                    PostingInput {
                        id: None,
                        account_id: acc1.id.clone(),
                        amount: -250_000,
                        memo: None,
                        action: None,
                        reconcile: Some(ReconcileState::Reconciled),
                        ..Default::default()
                    },
                    PostingInput {
                        id: None,
                        account_id: acc2.id.clone(),
                        amount: 250_000,
                        memo: None,
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                ],
            },
            "tester",
        )
        .expect("post reconciled entry");

        let p1 = entry
            .postings
            .iter()
            .find(|p| p.account_id == acc1.id)
            .unwrap();
        assert_eq!(p1.reconcile, ReconcileState::Reconciled);
        let orig_reconciled_at = p1.reconciled_at;
        assert!(orig_reconciled_at.is_some());

        // 1. Delete must be blocked
        let del_err = delete_journal_entry(&mut conn, &entry.id, "tester").unwrap_err();
        assert!(del_err.to_string().contains("reconciled postings"));

        // 2. Modifying reconciled posting amount must be blocked
        let update_amount_err = update_journal_entry(
            &mut conn,
            &entry.id,
            UpdateJournalEntryInput {
                date: None,
                description: None,
                notes: None,
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: None,
                fx_rate: None,
                postings: Some(vec![
                    PostingInput {
                        id: Some(p1.id.clone()),
                        account_id: acc1.id.clone(),
                        amount: -300_000,
                        memo: None,
                        action: None,
                        reconcile: Some(ReconcileState::Reconciled),
                        ..Default::default()
                    },
                    PostingInput {
                        id: None,
                        account_id: acc2.id.clone(),
                        amount: 300_000,
                        memo: None,
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                ]),
            },
            "tester",
        )
        .unwrap_err();
        assert!(update_amount_err.to_string().contains("reconciled posting"));

        // 3. Modifying harmless fields (description, memo) must succeed and preserve reconciled_at
        let updated = update_journal_entry(
            &mut conn,
            &entry.id,
            UpdateJournalEntryInput {
                date: None,
                description: Some("Updated Desc".into()),
                notes: Some(Some("Note".into())),
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: None,
                fx_rate: None,
                postings: Some(vec![
                    PostingInput {
                        id: Some(p1.id.clone()),
                        account_id: acc1.id.clone(),
                        amount: -250_000,
                        memo: Some("Updated memo".into()),
                        action: None,
                        reconcile: Some(ReconcileState::Reconciled),
                        ..Default::default()
                    },
                    PostingInput {
                        id: None,
                        account_id: acc2.id.clone(),
                        amount: 250_000,
                        memo: None,
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                ]),
            },
            "tester",
        )
        .expect("update harmless fields");
        assert_eq!(updated.description, "Updated Desc");
        let updated_p1 = updated.postings.iter().find(|p| p.id == p1.id).unwrap();
        assert_eq!(updated_p1.reconciled_at, orig_reconciled_at);

        // 4. Unreconciling allows deletion
        let unreconciled = update_journal_entry(
            &mut conn,
            &entry.id,
            UpdateJournalEntryInput {
                date: None,
                description: None,
                notes: None,
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: None,
                fx_rate: None,
                postings: Some(vec![
                    PostingInput {
                        id: Some(p1.id.clone()),
                        account_id: acc1.id.clone(),
                        amount: -250_000,
                        memo: None,
                        action: None,
                        reconcile: Some(ReconcileState::None),
                        ..Default::default()
                    },
                    PostingInput {
                        id: None,
                        account_id: acc2.id.clone(),
                        amount: 250_000,
                        memo: None,
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                ]),
            },
            "tester",
        )
        .expect("unreconcile");
        let unreconciled_p1 = unreconciled
            .postings
            .iter()
            .find(|p| p.id == p1.id)
            .unwrap();
        assert_eq!(unreconciled_p1.reconcile, ReconcileState::None);
        assert!(unreconciled_p1.reconciled_at.is_none());

        delete_journal_entry(&mut conn, &entry.id, "tester").expect("delete after unreconciling");
    }

    #[test]
    fn test_dashboard_metrics_multicurrency_conversion() {
        let mut conn = Connection::open_in_memory().expect("open db");
        crate::db::schema::run_migrations(&mut conn).expect("run migrations");

        let acc_bank = crate::accounts::service::create_account(
            &mut conn,
            CreateAccountInput {
                code: "1001".into(),
                name: "USD Bank".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                currency: Some("USD".into()),
                placeholder: Some(false),
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "tester",
        )
        .unwrap();

        let acc_saas = crate::accounts::service::create_account(
            &mut conn,
            CreateAccountInput {
                code: "5001".into(),
                name: "SaaS Expense".into(),
                account_type: AccountType::Expense,
                parent_id: None,
                currency: Some("USD".into()),
                placeholder: Some(false),
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "tester",
        )
        .unwrap();

        post_journal_entry(
            &mut conn,
            CreateJournalEntryInput {
                id: None,
                date: "2026-09-15".into(),
                description: "SaaS Subscription".into(),
                notes: None,
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: Some("USD".into()),
                fx_rate: Some(16000),
                postings: vec![
                    PostingInput {
                        id: None,
                        account_id: acc_saas.id,
                        amount: 10000,
                        memo: None,
                        action: None,
                        reconcile: None,
                        currency: Some("USD".into()),
                        fx_rate: Some(16000),
                        cost_amount: None,
                    },
                    PostingInput {
                        id: None,
                        account_id: acc_bank.id,
                        amount: -10000,
                        memo: None,
                        action: None,
                        reconcile: None,
                        currency: Some("USD".into()),
                        fx_rate: Some(16000),
                        cost_amount: None,
                    },
                ],
            },
            "tester",
        )
        .unwrap();

        let metrics = get_dashboard_metrics(&conn, Some(16000), Some("2026-09-20".into())).unwrap();
        assert_eq!(metrics.monthly_burn_30d, 1_600_000);
        assert_eq!(metrics.this_month_expense, 1_600_000);
    }
}

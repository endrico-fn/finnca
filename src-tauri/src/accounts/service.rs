use super::dto::{AccountBalanceView, CreateAccountInput, UpdateAccountInput};
use super::models::Account;
use super::repository;
use crate::audit::{self, models::AuditAction};
use crate::shared::{generate_id, AppError};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

pub fn create_account(
    conn: &Connection,
    input: CreateAccountInput,
    actor: &str,
) -> Result<Account, AppError> {
    let code = input.code.trim();
    if code.is_empty() {
        return Err(AppError::InvalidInput(
            "Account code cannot be empty".into(),
        ));
    }
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput(
            "Account name cannot be empty".into(),
        ));
    }

    if repository::get_by_code(conn, code)?.is_some() {
        return Err(AppError::Conflict(format!(
            "Account with code '{code}' already exists"
        )));
    }

    if let Some(parent_id) = &input.parent_id {
        let parent = repository::get_by_id(conn, parent_id)?.ok_or_else(|| {
            AppError::NotFound(format!("Parent account '{parent_id}' does not exist"))
        })?;
        if !parent.placeholder {
            return Err(AppError::Conflict(format!(
                "Parent account '{}' ({}) is postable; convert it to placeholder first",
                parent.name, parent.code
            )));
        }
        let parent_depth = calculate_ancestor_depth(conn, parent_id)?;
        if parent_depth + 1 > MAX_ACCOUNT_HIERARCHY_DEPTH {
            return Err(AppError::InvalidInput(format!(
                "Account hierarchy depth cannot exceed {MAX_ACCOUNT_HIERARCHY_DEPTH} levels"
            )));
        }
    }

    let account = Account {
        id: generate_id(),
        code: code.to_string(),
        name: name.to_string(),
        account_type: input.account_type,
        parent_id: input.parent_id,
        currency: input.currency.unwrap_or_else(|| "IDR".to_string()),
        placeholder: input.placeholder.unwrap_or(false),
        hidden: input.hidden.unwrap_or(false),
        color: input.color,
        note: input.note,
        description: input.description,
        interest_rate: input.interest_rate,
        created_at: chrono::Utc::now().timestamp_millis(),
    };

    repository::insert(conn, &account)?;
    let _ = audit::record(
        conn,
        actor,
        AuditAction::CreateAccount,
        "ACCOUNT",
        &account.id,
        Some(&format!(
            "Created account '{}' ({})",
            account.name, account.code
        )),
    );
    Ok(account)
}

pub fn update_account(
    conn: &Connection,
    id: &str,
    input: UpdateAccountInput,
    actor: &str,
) -> Result<Account, AppError> {
    let mut account = repository::get_by_id(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("Account '{id}' not found")))?;

    if let Some(code) = input.code {
        let code_trimmed = code.trim();
        if code_trimmed.is_empty() {
            return Err(AppError::InvalidInput(
                "Account code cannot be empty".into(),
            ));
        }
        if code_trimmed != account.code {
            if repository::get_by_code(conn, code_trimmed)?.is_some() {
                return Err(AppError::Conflict(format!(
                    "Account with code '{code_trimmed}' already exists"
                )));
            }
            account.code = code_trimmed.to_string();
        }
    }

    if let Some(name) = input.name {
        let name_trimmed = name.trim();
        if name_trimmed.is_empty() {
            return Err(AppError::InvalidInput(
                "Account name cannot be empty".into(),
            ));
        }
        account.name = name_trimmed.to_string();
    }

    if let Some(account_type) = input.account_type {
        if account_type != account.account_type && repository::has_postings(conn, id)? {
            return Err(AppError::Conflict(format!(
                "Cannot change type of '{}' ({}) with existing postings",
                account.name, account.code
            )));
        }
        account.account_type = account_type;
    }

    if let Some(parent_update) = input.parent_id {
        match parent_update {
            Some(new_parent_id) => {
                if new_parent_id == id {
                    return Err(AppError::InvalidInput(
                        "An account cannot be its own parent".into(),
                    ));
                }
                let new_parent = repository::get_by_id(conn, &new_parent_id)?.ok_or_else(|| {
                    AppError::NotFound(format!("Parent account '{new_parent_id}' does not exist"))
                })?;

                if is_descendant(conn, id, &new_parent_id)? {
                    return Err(AppError::InvalidInput(
                        "Cannot set an account's own descendant as its parent".into(),
                    ));
                }
                if !new_parent.placeholder {
                    return Err(AppError::Conflict(format!(
                        "Parent account '{}' ({}) is postable; convert it to placeholder first",
                        new_parent.name, new_parent.code
                    )));
                }

                let new_parent_depth = calculate_ancestor_depth(conn, &new_parent_id)?;
                let subtree_depth = get_subtree_depth(conn, id)?;
                if new_parent_depth + subtree_depth > MAX_ACCOUNT_HIERARCHY_DEPTH {
                    return Err(AppError::InvalidInput(format!(
                        "Account hierarchy depth cannot exceed {MAX_ACCOUNT_HIERARCHY_DEPTH} levels"
                    )));
                }

                account.parent_id = Some(new_parent_id);
            }
            None => {
                account.parent_id = None;
            }
        }
    }

    if let Some(currency) = input.currency {
        if currency != account.currency && repository::has_postings(conn, id)? {
            return Err(AppError::Conflict(format!(
                "Cannot change currency of '{}' ({}) with existing postings",
                account.name, account.code
            )));
        }
        account.currency = currency;
    }

    if let Some(placeholder) = input.placeholder {
        if placeholder && !account.placeholder && repository::has_postings(conn, id)? {
            return Err(AppError::Conflict(format!(
                "Cannot convert '{}' ({}) to placeholder with existing postings",
                account.name, account.code
            )));
        }
        if !placeholder && account.placeholder && repository::has_children(conn, id)? {
            return Err(AppError::Conflict(format!(
                "Cannot disable placeholder of '{}' ({}) with existing sub-accounts",
                account.name, account.code
            )));
        }
        account.placeholder = placeholder;
    }

    if let Some(hidden) = input.hidden {
        account.hidden = hidden;
    }

    if let Some(color) = input.color {
        account.color = color;
    }

    if let Some(note) = input.note {
        account.note = note;
    }

    if let Some(description) = input.description {
        account.description = description;
    }

    if let Some(interest_rate) = input.interest_rate {
        account.interest_rate = interest_rate;
    }

    repository::update(conn, &account)?;
    let _ = audit::record(
        conn,
        actor,
        AuditAction::UpdateAccount,
        "ACCOUNT",
        &account.id,
        Some(&format!(
            "Updated account '{}' ({})",
            account.name, account.code
        )),
    );
    Ok(account)
}

pub fn delete_account(conn: &Connection, id: &str, actor: &str) -> Result<(), AppError> {
    let existing = repository::get_by_id(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("Account '{id}' not found")))?;

    if repository::has_children(conn, id)? {
        return Err(AppError::Conflict(
            "Cannot delete account with existing child accounts".into(),
        ));
    }

    if repository::has_postings(conn, id)? {
        return Err(AppError::Conflict(
            "Cannot delete account with ledger postings".into(),
        ));
    }

    repository::delete(conn, id)?;
    let _ = audit::record(
        conn,
        actor,
        AuditAction::DeleteAccount,
        "ACCOUNT",
        id,
        Some(&format!(
            "Deleted account '{}' ({})",
            existing.name, existing.code
        )),
    );
    Ok(())
}

pub fn get_account(conn: &Connection, id: &str) -> Result<Account, AppError> {
    repository::get_by_id(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("Account '{id}' not found")))
}

pub fn list_accounts_with_balances(conn: &Connection) -> Result<Vec<AccountBalanceView>, AppError> {
    let accounts = repository::list_all(conn)?;
    let direct_balances = repository::get_all_direct_balances(conn)?;

    // Build children mapping
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    for acc in &accounts {
        if let Some(parent_id) = &acc.parent_id {
            children_map
                .entry(parent_id.clone())
                .or_default()
                .push(acc.id.clone());
        }
    }

    // Helper closure to compute recursive sum of direct balances
    fn calculate_recursive_balance(
        acc_id: &str,
        direct: &HashMap<String, i64>,
        children: &HashMap<String, Vec<String>>,
    ) -> i64 {
        let mut total = direct.get(acc_id).copied().unwrap_or(0);
        if let Some(kids) = children.get(acc_id) {
            for kid_id in kids {
                total += calculate_recursive_balance(kid_id, direct, children);
            }
        }
        total
    }

    let mut views = Vec::with_capacity(accounts.len());
    for acc in accounts {
        let direct_balance = direct_balances.get(&acc.id).copied().unwrap_or(0);
        let recursive_balance =
            calculate_recursive_balance(&acc.id, &direct_balances, &children_map);

        views.push(AccountBalanceView {
            account: acc,
            direct_balance,
            recursive_balance,
        });
    }

    Ok(views)
}

fn is_descendant(
    conn: &Connection,
    potential_ancestor_id: &str,
    target_id: &str,
) -> Result<bool, AppError> {
    let mut current_id = Some(target_id.to_string());
    let mut visited = HashSet::new();

    while let Some(curr) = current_id {
        if curr == potential_ancestor_id {
            return Ok(true);
        }
        if !visited.insert(curr.clone()) {
            break;
        }
        let acc = repository::get_by_id(conn, &curr)?;
        current_id = acc.and_then(|a| a.parent_id);
    }

    Ok(false)
}

pub const MAX_ACCOUNT_HIERARCHY_DEPTH: usize = 5;

fn calculate_ancestor_depth(conn: &Connection, parent_id: &str) -> Result<usize, AppError> {
    let mut depth = 1;
    let mut current_id = Some(parent_id.to_string());
    let mut visited = HashSet::new();

    while let Some(curr) = current_id {
        if !visited.insert(curr.clone()) {
            return Err(AppError::InvalidInput(
                "Cycle detected in account hierarchy".into(),
            ));
        }
        let acc = repository::get_by_id(conn, &curr)?;
        match acc.and_then(|a| a.parent_id) {
            Some(p) => {
                depth += 1;
                current_id = Some(p);
            }
            None => break,
        }
    }

    Ok(depth)
}

fn get_subtree_depth(conn: &Connection, account_id: &str) -> Result<usize, AppError> {
    let accounts = repository::list_all(conn)?;
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    for acc in &accounts {
        if let Some(parent_id) = &acc.parent_id {
            children_map
                .entry(parent_id.clone())
                .or_default()
                .push(acc.id.clone());
        }
    }

    fn max_depth(id: &str, children_map: &HashMap<String, Vec<String>>) -> usize {
        if let Some(kids) = children_map.get(id) {
            let mut deepest_child = 0;
            for kid in kids {
                let d = max_depth(kid, children_map);
                if d > deepest_child {
                    deepest_child = d;
                }
            }
            1 + deepest_child
        } else {
            1
        }
    }

    Ok(max_depth(account_id, &children_map))
}

pub use super::seeds::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::models::AccountType;
    use crate::db::open_vault_db;

    fn setup_test_db() -> Connection {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let db_path = temp_dir.path().join("test.db");
        let dek = [1u8; 32];
        open_vault_db(&db_path, &dek).expect("open test db")
    }

    #[test]
    fn test_create_and_fetch_account() {
        let conn = setup_test_db();
        let acc = create_account(
            &conn,
            CreateAccountInput {
                code: "1000".into(),
                name: "Cash in Hand".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                currency: Some("IDR".into()),
                placeholder: Some(false),
                hidden: Some(false),
                color: Some("#00ff00".into()),
                note: None,
                description: None,
                interest_rate: None,
            },
            "test",
        )
        .expect("create account");

        assert_eq!(acc.code, "1000");
        assert_eq!(acc.name, "Cash in Hand");
        assert_eq!(acc.account_type, AccountType::Asset);

        let fetched = get_account(&conn, &acc.id).expect("fetch account");
        assert_eq!(fetched, acc);
    }

    #[test]
    fn test_duplicate_code_conflict() {
        let conn = setup_test_db();
        let input = CreateAccountInput {
            code: "2000".into(),
            name: "Accounts Payable".into(),
            account_type: AccountType::Liability,
            parent_id: None,
            currency: None,
            placeholder: None,
            hidden: None,
            color: None,
            note: None,
            description: None,
            interest_rate: None,
        };

        create_account(&conn, input.clone(), "test").expect("first create");
        let err = create_account(&conn, input, "test").expect_err("second create must fail");
        assert_eq!(err.code(), "ERR_CONFLICT");
    }

    #[test]
    fn test_cycle_prevention() {
        let conn = setup_test_db();
        let parent = create_account(
            &conn,
            CreateAccountInput {
                code: "100".into(),
                name: "Parent".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                currency: None,
                placeholder: Some(true),
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "test",
        )
        .unwrap();

        let child = create_account(
            &conn,
            CreateAccountInput {
                code: "101".into(),
                name: "Child".into(),
                account_type: AccountType::Asset,
                parent_id: Some(parent.id.clone()),
                currency: None,
                placeholder: None,
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "test",
        )
        .unwrap();

        // Attempt to make parent's parent = child (cycle)
        let err = update_account(
            &conn,
            &parent.id,
            UpdateAccountInput {
                code: None,
                name: None,
                account_type: None,
                parent_id: Some(Some(child.id)),
                currency: None,
                placeholder: None,
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "test",
        )
        .expect_err("cycle must fail");

        assert_eq!(err.code(), "ERR_INVALID_INPUT");
    }

    #[test]
    fn test_balance_rollups() {
        let conn = setup_test_db();
        let parent = create_account(
            &conn,
            CreateAccountInput {
                code: "1000".into(),
                name: "Current Assets".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                currency: None,
                placeholder: Some(true),
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "test",
        )
        .unwrap();

        let child1 = create_account(
            &conn,
            CreateAccountInput {
                code: "1010".into(),
                name: "Bank BCA".into(),
                account_type: AccountType::Asset,
                parent_id: Some(parent.id.clone()),
                currency: None,
                placeholder: None,
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "test",
        )
        .unwrap();

        let child2 = create_account(
            &conn,
            CreateAccountInput {
                code: "1020".into(),
                name: "Cash Wallet".into(),
                account_type: AccountType::Asset,
                parent_id: Some(parent.id.clone()),
                currency: None,
                placeholder: None,
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "test",
        )
        .unwrap();

        // Add a mock journal entry and postings
        conn.execute(
            "INSERT INTO journal_entries (id, date, description, currency, posted_at) VALUES ('entry_1', '2026-09-14', 'Initial deposit', 'IDR', 1700000000);",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO postings (id, entry_id, account_id, amount) VALUES ('p_1', 'entry_1', ?1, 500000);",
            [&child1.id],
        ).unwrap();
        conn.execute(
            "INSERT INTO postings (id, entry_id, account_id, amount) VALUES ('p_2', 'entry_1', ?1, 300000);",
            [&child2.id],
        ).unwrap();

        let views = list_accounts_with_balances(&conn).unwrap();
        let parent_view = views.iter().find(|v| v.account.id == parent.id).unwrap();
        assert_eq!(parent_view.direct_balance, 0);
        assert_eq!(parent_view.recursive_balance, 800000);

        let child1_view = views.iter().find(|v| v.account.id == child1.id).unwrap();
        assert_eq!(child1_view.direct_balance, 500000);
        assert_eq!(child1_view.recursive_balance, 500000);
    }

    #[test]
    fn test_seed_root_placeholder_accounts() {
        let conn = setup_test_db();

        let seeded = seed_root_placeholder_accounts(&conn, "IDR").expect("seed accounts");
        assert_eq!(seeded.len(), 26);

        let roots: Vec<&Account> = seeded.iter().filter(|a| a.parent_id.is_none()).collect();
        assert_eq!(roots.len(), 5);
        for r in &roots {
            assert!(r.placeholder);
            assert_eq!(r.currency, "IDR");
        }

        let leaves: Vec<&Account> = seeded.iter().filter(|a| !a.placeholder).collect();
        assert_eq!(leaves.len(), 14);
        for l in &leaves {
            assert!(l.parent_id.is_some());
        }

        let views = list_accounts_with_balances(&conn).expect("list accounts");
        assert_eq!(views.len(), 26);

        let reseeded = seed_root_placeholder_accounts(&conn, "IDR").expect("re-seed accounts");
        assert_eq!(reseeded.len(), 0);

        let views_after = list_accounts_with_balances(&conn).expect("list accounts after");
        assert_eq!(views_after.len(), 26);
    }

    #[test]
    fn test_seed_comprehensive_accounts_bilingual() {
        let conn = setup_test_db();

        let seeded_id = seed_comprehensive_accounts(&conn, "id", "IDR").expect("seed id");
        assert_eq!(seeded_id.len(), 26);

        let cash = seeded_id.iter().find(|a| a.code == "1110").unwrap();
        assert_eq!(cash.name, "Dompet Tunai");
        assert!(!cash.placeholder);

        let ob = seeded_id.iter().find(|a| a.code == "3110").unwrap();
        assert_eq!(ob.name, "Saldo Awal");
        assert!(!ob.placeholder);

        let util = seeded_id.iter().find(|a| a.code == "5110").unwrap();
        assert_eq!(util.name, "Listrik, Air & Gas");
        assert!(!util.placeholder);
    }

    #[test]
    fn test_postable_parent_rejected() {
        let conn = setup_test_db();
        let leaf = create_account(
            &conn,
            CreateAccountInput {
                code: "1110".into(),
                name: "Cash".into(),
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
            "test",
        )
        .unwrap();
        let err = create_account(
            &conn,
            CreateAccountInput {
                code: "1111".into(),
                name: "Child".into(),
                account_type: AccountType::Asset,
                parent_id: Some(leaf.id.clone()),
                currency: Some("IDR".into()),
                placeholder: Some(false),
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "test",
        )
        .expect_err("postable parent must fail");
        assert_eq!(err.code(), "ERR_CONFLICT");
    }

    #[test]
    fn test_seed_business_accounts() {
        let conn = setup_test_db();
        let seeded = seed_account_template(&conn, "business", "id", "IDR").expect("seed business");
        assert_eq!(seeded.len(), 26);

        let ar = seeded.iter().find(|a| a.code == "1140").unwrap();
        assert_eq!(ar.name, "Piutang Usaha (AR)");
        assert_eq!(ar.account_type, AccountType::Asset);

        let ap = seeded.iter().find(|a| a.code == "2120").unwrap();
        assert_eq!(ap.name, "Hutang Usaha (AP)");
        assert_eq!(ap.account_type, AccountType::Liability);

        let rev = seeded.iter().find(|a| a.code == "4110").unwrap();
        assert_eq!(rev.name, "Pendapatan Jasa & Proyek");
        assert_eq!(rev.account_type, AccountType::Income);
    }

    #[test]
    fn test_max_hierarchy_depth_enforced() {
        let conn = setup_test_db();
        let mut parent_id = None;

        for i in 1..=5 {
            let acc = create_account(
                &conn,
                CreateAccountInput {
                    code: format!("100{i}"),
                    name: format!("Level {i}"),
                    account_type: AccountType::Asset,
                    parent_id,
                    currency: Some("IDR".into()),
                    placeholder: Some(true),
                    hidden: None,
                    color: None,
                    note: None,
                    description: None,
                    interest_rate: None,
                },
                "test",
            )
            .expect("create level");
            parent_id = Some(acc.id);
        }

        let err = create_account(
            &conn,
            CreateAccountInput {
                code: "1006".into(),
                name: "Level 6".into(),
                account_type: AccountType::Asset,
                parent_id,
                currency: Some("IDR".into()),
                placeholder: Some(false),
                hidden: None,
                color: None,
                note: None,
                description: None,
                interest_rate: None,
            },
            "test",
        )
        .expect_err("level 6 must fail");

        assert_eq!(err.code(), "ERR_INVALID_INPUT");
    }
}

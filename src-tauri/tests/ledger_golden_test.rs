use finnca_lib::accounts::models::{Account, AccountType};
use finnca_lib::accounts::repository as accounts_repo;
use finnca_lib::db::open_vault_db;
use finnca_lib::ledger::dto::{CreateJournalEntryInput, PostingInput};
use finnca_lib::ledger::models::ReconcileState;
use finnca_lib::ledger::service;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct FixtureAccount {
    id: String,
    code: String,
    name: String,
    #[serde(rename = "type")]
    acc_type: String,
    currency: String,
    #[serde(default)]
    placeholder: bool,
    #[serde(rename = "parentId")]
    parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FixtureSplit {
    id: Option<String>,
    #[serde(rename = "accountId")]
    account_id: String,
    amount: i64,
    memo: Option<String>,
    #[serde(default)]
    reconcile: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FixtureTransaction {
    id: String,
    date: String,
    description: String,
    currency: Option<String>,
    #[serde(rename = "fxRateAtTransaction")]
    fx_rate: Option<i64>,
    notes: Option<String>,
    splits: Vec<FixtureSplit>,
}

#[derive(Debug, Deserialize)]
struct FixtureTotals {
    #[serde(rename = "totalAssets")]
    total_assets: i64,
    #[serde(rename = "totalLiabilities")]
    total_liabilities: i64,
    #[serde(rename = "totalEquity")]
    total_equity: i64,
    #[serde(rename = "totalIncome")]
    total_income: i64,
    #[serde(rename = "totalExpenses")]
    total_expenses: i64,
    #[serde(rename = "netIncome")]
    net_income: i64,
    #[serde(rename = "balanceSheetDiscrepancy")]
    balance_sheet_discrepancy: i64,
    #[serde(rename = "isBalanceSheetAligned")]
    is_balance_sheet_aligned: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FixtureLedgerItem {
    #[serde(rename = "txId")]
    tx_id: String,
    date: String,
    amount: i64,
    running: i64,
}

#[derive(Debug, Deserialize)]
struct FixtureRoot {
    #[serde(rename = "scenarioName")]
    scenario_name: String,
    #[serde(rename = "inputVault")]
    input_vault: InputVault,
    #[serde(rename = "expectedOutput")]
    expected_output: ExpectedOutput,
}

#[derive(Debug, Deserialize)]
struct InputVault {
    #[serde(rename = "fxRate")]
    fx_rate: i64,
    accounts: Vec<FixtureAccount>,
    transactions: Vec<FixtureTransaction>,
}

#[derive(Debug, Deserialize)]
struct ExpectedOutput {
    #[serde(rename = "accountBalances")]
    expected_balances: HashMap<String, i64>,
    #[serde(rename = "accountLedgers")]
    account_ledgers: HashMap<String, Vec<FixtureLedgerItem>>,
    totals: FixtureTotals,
}

fn run_golden_scenario(fixture_path: &Path) {
    let raw = fs::read_to_string(fixture_path).expect("read fixture file");
    let fixture: FixtureRoot = serde_json::from_str(&raw).expect("parse fixture json");

    println!("Running golden fixture: {}", fixture.scenario_name);

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let db_path = temp_dir.path().join("golden_vault.db");
    let dek = [77u8; 32];
    let mut conn = open_vault_db(&db_path, &dek).expect("open encrypted test db");

    // 1. Seed Accounts
    for a in fixture.input_vault.accounts {
        let acc_type = AccountType::from_str(&a.acc_type).expect("valid acc type");
        let account = Account {
            id: a.id,
            code: a.code,
            name: a.name,
            account_type: acc_type,
            parent_id: a.parent_id,
            currency: a.currency,
            placeholder: a.placeholder,
            hidden: false,
            color: None,
            note: None,
            description: None,
            interest_rate: None,
            created_at: 1700000000,
        };
        accounts_repo::insert(&conn, &account).expect("insert account");
    }

    // 2. Post Transactions
    for tx in fixture.input_vault.transactions {
        let postings: Vec<PostingInput> = tx
            .splits
            .into_iter()
            .map(|s| {
                let rec = s.reconcile.and_then(|r| ReconcileState::from_str(&r).ok());
                PostingInput {
                    id: s.id,
                    account_id: s.account_id,
                    amount: s.amount,
                    memo: s.memo,
                    action: None,
                    reconcile: rec,
                    ..Default::default()
                }
            })
            .collect();

        service::post_journal_entry(
            &mut conn,
            CreateJournalEntryInput {
                id: Some(tx.id),
                date: tx.date,
                description: tx.description,
                notes: tx.notes,
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: tx.currency,
                fx_rate: tx.fx_rate,
                postings,
            },
            "golden-test",
        )
        .expect("post journal entry");
    }

    // 3. Verify Account Balances
    let accounts_with_balances = finnca_lib::accounts::service::list_accounts_with_balances(&conn)
        .expect("accounts with balances");
    for (acc_id, expected_balance) in &fixture.expected_output.expected_balances {
        let view = accounts_with_balances
            .iter()
            .find(|v| &v.account.id == acc_id)
            .unwrap_or_else(|| panic!("Account '{acc_id}' not found"));
        assert_eq!(
            view.recursive_balance, *expected_balance,
            "Account balance mismatch for account '{}' in scenario '{}'",
            acc_id, fixture.scenario_name
        );
    }

    // 4. Verify Running Ledgers
    for (acc_id, expected_items) in &fixture.expected_output.account_ledgers {
        let actual_ledger =
            service::get_account_running_ledger(&conn, acc_id).expect("account running ledger");
        assert_eq!(
            actual_ledger.len(),
            expected_items.len(),
            "Ledger count mismatch for account '{}' in scenario '{}'",
            acc_id,
            fixture.scenario_name
        );

        for (i, expected_item) in expected_items.iter().enumerate() {
            let actual = &actual_ledger[i];
            assert_eq!(
                actual.entry_id, expected_item.tx_id,
                "Tx ID mismatch at index {} for account '{}'",
                i, acc_id
            );
            assert_eq!(
                actual.amount, expected_item.amount,
                "Amount mismatch at index {} for account '{}'",
                i, acc_id
            );
            assert_eq!(
                actual.running_balance, expected_item.running,
                "Running balance mismatch at index {} for account '{}'",
                i, acc_id
            );
        }
    }

    // 5. Verify Totals
    let actual_totals =
        service::get_totals(&conn, Some(fixture.input_vault.fx_rate)).expect("calculate totals");
    let exp_totals = &fixture.expected_output.totals;

    assert_eq!(
        actual_totals.total_assets, exp_totals.total_assets,
        "Total assets mismatch in scenario '{}'",
        fixture.scenario_name
    );
    assert_eq!(
        actual_totals.total_liabilities, exp_totals.total_liabilities,
        "Total liabilities mismatch in scenario '{}'",
        fixture.scenario_name
    );
    assert_eq!(
        actual_totals.total_equity, exp_totals.total_equity,
        "Total equity mismatch in scenario '{}'",
        fixture.scenario_name
    );
    assert_eq!(
        actual_totals.total_income, exp_totals.total_income,
        "Total income mismatch in scenario '{}'",
        fixture.scenario_name
    );
    assert_eq!(
        actual_totals.total_expenses, exp_totals.total_expenses,
        "Total expenses mismatch in scenario '{}'",
        fixture.scenario_name
    );
    assert_eq!(
        actual_totals.net_income, exp_totals.net_income,
        "Net income mismatch in scenario '{}'",
        fixture.scenario_name
    );
    assert_eq!(
        actual_totals.balance_sheet_discrepancy, exp_totals.balance_sheet_discrepancy,
        "Discrepancy mismatch in scenario '{}'",
        fixture.scenario_name
    );
    assert_eq!(
        actual_totals.is_balance_sheet_aligned, exp_totals.is_balance_sheet_aligned,
        "Alignment mismatch in scenario '{}'",
        fixture.scenario_name
    );
}

#[test]
fn test_golden_01_simple_transfers() {
    run_golden_scenario(Path::new("tests/fixtures/01_simple_transfers.json"));
}

#[test]
fn test_golden_02_multi_split_expenses() {
    run_golden_scenario(Path::new("tests/fixtures/02_multi_split_expenses.json"));
}

#[test]
fn test_golden_03_multicurrency_usd_idr() {
    run_golden_scenario(Path::new("tests/fixtures/03_multicurrency_usd_idr.json"));
}

#[test]
fn test_golden_04_reconciled_uncleared_balances() {
    run_golden_scenario(Path::new(
        "tests/fixtures/04_reconciled_uncleared_balances.json",
    ));
}

#[test]
fn test_golden_05_full_month_statements() {
    run_golden_scenario(Path::new("tests/fixtures/05_full_month_statements.json"));
}

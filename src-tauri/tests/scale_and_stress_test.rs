use finnca_lib::accounts::service as accounts_service;
use finnca_lib::budget::dto::UpsertBudgetInput;
use finnca_lib::budget::service as budget_service;
use finnca_lib::db::open_vault_db;
use finnca_lib::ledger::dto::{CreateJournalEntryInput, PostingInput};
use finnca_lib::ledger::models::ReconcileState;
use finnca_lib::ledger::service as ledger_service;
use finnca_lib::report::generators::{
    balance_sheet, cash_flow, historical_trends, monthly_cashflow, profit_loss, trial_balance,
};
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_scale_and_financial_invariants_2500_transactions() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let db_path = temp_dir.path().join("scale_test_vault.db");
    let dek = [42u8; 32];
    let mut conn = open_vault_db(&db_path, &dek).expect("open encrypted test db");

    // 1. Seed chart of accounts
    let accounts = accounts_service::seed_comprehensive_accounts(&conn, "id", "IDR")
        .expect("seed comprehensive accounts");
    assert!(!accounts.is_empty());

    let mut acct_map: HashMap<String, String> = HashMap::new();
    for a in &accounts {
        acct_map.insert(a.code.clone(), a.id.clone());
    }

    let acc_cash = acct_map.get("1110").expect("Cash").clone();
    let acc_bank = acct_map.get("1120").expect("Bank").clone();
    let acc_savings = acct_map.get("1130").expect("Savings").clone();
    let acc_cc = acct_map.get("2110").expect("Credit Card").clone();
    let acc_opening = acct_map.get("3110").expect("Opening Balance").clone();
    let acc_salary = acct_map.get("4110").expect("Salary").clone();
    let acc_other_inc = acct_map.get("4120").expect("Other Income").clone();
    let acc_util = acct_map.get("5110").expect("Utilities").clone();
    let acc_online = acct_map.get("5120").expect("Online Services").clone();
    let acc_clothes = acct_map.get("5210").expect("Clothes").clone();
    let acc_books = acct_map.get("5220").expect("Books").clone();
    let acc_hobbies = acct_map.get("5230").expect("Hobbies").clone();
    let acc_transit = acct_map.get("5310").expect("Transit").clone();

    // 2. Setup budget envelopes
    budget_service::upsert_budget(
        &conn,
        UpsertBudgetInput {
            id: None,
            month: "2024-06".into(),
            account_id: acc_util.clone(),
            amount: 1_500_000,
        },
    )
    .expect("upsert budget");

    // 3. Synthesize 2,500 transactions across 2024-2025
    println!(">>> Generating and posting 2,500 transactions...");
    let start_gen = Instant::now();

    // Tx 0: Opening Balance (50,000,000 IDR into Bank)
    ledger_service::post_journal_entry(
        &mut conn,
        CreateJournalEntryInput {
            id: Some("tx_init_opening".into()),
            date: "2024-01-01".into(),
            description: "Opening Balance Initial Deposit".into(),
            notes: Some("Initial funding".into()),
            reference_no: None,
            due_date: None,
            plan_id: None,
            currency: Some("IDR".into()),
            fx_rate: Some(16000),
            postings: vec![
                PostingInput {
                    id: None,
                    account_id: acc_bank.clone(),
                    amount: 50_000_000,
                    memo: Some("Initial bank capital".into()),
                    action: None,
                    reconcile: Some(ReconcileState::Reconciled),
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_opening.clone(),
                    amount: -50_000_000,
                    memo: Some("Equity offset".into()),
                    action: None,
                    reconcile: Some(ReconcileState::Reconciled),
                    ..Default::default()
                },
            ],
        },
        "stress-test",
    )
    .expect("post opening tx");

    let mut tx_count: usize = 1;

    // Loop 720 days (approx 2 years), generating 3-4 transactions per day
    for day_idx in 1..=720 {
        let year = 2024 + (day_idx / 365);
        let rem_day = (day_idx % 365) + 1;
        let month = (rem_day / 30).clamp(1, 12);
        let day = (rem_day % 28).clamp(1, 28);
        let date_str = format!("{year:04}-{month:02}-{day:02}");

        // Monthly Salary on the 25th
        if day == 25 {
            ledger_service::post_journal_entry(
                &mut conn,
                CreateJournalEntryInput {
                    id: None,
                    date: date_str.clone(),
                    description: format!("Monthly Salary {date_str}"),
                    notes: None,
                    reference_no: None,
                    due_date: None,
                    plan_id: None,
                    currency: Some("IDR".into()),
                    fx_rate: Some(16000),
                    postings: vec![
                        PostingInput {
                            id: None,
                            account_id: acc_bank.clone(),
                            amount: 25_000_000,
                            memo: Some("Take home pay".into()),
                            action: None,
                            reconcile: Some(ReconcileState::Cleared),
                            ..Default::default()
                        },
                        PostingInput {
                            id: None,
                            account_id: acc_salary.clone(),
                            amount: -25_000_000,
                            memo: Some("Gross salary".into()),
                            action: None,
                            reconcile: Some(ReconcileState::None),
                            ..Default::default()
                        },
                    ],
                },
                "stress-test",
            )
            .expect("post salary");
            tx_count += 1;
        }

        // Weekly Inter-Account Liquid Transfer (Bank -> Cash or Savings)
        if day_idx % 7 == 0 {
            let target_acc = if day_idx % 14 == 0 {
                acc_savings.clone()
            } else {
                acc_cash.clone()
            };
            let transfer_amt = 1_000_000 + (day_idx as i64 * 100);
            ledger_service::post_journal_entry(
                &mut conn,
                CreateJournalEntryInput {
                    id: None,
                    date: date_str.clone(),
                    description: format!("Liquid Transfer {date_str}"),
                    notes: None,
                    reference_no: None,
                    due_date: None,
                    plan_id: None,
                    currency: Some("IDR".into()),
                    fx_rate: Some(16000),
                    postings: vec![
                        PostingInput {
                            id: None,
                            account_id: target_acc,
                            amount: transfer_amt,
                            memo: Some("Transfer inflow".into()),
                            action: None,
                            reconcile: Some(ReconcileState::Cleared),
                            ..Default::default()
                        },
                        PostingInput {
                            id: None,
                            account_id: acc_bank.clone(),
                            amount: -transfer_amt,
                            memo: Some("Transfer outflow".into()),
                            action: None,
                            reconcile: Some(ReconcileState::Cleared),
                            ..Default::default()
                        },
                    ],
                },
                "stress-test",
            )
            .expect("post transfer");
            tx_count += 1;
        }

        // Daily Multi-Split Expenses (Food, Transit, Utilities, Online)
        let split_1 = 45_000 + (day_idx as i64 * 30);
        let split_2 = 25_000 + (day_idx as i64 * 20);
        let total_daily = split_1 + split_2;

        let funding_acc = if day_idx % 3 == 0 {
            acc_cc.clone() // Liability
        } else if day_idx % 2 == 0 {
            acc_cash.clone() // Cash on hand
        } else {
            acc_bank.clone() // Checking
        };

        ledger_service::post_journal_entry(
            &mut conn,
            CreateJournalEntryInput {
                id: None,
                date: date_str.clone(),
                description: format!("Daily Split Expense {day_idx}"),
                notes: None,
                reference_no: None,
                due_date: None,
                plan_id: None,
                currency: Some("IDR".into()),
                fx_rate: Some(16000),
                postings: vec![
                    PostingInput {
                        id: None,
                        account_id: acc_transit.clone(),
                        amount: split_1,
                        memo: Some("Fuel / commute".into()),
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                    PostingInput {
                        id: None,
                        account_id: acc_online.clone(),
                        amount: split_2,
                        memo: Some("App subscription/service".into()),
                        action: None,
                        reconcile: None,
                        ..Default::default()
                    },
                    PostingInput {
                        id: None,
                        account_id: funding_acc,
                        amount: -total_daily,
                        memo: Some("Payment".into()),
                        action: None,
                        reconcile: Some(ReconcileState::None),
                        ..Default::default()
                    },
                ],
            },
            "stress-test",
        )
        .expect("post daily expense");
        tx_count += 1;

        // Additional Variable Expenses (Hobbies, Books, Clothes)
        if day_idx % 2 == 0 {
            let (target_expense, desc) = match day_idx % 3 {
                0 => (acc_hobbies.clone(), "Weekend Recreation"),
                1 => (acc_books.clone(), "Technical Architecture Books"),
                _ => (acc_clothes.clone(), "Apparel Purchase"),
            };
            let amt = 120_000 + (day_idx as i64 * 50);

            ledger_service::post_journal_entry(
                &mut conn,
                CreateJournalEntryInput {
                    id: None,
                    date: date_str.clone(),
                    description: format!("{desc} #{day_idx}"),
                    notes: None,
                    reference_no: None,
                    due_date: None,
                    plan_id: None,
                    currency: Some("IDR".into()),
                    fx_rate: Some(16000),
                    postings: vec![
                        PostingInput {
                            id: None,
                            account_id: target_expense,
                            amount: amt,
                            memo: None,
                            action: None,
                            reconcile: None,
                            ..Default::default()
                        },
                        PostingInput {
                            id: None,
                            account_id: acc_bank.clone(),
                            amount: -amt,
                            memo: None,
                            action: None,
                            reconcile: Some(ReconcileState::Cleared),
                            ..Default::default()
                        },
                    ],
                },
                "stress-test",
            )
            .expect("post variable expense");
            tx_count += 1;
        }

        // Occasional Freelance / Other Income
        if day_idx % 15 == 0 {
            let fee = 3_500_000 + (day_idx as i64 * 200);
            ledger_service::post_journal_entry(
                &mut conn,
                CreateJournalEntryInput {
                    id: None,
                    date: date_str.clone(),
                    description: format!("Freelance Consulting Fee {day_idx}"),
                    notes: None,
                    reference_no: None,
                    due_date: None,
                    plan_id: None,
                    currency: Some("IDR".into()),
                    fx_rate: Some(16000),
                    postings: vec![
                        PostingInput {
                            id: None,
                            account_id: acc_bank.clone(),
                            amount: fee,
                            memo: Some("Client payment".into()),
                            action: None,
                            reconcile: Some(ReconcileState::Reconciled),
                            ..Default::default()
                        },
                        PostingInput {
                            id: None,
                            account_id: acc_other_inc.clone(),
                            amount: -fee,
                            memo: Some("Consulting services".into()),
                            action: None,
                            reconcile: Some(ReconcileState::None),
                            ..Default::default()
                        },
                    ],
                },
                "stress-test",
            )
            .expect("post freelance income");
            tx_count += 1;
        }

        // Monthly Credit Card Payoff
        if day == 27 {
            ledger_service::post_journal_entry(
                &mut conn,
                CreateJournalEntryInput {
                    id: None,
                    date: date_str.clone(),
                    description: format!("Credit Card Full Settlement {date_str}"),
                    notes: None,
                    reference_no: None,
                    due_date: None,
                    plan_id: None,
                    currency: Some("IDR".into()),
                    fx_rate: Some(16000),
                    postings: vec![
                        PostingInput {
                            id: None,
                            account_id: acc_cc.clone(),
                            amount: 1_200_000,
                            memo: Some("Pay down credit card liability".into()),
                            action: None,
                            reconcile: Some(ReconcileState::Reconciled),
                            ..Default::default()
                        },
                        PostingInput {
                            id: None,
                            account_id: acc_bank.clone(),
                            amount: -1_200_000,
                            memo: Some("Bank auto-debit".into()),
                            action: None,
                            reconcile: Some(ReconcileState::Reconciled),
                            ..Default::default()
                        },
                    ],
                },
                "stress-test",
            )
            .expect("post cc payoff");
            tx_count += 1;
        }

        if tx_count >= 2500 {
            break;
        }
    }

    // Edge Case: Extreme High-Value Transaction (5 Billion IDR Capital Injection)
    ledger_service::post_journal_entry(
        &mut conn,
        CreateJournalEntryInput {
            id: Some("tx_huge_capital".into()),
            date: "2025-06-15".into(),
            description: "Major Capital Asset Injection (Zero-Float Overflow Check)".into(),
            notes: Some("Stress testing large integer units".into()),
            reference_no: None,
            due_date: None,
            plan_id: None,
            currency: Some("IDR".into()),
            fx_rate: Some(16000),
            postings: vec![
                PostingInput {
                    id: None,
                    account_id: acc_bank.clone(),
                    amount: 5_000_000_000,
                    memo: Some("5 Billion IDR deposit".into()),
                    action: None,
                    reconcile: Some(ReconcileState::Reconciled),
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_opening.clone(),
                    amount: -5_000_000_000,
                    memo: Some("Additional paid-in capital".into()),
                    action: None,
                    reconcile: Some(ReconcileState::Reconciled),
                    ..Default::default()
                },
            ],
        },
        "stress-test",
    )
    .expect("post large capital tx");
    tx_count += 1;

    // Edge Case: Backdated Transaction posted at the end
    ledger_service::post_journal_entry(
        &mut conn,
        CreateJournalEntryInput {
            id: Some("tx_backdated_entry".into()),
            date: "2024-03-15".into(),
            description: "Belated Utility Bill (Backdated Edge Case)".into(),
            notes: Some("Inserted out of chronological order".into()),
            reference_no: None,
            due_date: None,
            plan_id: None,
            currency: Some("IDR".into()),
            fx_rate: Some(16000),
            postings: vec![
                PostingInput {
                    id: None,
                    account_id: acc_util.clone(),
                    amount: 350_000,
                    memo: Some("March water bill".into()),
                    action: None,
                    reconcile: Some(ReconcileState::Cleared),
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_bank.clone(),
                    amount: -350_000,
                    memo: Some("Bank transfer".into()),
                    action: None,
                    reconcile: Some(ReconcileState::Cleared),
                    ..Default::default()
                },
            ],
        },
        "stress-test",
    )
    .expect("post backdated tx");
    tx_count += 1;

    let duration_gen = start_gen.elapsed();
    println!(
        ">>> Inserted {} transactions in {:?} ({:.1} tx/sec)",
        tx_count,
        duration_gen,
        tx_count as f64 / duration_gen.as_secs_f64()
    );

    // =========================================================================
    // INVARIANT VERIFICATION 1: Trial Balance (Sum Debit == Sum Credit)
    // =========================================================================
    let t_tb_start = Instant::now();
    let tb = trial_balance::generate(&conn, None).expect("generate trial balance");
    let t_tb_dur = t_tb_start.elapsed();

    println!(
        ">>> [INVARIANT 1] Trial Balance: Debit = {}, Credit = {}, Balanced = {} (took {:?})",
        tb.total_debit, tb.total_credit, tb.is_balanced, t_tb_dur
    );
    assert!(tb.is_balanced, "Trial balance must strictly balance!");
    assert_eq!(
        tb.total_debit, tb.total_credit,
        "Total debits must equal total credits exactly"
    );
    assert!(tb.total_debit > 0, "Debits must be positive non-zero");

    // =========================================================================
    // INVARIANT VERIFICATION 2: Balance Sheet Accounting Identity
    // Total Assets = Total Liabilities + Total Equity + Net Income
    // =========================================================================
    let t_bs_start = Instant::now();
    let bs = balance_sheet::generate(&conn, None).expect("generate balance sheet");
    let t_bs_dur = t_bs_start.elapsed();

    println!(
        ">>> [INVARIANT 2] Balance Sheet: Assets = {}, Liab = {}, Eq = {}, NetInc = {}, Discrepancy = {} (took {:?})",
        bs.total_assets, bs.total_liabilities, bs.total_equity, bs.net_income, bs.discrepancy, t_bs_dur
    );
    assert_eq!(
        bs.discrepancy, 0,
        "Balance sheet discrepancy must be strictly 0 under all circumstances!"
    );
    assert!(
        bs.is_balanced,
        "Balance sheet must report is_balanced = true"
    );

    // =========================================================================
    // INVARIANT VERIFICATION 3: Profit & Loss (Income - Expense == Net Income)
    // =========================================================================
    let t_pl_start = Instant::now();
    let pl = profit_loss::generate(&conn, None, None).expect("generate profit loss");
    let t_pl_dur = t_pl_start.elapsed();

    println!(
        ">>> [INVARIANT 3] Profit & Loss: Total Income = {}, Total Expenses = {}, Net Income = {} (took {:?})",
        pl.total_income, pl.total_expenses, pl.net_income, t_pl_dur
    );
    assert_eq!(
        pl.net_income,
        pl.total_income - pl.total_expenses,
        "Net income must equal total_income - total_expenses"
    );
    assert_eq!(
        pl.net_income, bs.net_income,
        "Profit & Loss Net Income must strictly match Balance Sheet Net Income!"
    );

    // =========================================================================
    // INVARIANT VERIFICATION 4: Cash Flow Identity & Liquidity Reconciliation
    // Ending Cash = Starting Cash + Net Cash Change
    // Ending Cash == Sum of all liquid accounts in system
    // =========================================================================
    let t_cf_start = Instant::now();
    let cf = cash_flow::generate(&conn, None, None).expect("generate cash flow");
    let t_cf_dur = t_cf_start.elapsed();

    println!(
        ">>> [INVARIANT 4] Cash Flow: Starting = {}, Net Change = {}, Ending = {} (took {:?})",
        cf.starting_cash, cf.net_cash_change, cf.ending_cash, t_cf_dur
    );
    assert_eq!(
        cf.ending_cash,
        cf.starting_cash + cf.net_cash_change,
        "Cash flow ending cash must equal starting_cash + net_cash_change"
    );
    assert_eq!(
        cf.net_cash_change,
        cf.operating_cash_flow + cf.investing_cash_flow + cf.financing_cash_flow,
        "Net cash change must equal sum of operating + investing + financing"
    );

    // Compare ending_cash against direct query of liquid accounts
    let accounts_with_balances =
        accounts_service::list_accounts_with_balances(&conn).expect("list accounts with balances");
    let actual_liquid_sum: i64 = accounts_with_balances
        .iter()
        .filter(|a| {
            !a.account.placeholder
                && (a.account.code == "1110"
                    || a.account.code == "1120"
                    || a.account.code == "1130")
        })
        .map(|a| a.recursive_balance)
        .sum();

    assert_eq!(
        cf.ending_cash, actual_liquid_sum,
        "Cash flow ending_cash must strictly equal actual leaf balance sum of cash/bank accounts!"
    );

    // =========================================================================
    // INVARIANT VERIFICATION 5: Running Ledger Monotonicity
    // =========================================================================
    let t_rl_start = Instant::now();
    let bank_ledger = ledger_service::get_account_running_ledger(&conn, &acc_bank)
        .expect("get account running ledger");
    let t_rl_dur = t_rl_start.elapsed();

    println!(
        ">>> [INVARIANT 5] Bank Running Ledger: {} entries (took {:?})",
        bank_ledger.len(),
        t_rl_dur
    );
    assert!(!bank_ledger.is_empty());

    let mut running_calc: i64 = 0;
    for (idx, item) in bank_ledger.iter().enumerate() {
        running_calc += item.amount;
        assert_eq!(
            item.running_balance, running_calc,
            "Running balance corrupted at row {idx} for account {acc_bank}"
        );
    }

    let bank_account_view = accounts_with_balances
        .iter()
        .find(|a| a.account.id == acc_bank)
        .expect("bank account view");
    assert_eq!(
        bank_ledger.last().unwrap().running_balance,
        bank_account_view.recursive_balance,
        "Final running ledger balance must equal total recursive account balance"
    );

    // =========================================================================
    // INVARIANT VERIFICATION 6: Historical Trends Reporting & Monthly Cashflow
    // =========================================================================
    let t_trends_start = Instant::now();
    let trends = historical_trends::generate(&conn, "2024-01-01", "2025-12-31", 16000)
        .expect("generate trends");
    let t_trends_dur = t_trends_start.elapsed();

    println!(
        ">>> [INVARIANT 6] Trends: Generated {} points from {} to {} (took {:?})",
        trends.points.len(),
        trends.from_date,
        trends.to_date,
        t_trends_dur
    );
    assert!(
        !trends.points.is_empty(),
        "Trends should return daily points"
    );

    let t_mcf_start = Instant::now();
    let mcf = monthly_cashflow::generate(&conn, 24).expect("generate monthly cashflow");
    let t_mcf_dur = t_mcf_start.elapsed();

    println!(
        ">>> [INVARIANT 7] Monthly Cashflow: Generated {} months (took {:?})",
        mcf.len(),
        t_mcf_dur
    );
    assert!(
        !mcf.is_empty(),
        "Monthly cashflow points should not be empty"
    );

    // =========================================================================
    // PERFORMANCE BUDGET ASSERTIONS: All queries must complete under latency threshold
    // =========================================================================
    println!("\n========================================================");
    println!(
        ">>> SCALE & STRESS BENCHMARK SUMMARY ({} TRANSACTIONS)",
        tx_count
    );
    println!("--------------------------------------------------------");
    println!(
        "Trial Balance Query       : {:>8.2} ms",
        t_tb_dur.as_secs_f64() * 1000.0
    );
    println!(
        "Balance Sheet Query       : {:>8.2} ms",
        t_bs_dur.as_secs_f64() * 1000.0
    );
    println!(
        "Profit & Loss Query       : {:>8.2} ms",
        t_pl_dur.as_secs_f64() * 1000.0
    );
    println!(
        "Cash Flow Query           : {:>8.2} ms",
        t_cf_dur.as_secs_f64() * 1000.0
    );
    println!(
        "Account Running Ledger    : {:>8.2} ms",
        t_rl_dur.as_secs_f64() * 1000.0
    );
    println!(
        "Historical Trends Query   : {:>8.2} ms",
        t_trends_dur.as_secs_f64() * 1000.0
    );
    println!(
        "Monthly Cashflow Query    : {:>8.2} ms",
        t_mcf_dur.as_secs_f64() * 1000.0
    );
    println!("========================================================\n");

    assert!(
        t_bs_dur.as_millis() < 250,
        "Balance sheet generation exceeded 250ms SLA on 2,500 txs!"
    );
    assert!(
        t_cf_dur.as_millis() < 350,
        "Cash flow generation exceeded 350ms SLA on 2,500 txs!"
    );
    assert!(
        t_tb_dur.as_millis() < 250,
        "Trial balance generation exceeded 250ms SLA on 2,500 txs!"
    );
}

#[test]
fn test_extreme_financial_edge_cases_and_invariants() {
    use finnca_lib::accounts::dto::CreateAccountInput;
    use finnca_lib::accounts::models::AccountType;
    use finnca_lib::report::generators::fx_revaluation;

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let db_path = temp_dir.path().join("edge_cases_vault.db");
    let dek = [99u8; 32];
    let mut conn = open_vault_db(&db_path, &dek).expect("open test db");

    let accounts =
        accounts_service::seed_comprehensive_accounts(&conn, "en", "IDR").expect("seed accounts");
    let mut acct_map: HashMap<String, String> = HashMap::new();
    for a in &accounts {
        acct_map.insert(a.code.clone(), a.id.clone());
    }

    let acc_cash = acct_map.get("1110").unwrap().clone();
    let acc_bank = acct_map.get("1120").unwrap().clone();
    let acc_savings = acct_map.get("1130").unwrap().clone();
    let acc_opening = acct_map.get("3110").unwrap().clone();
    let acc_salary = acct_map.get("4110").unwrap().clone();
    let acc_util = acct_map.get("5110").unwrap().clone();
    let acc_online = acct_map.get("5120").unwrap().clone();
    let acc_clothes = acct_map.get("5210").unwrap().clone();
    let acc_transit = acct_map.get("5310").unwrap().clone();

    // 1. Rejection tests (strict double-entry invariants)
    let err_unbalanced = ledger_service::post_journal_entry(
        &mut conn,
        CreateJournalEntryInput {
            id: None,
            date: "2025-01-01".into(),
            description: "Unbalanced Entry".into(),
            notes: None,
            reference_no: None,
            due_date: None,
            plan_id: None,
            currency: Some("IDR".into()),
            fx_rate: Some(16000),
            postings: vec![
                PostingInput {
                    id: None,
                    account_id: acc_bank.clone(),
                    amount: 100_000,
                    memo: None,
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_salary.clone(),
                    amount: -90_000,
                    memo: None,
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
            ],
        },
        "test",
    );
    assert!(err_unbalanced.is_err(), "Unbalanced entry must be rejected");

    let err_zero = ledger_service::post_journal_entry(
        &mut conn,
        CreateJournalEntryInput {
            id: None,
            date: "2025-01-01".into(),
            description: "Zero Posting Entry".into(),
            notes: None,
            reference_no: None,
            due_date: None,
            plan_id: None,
            currency: Some("IDR".into()),
            fx_rate: Some(16000),
            postings: vec![
                PostingInput {
                    id: None,
                    account_id: acc_bank.clone(),
                    amount: 0,
                    memo: None,
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_salary.clone(),
                    amount: 0,
                    memo: None,
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
            ],
        },
        "test",
    );
    assert!(err_zero.is_err(), "Zero-amount postings must be rejected");

    // 2. Severe Overdraft / Negative Balance Test
    // Initial funding: Rp 2,000,000 into Cash
    ledger_service::post_journal_entry(
        &mut conn,
        CreateJournalEntryInput {
            id: Some("tx_init".into()),
            date: "2025-01-01".into(),
            description: "Initial Cash".into(),
            notes: None,
            reference_no: None,
            due_date: None,
            plan_id: None,
            currency: Some("IDR".into()),
            fx_rate: Some(16000),
            postings: vec![
                PostingInput {
                    id: None,
                    account_id: acc_cash.clone(),
                    amount: 2_000_000,
                    memo: None,
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_opening.clone(),
                    amount: -2_000_000,
                    memo: None,
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
            ],
        },
        "test",
    )
    .expect("post init");

    // Extreme Expense: Spend Rp 10,000,000 from Cash (Overdraft by -Rp 8,000,000)
    let tx_overdraft = ledger_service::post_journal_entry(
        &mut conn,
        CreateJournalEntryInput {
            id: Some("tx_overdraft".into()),
            date: "2025-01-05".into(),
            description: "Extreme Overdraft Expense".into(),
            notes: None,
            reference_no: None,
            due_date: None,
            plan_id: None,
            currency: Some("IDR".into()),
            fx_rate: Some(16000),
            postings: vec![
                PostingInput {
                    id: None,
                    account_id: acc_clothes.clone(),
                    amount: 10_000_000,
                    memo: None,
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_cash.clone(),
                    amount: -10_000_000,
                    memo: None,
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
            ],
        },
        "test",
    )
    .expect("post overdraft tx");

    // Verify Invariants during Overdraft
    let tb_od = trial_balance::generate(&conn, None).expect("tb overdraft");
    assert!(
        tb_od.is_balanced,
        "Trial balance must balance during overdraft"
    );
    assert_eq!(tb_od.total_debit, tb_od.total_credit);

    let bs_od = balance_sheet::generate(&conn, None).expect("bs overdraft");
    assert_eq!(
        bs_od.discrepancy, 0,
        "Balance sheet discrepancy must be 0 even under overdraft"
    );
    assert!(bs_od.is_balanced);

    let cf_od = cash_flow::generate(&conn, None, None).expect("cf overdraft");
    assert_eq!(
        cf_od.ending_cash, -8_000_000,
        "Cash flow ending balance must be -8,000,000"
    );
    assert_eq!(
        cf_od.starting_cash + cf_od.net_cash_change,
        cf_od.ending_cash
    );

    // 3. Multi-Split Complex Transaction (7-way Split Payroll)
    ledger_service::post_journal_entry(
        &mut conn,
        CreateJournalEntryInput {
            id: Some("tx_complex_payroll".into()),
            date: "2025-01-25".into(),
            description: "Complex Multi-Split Executive Payroll".into(),
            notes: Some("7-way split verification".into()),
            reference_no: None,
            due_date: None,
            plan_id: None,
            currency: Some("IDR".into()),
            fx_rate: Some(16000),
            postings: vec![
                // Inflows into accounts
                PostingInput {
                    id: None,
                    account_id: acc_bank.clone(),
                    amount: 20_000_000,
                    memo: Some("Checking deposit".into()),
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_savings.clone(),
                    amount: 8_000_000,
                    memo: Some("Savings allocation".into()),
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_cash.clone(),
                    amount: 2_000_000,
                    memo: Some("Pocket money".into()),
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
                // Direct expense deductions
                PostingInput {
                    id: None,
                    account_id: acc_util.clone(),
                    amount: 500_000,
                    memo: Some("Direct bill debit".into()),
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_transit.clone(),
                    amount: 500_000,
                    memo: Some("Transport allowance expense".into()),
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_online.clone(),
                    amount: 1_000_000,
                    memo: Some("Work software debit".into()),
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
                // Gross Salary credit
                PostingInput {
                    id: None,
                    account_id: acc_salary.clone(),
                    amount: -32_000_000,
                    memo: Some("Total gross remuneration".into()),
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
            ],
        },
        "test",
    )
    .expect("post complex payroll");

    let tb_payroll = trial_balance::generate(&conn, None).expect("tb payroll");
    assert!(tb_payroll.is_balanced);
    assert_eq!(tb_payroll.total_debit, tb_payroll.total_credit);

    // 4. Mid-Ledger Deletion & Balance Restoration
    // Delete the overdraft transaction and verify that running balances heal perfectly
    ledger_service::delete_journal_entry(&mut conn, &tx_overdraft.id, "test")
        .expect("delete overdraft tx");

    let bs_after_del = balance_sheet::generate(&conn, None).expect("bs after del");
    assert_eq!(bs_after_del.discrepancy, 0);

    let cf_after_del = cash_flow::generate(&conn, None, None).expect("cf after del");
    // Cash balance now: 2,000,000 (init) + 2,000,000 (from payroll) = 4,000,000 (cash) + 20,000,000 (bank) + 8,000,000 (savings) = 32,000,000
    assert_eq!(cf_after_del.ending_cash, 32_000_000);

    // 5. Multi-Currency USD Asset Ingestion & Spot FX Revaluation
    let acc_usd_cash = accounts_service::create_account(
        &conn,
        CreateAccountInput {
            code: "1140".into(),
            name: "USD Vault Cash".into(),
            account_type: AccountType::Asset,
            parent_id: None,
            currency: Some("USD".into()),
            placeholder: Some(false),
            hidden: None,
            color: None,
            note: None,
            description: Some("Cash held in USD notes".into()),
            interest_rate: None,
        },
        "test",
    )
    .expect("create USD asset account");

    let acc_usd_income = accounts_service::create_account(
        &conn,
        CreateAccountInput {
            code: "4010".into(),
            name: "USD Client Revenue".into(),
            account_type: AccountType::Income,
            parent_id: None,
            currency: Some("USD".into()),
            placeholder: Some(false),
            hidden: None,
            color: None,
            note: None,
            description: Some("International Client Revenue".into()),
            interest_rate: None,
        },
        "test",
    )
    .expect("create USD income account");

    // Receive $1,000 USD (100,000 minor units) at historical FX rate 16,000
    ledger_service::post_journal_entry(
        &mut conn,
        CreateJournalEntryInput {
            id: Some("tx_usd_revenue".into()),
            date: "2025-02-01".into(),
            description: "International Client Payment $1,000 USD".into(),
            notes: None,
            reference_no: None,
            due_date: None,
            plan_id: None,
            currency: Some("USD".into()),
            fx_rate: Some(16000),
            postings: vec![
                PostingInput {
                    id: None,
                    account_id: acc_usd_cash.id.clone(),
                    amount: 100_000, // +$1,000.00 USD
                    memo: Some("1000 USD deposited".into()),
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
                PostingInput {
                    id: None,
                    account_id: acc_usd_income.id.clone(),
                    amount: -100_000, // -$1,000.00 USD
                    memo: Some("Revenue earned".into()),
                    action: None,
                    reconcile: None,
                    ..Default::default()
                },
            ],
        },
        "test",
    )
    .expect("post usd revenue tx");

    // Revalue at Spot Rate 16,500 (Unrealized gain of +500 IDR per USD = +500,000 IDR)
    let fx_report = fx_revaluation::generate(&conn, None, 16500).expect("generate fx reval");
    assert!(!fx_report.items.is_empty());
    let item = fx_report
        .items
        .iter()
        .find(|i| i.account_id == acc_usd_cash.id)
        .expect("USD cash account in FX report");
    assert_eq!(item.native_balance, 100_000); // 1,000.00 USD
    assert_eq!(item.cost_basis_idr, 16_000_000);
    assert_eq!(item.current_value_idr, 16_500_000);
    assert_eq!(item.unrealized_gain_idr, 500_000);

    println!(">>> [EDGE CASES VERIFIED] Overdraft, 7-way split, transaction deletion, and FX spot revaluation all passed with 0 drift!");
}

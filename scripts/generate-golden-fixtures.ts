import fs from 'fs';
import path from 'path';
import type { VaultData } from '../src/lib/accounting/types';
import { convertMinor } from '../src/lib/accounting/core/math';
import { accountBalanceMinor, buildChildrenMap } from '../src/lib/accounting/ledger/accounts';
import {
  isBalanced,
  transactionImbalance,
  ledgerForAccount,
} from '../src/lib/accounting/ledger/transactions';

const FIXTURES_DIR = path.resolve(process.cwd(), 'src/tests/fixtures/ledger-golden');

function calculateScenarioSummary(vault: VaultData) {
  const cmap = buildChildrenMap(vault.accounts);
  const byId = new Map(vault.accounts.map((a) => [a.id, a]));

  // Account balances
  const accountBalances: Record<string, number> = {};
  for (const acc of vault.accounts) {
    accountBalances[acc.id] = accountBalanceMinor(acc.id, vault, cmap);
  }

  // Running ledgers for all leaf accounts
  const accountLedgers: Record<
    string,
    { txId: string; date: string; amount: number; running: number }[]
  > = {};
  for (const acc of vault.accounts) {
    if (!cmap.has(acc.id) && !acc.placeholder) {
      const ledger = ledgerForAccount(acc.id, vault, cmap);
      accountLedgers[acc.id] = ledger.map((l) => ({
        txId: l.tx.id,
        date: l.tx.date,
        amount: l.split.amount,
        running: l.running,
      }));
    }
  }

  // Invariants & Totals
  let totalDebits = 0;
  let totalCredits = 0;
  const transactionChecks: { id: string; balanced: boolean; imbalance: number }[] = [];

  for (const tx of vault.transactions) {
    const balanced = isBalanced(tx);
    const imbalance = transactionImbalance(tx);
    transactionChecks.push({ id: tx.id, balanced, imbalance });

    for (const sp of tx.splits) {
      if (sp.amount > 0) totalDebits += sp.amount;
      else totalCredits += Math.abs(sp.amount);
    }
  }

  // Income Statement (IDR equivalent)
  let totalIncome = 0;
  let totalExpense = 0;
  for (const tx of vault.transactions) {
    const fx = tx.fxRateAtTransaction || vault.fxRate || 16000;
    for (const sp of tx.splits) {
      const acc = byId.get(sp.accountId);
      if (!acc) continue;
      const amtInIdr =
        acc.currency === 'USD' ? convertMinor(sp.amount, 'USD', 'IDR', fx) : sp.amount;
      if (acc.type === 'INCOME') totalIncome += -amtInIdr;
      if (acc.type === 'EXPENSE') totalExpense += amtInIdr;
    }
  }

  // Balance Sheet Categories (IDR equivalent)
  let totalAssets = 0;
  let totalLiabilities = 0;
  let totalEquity = 0;

  for (const acc of vault.accounts) {
    if (cmap.has(acc.id) || acc.placeholder) continue;
    const raw = accountBalances[acc.id];
    const balIdr = acc.currency === 'USD' ? convertMinor(raw, 'USD', 'IDR', vault.fxRate) : raw;

    if (acc.type === 'ASSET') totalAssets += balIdr;
    else if (acc.type === 'LIABILITY') totalLiabilities += -balIdr;
    else if (acc.type === 'EQUITY') totalEquity += -balIdr;
  }

  const netIncome = totalIncome - totalExpense;
  const balanceSheetDiscrepancy = totalAssets - totalLiabilities - totalEquity - netIncome;

  return {
    accountBalances,
    accountLedgers,
    totals: {
      totalAssets,
      totalLiabilities,
      totalEquity,
      totalIncome,
      totalExpenses: totalExpense,
      netIncome,
      balanceSheetDiscrepancy,
      isBalanceSheetAligned: balanceSheetDiscrepancy === 0,
    },
    invariants: {
      totalDebits,
      totalCredits,
      isZeroSum: totalDebits === totalCredits,
      allTransactionsBalanced: transactionChecks.every((t) => t.balanced),
      transactionChecks,
    },
  };
}

// -------------------------------------------------------------
// Scenario 1: Simple Transfers & Opening Balances
// -------------------------------------------------------------
const scenario1: VaultData = {
  version: 2,
  fxRate: 16000,
  updatedAt: '2026-09-14T00:00:00Z',
  accounts: [
    {
      id: 'acc-eq-opening',
      code: '3000',
      name: 'Opening Balance Equity',
      type: 'EQUITY',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-asset-bank-bca',
      code: '1001',
      name: 'Bank BCA',
      type: 'ASSET',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-asset-cash',
      code: '1002',
      name: 'Cash Wallet',
      type: 'ASSET',
      currency: 'IDR',
      placeholder: false,
    },
  ],
  transactions: [
    {
      id: 'tx-01',
      date: '2026-09-01',
      description: 'Opening Capital injection',
      currency: 'IDR',
      splits: [
        { id: 'sp-01-1', accountId: 'acc-asset-bank-bca', amount: 50_000_000, reconcile: 'y' },
        { id: 'sp-01-2', accountId: 'acc-eq-opening', amount: -50_000_000, reconcile: 'y' },
      ],
    },
    {
      id: 'tx-02',
      date: '2026-09-02',
      description: 'ATM Cash Withdrawal',
      currency: 'IDR',
      splits: [
        { id: 'sp-02-1', accountId: 'acc-asset-cash', amount: 2_000_000, reconcile: 'n' },
        { id: 'sp-02-2', accountId: 'acc-asset-bank-bca', amount: -2_000_000, reconcile: 'y' },
      ],
    },
    {
      id: 'tx-03',
      date: '2026-09-05',
      description: 'Deposit unused cash back to BCA',
      currency: 'IDR',
      splits: [
        { id: 'sp-03-1', accountId: 'acc-asset-bank-bca', amount: 500_000, reconcile: 'y' },
        { id: 'sp-03-2', accountId: 'acc-asset-cash', amount: -500_000, reconcile: 'n' },
      ],
    },
  ],
};

// -------------------------------------------------------------
// Scenario 2: Multi-Split Expenses & Payroll Deduction
// -------------------------------------------------------------
const scenario2: VaultData = {
  version: 2,
  fxRate: 16000,
  updatedAt: '2026-09-14T00:00:00Z',
  accounts: [
    {
      id: 'acc-bca',
      code: '1001',
      name: 'Bank BCA',
      type: 'ASSET',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-salary',
      code: '4001',
      name: 'Gross Salary',
      type: 'INCOME',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-tax',
      code: '5001',
      name: 'Income Tax PPh21',
      type: 'EXPENSE',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-bpjs',
      code: '5002',
      name: 'Health Insurance',
      type: 'EXPENSE',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-groceries',
      code: '5003',
      name: 'Food & Groceries',
      type: 'EXPENSE',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-household',
      code: '5004',
      name: 'Household Supplies',
      type: 'EXPENSE',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-discount',
      code: '4002',
      name: 'Store Discounts/Rebates',
      type: 'INCOME',
      currency: 'IDR',
      placeholder: false,
    },
  ],
  transactions: [
    {
      id: 'tx-salary-sep',
      date: '2026-09-01',
      description: 'Monthly Salary with Deductions',
      currency: 'IDR',
      splits: [
        { id: 'sp-s1', accountId: 'acc-salary', amount: -25_000_000, reconcile: 'y' },
        { id: 'sp-s2', accountId: 'acc-tax', amount: 2_500_000, reconcile: 'y' },
        { id: 'sp-s3', accountId: 'acc-bpjs', amount: 500_000, reconcile: 'y' },
        { id: 'sp-s4', accountId: 'acc-bca', amount: 22_000_000, reconcile: 'y' },
      ],
    },
    {
      id: 'tx-supermarket',
      date: '2026-09-03',
      description: 'GrandLucky Supermarket run',
      currency: 'IDR',
      splits: [
        { id: 'sp-m1', accountId: 'acc-groceries', amount: 1_250_000, reconcile: 'n' },
        { id: 'sp-m2', accountId: 'acc-household', amount: 350_000, reconcile: 'n' },
        { id: 'sp-m3', accountId: 'acc-discount', amount: -100_000, reconcile: 'n' },
        { id: 'sp-m4', accountId: 'acc-bca', amount: -1_500_000, reconcile: 'y' },
      ],
    },
  ],
};

// -------------------------------------------------------------
// Scenario 3: Multi-Currency USD & IDR
// -------------------------------------------------------------
const scenario3: VaultData = {
  version: 2,
  fxRate: 16250, // 1 USD = 16,250 IDR
  updatedAt: '2026-09-14T00:00:00Z',
  accounts: [
    {
      id: 'acc-bca-idr',
      code: '1001',
      name: 'Bank BCA IDR',
      type: 'ASSET',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-wise-usd',
      code: '1002',
      name: 'Wise USD Account',
      type: 'ASSET',
      currency: 'USD',
      placeholder: false,
    },
    {
      id: 'acc-client-usd',
      code: '4010',
      name: 'Global Client Revenue',
      type: 'INCOME',
      currency: 'USD',
      placeholder: false,
    },
    {
      id: 'acc-saas-usd',
      code: '5010',
      name: 'Cloud Infrastructure SaaS',
      type: 'EXPENSE',
      currency: 'USD',
      placeholder: false,
    },
  ],
  transactions: [
    {
      id: 'tx-usd-inv',
      date: '2026-09-02',
      description: 'Client Invoice #8841 Payment',
      currency: 'USD',
      fxRateAtTransaction: 16200,
      splits: [
        { id: 'sp-u1', accountId: 'acc-client-usd', amount: -150000, reconcile: 'y' }, // $1,500.00
        { id: 'sp-u2', accountId: 'acc-wise-usd', amount: 150000, reconcile: 'y' },
      ],
    },
    {
      id: 'tx-saas-aws',
      date: '2026-09-05',
      description: 'AWS Cloud Hosting',
      currency: 'USD',
      fxRateAtTransaction: 16250,
      splits: [
        { id: 'sp-u3', accountId: 'acc-saas-usd', amount: 12550, reconcile: 'n' }, // $125.50
        { id: 'sp-u4', accountId: 'acc-wise-usd', amount: -12550, reconcile: 'n' },
      ],
    },
  ],
};

// -------------------------------------------------------------
// Scenario 4: Reconciled, Uncleared & Pending Splits
// -------------------------------------------------------------
const scenario4: VaultData = {
  version: 2,
  fxRate: 16000,
  updatedAt: '2026-09-14T00:00:00Z',
  accounts: [
    {
      id: 'acc-checking',
      code: '1010',
      name: 'Checking Account',
      type: 'ASSET',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-dining',
      code: '5020',
      name: 'Dining Out',
      type: 'EXPENSE',
      currency: 'IDR',
      placeholder: false,
    },
  ],
  transactions: [
    {
      id: 'tx-reconciled',
      date: '2026-09-01',
      description: 'Lunch cleared on statement',
      currency: 'IDR',
      splits: [
        { id: 'sp-rec-1', accountId: 'acc-dining', amount: 120_000, reconcile: 'y' },
        { id: 'sp-rec-2', accountId: 'acc-checking', amount: -120_000, reconcile: 'y' },
      ],
    },
    {
      id: 'tx-cleared-pending',
      date: '2026-09-08',
      description: 'Dinner cleared online pending statement',
      currency: 'IDR',
      splits: [
        { id: 'sp-cl-1', accountId: 'acc-dining', amount: 250_000, reconcile: 'c' },
        { id: 'sp-cl-2', accountId: 'acc-checking', amount: -250_000, reconcile: 'c' },
      ],
    },
    {
      id: 'tx-uncleared',
      date: '2026-09-12',
      description: 'Unpresented check for catering',
      currency: 'IDR',
      splits: [
        { id: 'sp-un-1', accountId: 'acc-dining', amount: 1_000_000, reconcile: 'n' },
        { id: 'sp-un-2', accountId: 'acc-checking', amount: -1_000_000, reconcile: 'n' },
      ],
    },
  ],
};

// -------------------------------------------------------------
// Scenario 5: Full Month Statement Simulation
// -------------------------------------------------------------
const scenario5: VaultData = {
  version: 2,
  fxRate: 16000,
  updatedAt: '2026-09-14T00:00:00Z',
  accounts: [
    // Assets
    {
      id: 'acc-asset-parent',
      code: '1000',
      name: 'Assets',
      type: 'ASSET',
      currency: 'IDR',
      placeholder: true,
    },
    {
      id: 'acc-bca',
      code: '1010',
      name: 'BCA Main',
      type: 'ASSET',
      parentId: 'acc-asset-parent',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-cash',
      code: '1020',
      name: 'Cash',
      type: 'ASSET',
      parentId: 'acc-asset-parent',
      currency: 'IDR',
      placeholder: false,
    },
    // Liabilities
    {
      id: 'acc-cc',
      code: '2010',
      name: 'Credit Card',
      type: 'LIABILITY',
      currency: 'IDR',
      placeholder: false,
    },
    // Equity
    {
      id: 'acc-equity',
      code: '3000',
      name: 'Initial Equity',
      type: 'EQUITY',
      currency: 'IDR',
      placeholder: false,
    },
    // Income
    {
      id: 'acc-rev',
      code: '4000',
      name: 'Salary',
      type: 'INCOME',
      currency: 'IDR',
      placeholder: false,
    },
    // Expenses
    {
      id: 'acc-exp-rent',
      code: '5010',
      name: 'Apartment Rent',
      type: 'EXPENSE',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-exp-utilities',
      code: '5020',
      name: 'Electricity & Water',
      type: 'EXPENSE',
      currency: 'IDR',
      placeholder: false,
    },
    {
      id: 'acc-exp-food',
      code: '5030',
      name: 'Food',
      type: 'EXPENSE',
      currency: 'IDR',
      placeholder: false,
    },
  ],
  transactions: [
    {
      id: 'tx-m-01',
      date: '2026-09-01',
      description: 'Opening balances',
      currency: 'IDR',
      splits: [
        { id: 'sp-m01-1', accountId: 'acc-bca', amount: 30_000_000, reconcile: 'y' },
        { id: 'sp-m01-2', accountId: 'acc-equity', amount: -30_000_000, reconcile: 'y' },
      ],
    },
    {
      id: 'tx-m-02',
      date: '2026-09-02',
      description: 'Pay monthly apartment rent',
      currency: 'IDR',
      splits: [
        { id: 'sp-m02-1', accountId: 'acc-exp-rent', amount: 6_000_000, reconcile: 'y' },
        { id: 'sp-m02-2', accountId: 'acc-bca', amount: -6_000_000, reconcile: 'y' },
      ],
    },
    {
      id: 'tx-m-03',
      date: '2026-09-05',
      description: 'Electricity bill payment',
      currency: 'IDR',
      splits: [
        { id: 'sp-m03-1', accountId: 'acc-exp-utilities', amount: 850_000, reconcile: 'y' },
        { id: 'sp-m03-2', accountId: 'acc-bca', amount: -850_000, reconcile: 'y' },
      ],
    },
    {
      id: 'tx-m-04',
      date: '2026-09-10',
      description: 'Groceries paid via Credit Card',
      currency: 'IDR',
      splits: [
        { id: 'sp-m04-1', accountId: 'acc-exp-food', amount: 1_200_000, reconcile: 'n' },
        { id: 'sp-m04-2', accountId: 'acc-cc', amount: -1_200_000, reconcile: 'n' },
      ],
    },
    {
      id: 'tx-m-05',
      date: '2026-09-25',
      description: 'Credit Card Bill settlement from BCA',
      currency: 'IDR',
      splits: [
        { id: 'sp-m05-1', accountId: 'acc-cc', amount: 1_200_000, reconcile: 'y' },
        { id: 'sp-m05-2', accountId: 'acc-bca', amount: -1_200_000, reconcile: 'y' },
      ],
    },
    {
      id: 'tx-m-06',
      date: '2026-09-28',
      description: 'Monthly Salary received',
      currency: 'IDR',
      splits: [
        { id: 'sp-m06-1', accountId: 'acc-bca', amount: 20_000_000, reconcile: 'y' },
        { id: 'sp-m06-2', accountId: 'acc-rev', amount: -20_000_000, reconcile: 'y' },
      ],
    },
  ],
};

function main() {
  if (!fs.existsSync(FIXTURES_DIR)) {
    fs.mkdirSync(FIXTURES_DIR, { recursive: true });
  }

  const scenarios = [
    { filename: '01_simple_transfers.json', name: 'Simple Asset Transfers', vault: scenario1 },
    {
      filename: '02_multi_split_expenses.json',
      name: 'Multi-Split Expenses & Payroll',
      vault: scenario2,
    },
    {
      filename: '03_multicurrency_usd_idr.json',
      name: 'Multi-Currency USD & IDR Operations',
      vault: scenario3,
    },
    {
      filename: '04_reconciled_uncleared_balances.json',
      name: 'Reconciled & Uncleared Balances',
      vault: scenario4,
    },
    {
      filename: '05_full_month_statements.json',
      name: 'Full Month Statement Simulation',
      vault: scenario5,
    },
  ];

  console.log(`Generating Golden Master Fixtures in: ${FIXTURES_DIR}\n`);

  for (const s of scenarios) {
    const summary = calculateScenarioSummary(s.vault);
    const goldenPayload = {
      scenarioName: s.name,
      generatedAt: new Date().toISOString(),
      engine: 'TypeScript Reference Oracle v1',
      inputVault: s.vault,
      expectedOutput: summary,
    };

    const outPath = path.join(FIXTURES_DIR, s.filename);
    fs.writeFileSync(outPath, JSON.stringify(goldenPayload, null, 2), 'utf-8');
    console.log(`✅ [CREATED] ${s.filename}`);
    console.log(`   - Accounts: ${s.vault.accounts.length}`);
    console.log(`   - Transactions: ${s.vault.transactions.length}`);
    console.log(
      `   - Invariants check: isZeroSum=${summary.invariants.isZeroSum}, allBalanced=${summary.invariants.allTransactionsBalanced}`
    );
    console.log(`   - Net Income: ${summary.totals.netIncome}\n`);
  }

  // Also create a symlink / copy to src-tauri/tests/fixtures/
  const tauriFixturesDir = path.resolve(process.cwd(), 'src-tauri/tests/fixtures');
  if (!fs.existsSync(tauriFixturesDir)) {
    fs.mkdirSync(tauriFixturesDir, { recursive: true });
  }
  for (const s of scenarios) {
    const src = path.join(FIXTURES_DIR, s.filename);
    const dest = path.join(tauriFixturesDir, s.filename);
    fs.copyFileSync(src, dest);
  }
  console.log(`🚀 Synchronized golden fixtures to Rust test suite: ${tauriFixturesDir}`);
}

main();

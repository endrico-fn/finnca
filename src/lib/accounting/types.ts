export type AccountType = 'ASSET' | 'LIABILITY' | 'EQUITY' | 'INCOME' | 'EXPENSE';

export const ACCOUNT_TYPE_LABEL: Record<AccountType, string> = {
  ASSET: 'Asset',
  LIABILITY: 'Liability',
  EQUITY: 'Equity',
  INCOME: 'Income',
  EXPENSE: 'Expense',
};

export const ACCOUNT_TYPE_COLOR: Record<AccountType, string> = {
  ASSET: 'var(--color-teal)',
  LIABILITY: 'var(--color-liability)',
  EQUITY: 'var(--color-equity)',
  INCOME: 'var(--color-income)',
  EXPENSE: 'var(--color-expense)',
};

export type Currency = 'IDR' | 'USD';
export const DEFAULT_FX_RATE = 17647;
export interface Account {
  id: string;
  code: string;
  name: string;
  type: AccountType;
  parentId: string | null;
  currency: Currency;
  placeholder: boolean;
  hidden: boolean;
  color?: string;
  note?: string;
  description?: string;
  interestRate?: number; // Annual percentage (e.g. 15 for 15%)
  createdAt: string;
}

export type ReconcileState = 'n' | 'c' | 'y';

export interface Split {
  id: string;
  accountId: string;
  amount: number;
  memo?: string;
  action?: string;
  reconcile: ReconcileState;
}

export interface Transaction {
  id: string;
  date: string; // yyyy-mm-dd
  dueDate?: string; // yyyy-mm-dd for payables/receivables
  settled?: boolean; // true if settled/paid
  description: string;
  num?: string;
  notes?: string;
  planId?: string;
  currency: Currency;
  fxRateAtTransaction?: number; // Exchange rate at the time of tx for FX Cost Basis
  splits: Split[];
}

export interface Profile {
  displayName?: string;
}

export interface FxRateHistoryEntry {
  date: string; // yyyy-mm-dd
  rate: number; // IDR per 1 USD
}

export type PlanType = 'RECEIVABLE' | 'PAYABLE' | 'RECURRING';
export type PlanFrequency = 'DAILY' | 'WEEKLY' | 'MONTHLY';
export type PlanStatus = 'ACTIVE' | 'COMPLETED' | 'OVERDUE' | 'ARCHIVED';

export interface PaymentPlan {
  id: string;
  title: string;
  type: PlanType;
  status?: PlanStatus;
  totalAmount: number; // in IDR minor
  installmentAmount: number; // in IDR minor
  frequency: PlanFrequency;
  startDate: string; // yyyy-mm-dd
  dueDate?: string; // yyyy-mm-dd
  dayOfMonth?: number; // 1-31
  fromAccountId: string;
  toAccountId: string;
  notes?: string;
  createdAt: string;
}

export interface BudgetAllocation {
  id: string;
  month: string; // YYYY-MM
  accountId: string;
  amount: number; // in minor units
}

export interface DashboardPrefs {
  liquidAccountIds?: string[];
}

export interface VaultData {
  version: 2;
  accounts: Account[];
  transactions: Transaction[];
  fxRate: number;
  fxHistory?: FxRateHistoryEntry[];
  profile?: Profile;
  plans?: PaymentPlan[];
  budgets?: BudgetAllocation[]; // Zero-Based Budgeting envelopes
  dashboardPrefs?: DashboardPrefs;
  updatedAt: string;
}

export function isDebitNormal(t: AccountType): boolean {
  return t === 'ASSET' || t === 'EXPENSE';
}

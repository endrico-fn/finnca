import { invokeIpc } from './client';
import type { AppStateView } from '$lib/core/types';

// Manual IPC contract: canonical reconcile codes are short n/c/y (see ledger/models.rs).

export type AccountType = 'ASSET' | 'LIABILITY' | 'EQUITY' | 'INCOME' | 'EXPENSE';
export type ReconcileStatus = 'n' | 'c' | 'y';

export interface Account {
  id: string;
  code: string;
  name: string;
  account_type: AccountType;
  parent_id: string | null;
  currency: string;
  placeholder: boolean;
  hidden: boolean;
  color: string | null;
  note: string | null;
  description: string | null;
  interest_rate: number | null;
  created_at: number;
}

export interface AccountBalanceView {
  account: Account;
  direct_balance: number;
  recursive_balance: number;
}

export interface CreateAccountInput {
  code: string;
  name: string;
  account_type: AccountType;
  parent_id?: string | null;
  currency?: string | null;
  placeholder?: boolean | null;
  hidden?: boolean | null;
  color?: string | null;
  note?: string | null;
  description?: string | null;
  interest_rate?: number | null;
}

export interface UpdateAccountInput {
  code?: string | null;
  name?: string | null;
  account_type?: AccountType | null;
  parent_id?: string | null;
  currency?: string | null;
  placeholder?: boolean | null;
  hidden?: boolean | null;
  color?: string | null;
  note?: string | null;
  description?: string | null;
  interest_rate?: number | null;
}

export interface PostingInput {
  id?: string | null;
  account_id: string;
  amount: number;
  memo?: string | null;
  action?: string | null;
  reconcile?: 'c' | 'y' | null;
}

export interface CreateJournalEntryInput {
  id?: string | null;
  date: string;
  description: string;
  notes?: string | null;
  currency?: string | null;
  fx_rate?: number | null;
  postings: PostingInput[];
}

export interface UpdateJournalEntryInput {
  date?: string | null;
  description?: string | null;
  notes?: string | null;
  currency?: string | null;
  fx_rate?: number | null;
  postings?: PostingInput[] | null;
}

export interface PostingView {
  id: string;
  entry_id: string;
  account_id: string;
  account_code: string;
  account_name: string;
  account_type: AccountType;
  amount: number;
  memo: string | null;
  action: string | null;
  reconcile: ReconcileStatus;
}

export interface JournalEntryView {
  id: string;
  date: string;
  description: string;
  notes: string | null;
  currency: string;
  fx_rate: number;
  posted_at: number;
  postings: PostingView[];
}

export interface LedgerTotalsView {
  total_assets: number;
  total_liabilities: number;
  total_equity: number;
  total_income: number;
  total_expenses: number;
  net_income: number;
  balance_sheet_discrepancy: number;
  is_balance_sheet_aligned: boolean;
}

export interface AccountRunningLedgerItem {
  entry_id: string;
  date: string;
  description: string;
  amount: number;
  running_balance: number;
  reconcile: ReconcileStatus;
}

export interface VaultRegistryEntry {
  id: string;
  name: string;
  path: string;
  username: string;
  last_opened_at?: string | null;
}

// =================== Commands ===================

export function getAppState(): Promise<AppStateView> {
  return invokeIpc<AppStateView>('get_app_state', {}, { label: 'get_app_state' });
}

export function unlockVault(password: string): Promise<AppStateView> {
  return invokeIpc<AppStateView>('unlock', { password }, { timeoutMs: 120_000, label: 'unlock' });
}

export function lockVault(): Promise<AppStateView> {
  return invokeIpc<AppStateView>('lock', {}, { label: 'lock' });
}

export function createVault(name: string, path: string): Promise<AppStateView> {
  return invokeIpc<AppStateView>(
    'create_vault',
    { name, path },
    { timeoutMs: 30_000, label: 'create_vault' }
  );
}

export function createAccount(
  username: string,
  password: string,
  templateLanguage?: string
): Promise<AppStateView> {
  return invokeIpc<AppStateView>(
    'create_account',
    { username, password, templateLanguage: templateLanguage ?? 'en' },
    { timeoutMs: 120_000, label: 'create_account' }
  );
}

export interface VaultInspectionResult {
  status: 'valid_sqlite' | 'valid_legacy' | 'not_found' | 'empty' | 'unrecognized';
  vault_name: string | null;
  username: string | null;
  is_valid: boolean;
  message: string;
}

export function inspectVaultFolder(path: string): Promise<VaultInspectionResult> {
  return invokeIpc<VaultInspectionResult>(
    'inspect_vault_folder',
    { path },
    { timeoutMs: 10_000, label: 'inspect_vault_folder' }
  );
}

export function importVault(path: string, password: string): Promise<AppStateView> {
  return invokeIpc<AppStateView>(
    'import_vault',
    { path, password },
    { timeoutMs: 120_000, label: 'import_vault' }
  );
}

export function changePassword(oldPassword: string, newPassword: string): Promise<AppStateView> {
  return invokeIpc<AppStateView>(
    'change_password',
    { oldPassword, newPassword },
    { timeoutMs: 120_000, label: 'change_password' }
  );
}

export function deleteVaultAndAccount(password: string): Promise<AppStateView> {
  return invokeIpc<AppStateView>(
    'delete_vault_and_account',
    { password },
    { timeoutMs: 30_000, label: 'delete_vault_and_account' }
  );
}

// Accounts commands
export function createAccountCmd(input: CreateAccountInput): Promise<Account> {
  return invokeIpc<Account>('create_account_cmd', { input }, { label: 'create_account_cmd' });
}

export function updateAccountCmd(id: string, input: UpdateAccountInput): Promise<Account> {
  return invokeIpc<Account>('update_account_cmd', { id, input }, { label: 'update_account_cmd' });
}

export function deleteAccountCmd(id: string): Promise<void> {
  return invokeIpc<void>('delete_account_cmd', { id }, { label: 'delete_account_cmd' });
}

export function getAccountCmd(id: string): Promise<Account> {
  return invokeIpc<Account>('get_account_cmd', { id }, { label: 'get_account_cmd' });
}

export function listAccountsCmd(): Promise<AccountBalanceView[]> {
  return invokeIpc<AccountBalanceView[]>('list_accounts_cmd', {}, { label: 'list_accounts_cmd' });
}

export function seedRootAccountsCmd(): Promise<Account[]> {
  return invokeIpc<Account[]>('seed_root_accounts_cmd', {}, { label: 'seed_root_accounts_cmd' });
}

export function seedStarterAccountsCmd(language?: string): Promise<Account[]> {
  return invokeIpc<Account[]>(
    'seed_starter_accounts_cmd',
    { language: language ?? 'en' },
    { label: 'seed_starter_accounts_cmd' }
  );
}

// Ledger commands
export function postJournalEntryCmd(input: CreateJournalEntryInput): Promise<JournalEntryView> {
  return invokeIpc<JournalEntryView>(
    'post_journal_entry_cmd',
    { input },
    { label: 'post_journal_entry_cmd' }
  );
}

export function updateJournalEntryCmd(
  id: string,
  input: UpdateJournalEntryInput
): Promise<JournalEntryView> {
  return invokeIpc<JournalEntryView>(
    'update_journal_entry_cmd',
    { id, input },
    { label: 'update_journal_entry_cmd' }
  );
}

export function deleteJournalEntryCmd(id: string): Promise<void> {
  return invokeIpc<void>('delete_journal_entry_cmd', { id }, { label: 'delete_journal_entry_cmd' });
}

export function getJournalEntryCmd(id: string): Promise<JournalEntryView> {
  return invokeIpc<JournalEntryView>(
    'get_journal_entry_cmd',
    { id },
    { label: 'get_journal_entry_cmd' }
  );
}

export function listJournalEntriesCmd(
  limit?: number,
  offset?: number
): Promise<JournalEntryView[]> {
  return invokeIpc<JournalEntryView[]>(
    'list_journal_entries_cmd',
    { limit, offset },
    { label: 'list_journal_entries_cmd' }
  );
}

export function getAccountLedgerCmd(accountId: string): Promise<AccountRunningLedgerItem[]> {
  return invokeIpc<AccountRunningLedgerItem[]>(
    'get_account_ledger_cmd',
    { accountId },
    { label: 'get_account_ledger_cmd' }
  );
}

export function getLedgerTotalsCmd(fxRate?: number): Promise<LedgerTotalsView> {
  return invokeIpc<LedgerTotalsView>(
    'get_ledger_totals_cmd',
    { fxRate },
    { label: 'get_ledger_totals_cmd' }
  );
}

// Registry and settings
export function getKnownVaultsCmd(): Promise<VaultRegistryEntry[]> {
  return invokeIpc<VaultRegistryEntry[]>(
    'get_known_vaults_cmd',
    {},
    { label: 'get_known_vaults_cmd' }
  );
}

export function rememberKnownVaultCmd(entry: VaultRegistryEntry): Promise<void> {
  return invokeIpc<void>(
    'remember_known_vault_cmd',
    { entry },
    { label: 'remember_known_vault_cmd' }
  );
}

export function forgetKnownVaultCmd(idOrPath: string): Promise<void> {
  return invokeIpc<void>(
    'forget_known_vault_cmd',
    { idOrPath },
    { label: 'forget_known_vault_cmd' }
  );
}

export function setActiveVault(path: string): Promise<AppStateView> {
  return invokeIpc<AppStateView>('set_active_vault', { path }, { label: 'set_active_vault' });
}

export function setAutoLockMode(mode: 'always' | 'on-reboot'): Promise<AppStateView> {
  return invokeIpc<AppStateView>('set_auto_lock_mode', { mode }, { label: 'set_auto_lock_mode' });
}

export function renameUser(username: string): Promise<AppStateView> {
  return invokeIpc<AppStateView>('rename_user', { username }, { label: 'rename_user' });
}

export function renameVault(name: string): Promise<AppStateView> {
  return invokeIpc<AppStateView>('rename_vault', { name }, { label: 'rename_vault' });
}

export function openVaultFolder(): Promise<void> {
  return invokeIpc<void>('open_vault_folder', {}, { label: 'open_vault_folder' });
}

export function exportVaultBackupFolder(destDir: string): Promise<void> {
  return invokeIpc<void>(
    'export_vault_backup_folder',
    { destDir },
    { label: 'export_vault_backup_folder' }
  );
}

export function readStatementFileCmd(path: string): Promise<string> {
  return invokeIpc<string>(
    'read_statement_file_cmd',
    { path },
    { label: 'read_statement_file_cmd' }
  );
}

export function writeFileRawCmd(path: string, contents: number[]): Promise<void> {
  return invokeIpc<void>(
    'write_file_raw',
    { path, contents },
    { label: 'write_file_raw' }
  );
}

export function exportTextFileCmd(path: string, contents: string): Promise<void> {
  return invokeIpc<void>(
    'export_text_file',
    { path, contents },
    { label: 'export_text_file' }
  );
}

// Plan commands & DTOs
export type PlanType = 'RECEIVABLE' | 'PAYABLE' | 'RECURRING';
export type PlanFrequency = 'DAILY' | 'WEEKLY' | 'MONTHLY';
export type PlanStatus = 'ACTIVE' | 'COMPLETED' | 'OVERDUE' | 'ARCHIVED';

export interface PlanDto {
  id: string;
  title: string;
  plan_type: PlanType;
  status: PlanStatus;
  total_amount: number;
  installment_amount: number;
  frequency: PlanFrequency;
  start_date: string;
  due_date: string | null;
  day_of_month: number | null;
  from_account_id: string;
  to_account_id: string;
  notes: string | null;
  created_at: number;
}

export interface PlanProgressViewDto {
  plan: PlanDto;
  paid_amount: number;
  remaining_amount: number;
  progress_percent: number;
  is_settled: boolean;
  installments_paid_count: number;
}

export function listPlansWithProgressCmd(): Promise<PlanProgressViewDto[]> {
  return invokeIpc<PlanProgressViewDto[]>(
    'list_plans_with_progress_cmd',
    {},
    { label: 'list_plans_with_progress_cmd' }
  );
}

// Audit commands
export interface AuditEntry {
  id: string;
  actor: string;
  action: string;
  entity_type: string;
  entity_id: string;
  detail?: string | null;
  created_at: number;
}

export interface AuditPage {
  entries: AuditEntry[];
  total: number;
  page: number;
  per_page: number;
}

export function getAuditLogCmd(page = 1, perPage = 50): Promise<AuditPage> {
  return invokeIpc<AuditPage>(
    'get_audit_log_cmd',
    { page, perPage },
    { label: 'get_audit_log_cmd' }
  );
}

export function getEntityAuditLogCmd(entityType: string, entityId: string): Promise<AuditEntry[]> {
  return invokeIpc<AuditEntry[]>(
    'get_entity_audit_log_cmd',
    { entityType, entityId },
    { label: 'get_entity_audit_log_cmd' }
  );
}

// Report commands
export interface FxRevaluationItem {
  account_id: string;
  code: string;
  name: string;
  currency: string;
  native_balance: number;
  cost_basis_idr: number;
  current_value_idr: number;
  unrealized_gain_idr: number;
}

export interface FxRevaluationReport {
  as_of_date: string | null;
  current_fx_rate: number;
  items: FxRevaluationItem[];
  total_cost_basis_idr: number;
  total_current_value_idr: number;
  total_unrealized_gain_idr: number;
}

export interface DailyTrendPoint {
  date: string;
  net_worth: number;
  assets: number;
  liabilities: number;
  liquid_cash: number;
}

export interface HistoricalTrendsReport {
  from_date: string;
  to_date: string;
  points: DailyTrendPoint[];
}

export interface AccountReportRow {
  account_id: string;
  code: string;
  name: string;
  currency: string;
  amount: number;
}

export interface ProfitLossReport {
  from_date: string | null;
  to_date: string | null;
  income_rows: AccountReportRow[];
  expense_rows: AccountReportRow[];
  total_income: number;
  total_expenses: number;
  net_income: number;
}

export interface BalanceSheetReport {
  as_of_date: string | null;
  asset_rows: AccountReportRow[];
  liability_rows: AccountReportRow[];
  equity_rows: AccountReportRow[];
  total_assets: number;
  total_liabilities: number;
  total_equity: number;
  net_income: number;
  discrepancy: number;
  is_balanced: boolean;
}

export interface CashFlowActivityRow {
  category: string;
  description: string;
  amount: number;
}

export interface CashFlowReport {
  from_date: string | null;
  to_date: string | null;
  starting_cash: number;
  operating_cash_flow: number;
  investing_cash_flow: number;
  financing_cash_flow: number;
  net_cash_change: number;
  ending_cash: number;
  operating_rows: CashFlowActivityRow[];
}

export interface TrialBalanceRow {
  account_id: string;
  code: string;
  name: string;
  account_type: string;
  currency: string;
  debit: number;
  credit: number;
}

export interface TrialBalanceReport {
  as_of_date: string | null;
  rows: TrialBalanceRow[];
  total_debit: number;
  total_credit: number;
  is_balanced: boolean;
}

export function getProfitLossReportCmd(
  fromDate?: string | null,
  toDate?: string | null
): Promise<ProfitLossReport> {
  return invokeIpc<ProfitLossReport>(
    'get_profit_loss_report_cmd',
    { fromDate: fromDate || null, toDate: toDate || null },
    { label: 'get_profit_loss_report_cmd' }
  );
}

export function getBalanceSheetReportCmd(
  asOfDate?: string | null
): Promise<BalanceSheetReport> {
  return invokeIpc<BalanceSheetReport>(
    'get_balance_sheet_report_cmd',
    { asOfDate: asOfDate || null },
    { label: 'get_balance_sheet_report_cmd' }
  );
}

export function getCashFlowReportCmd(
  fromDate?: string | null,
  toDate?: string | null
): Promise<CashFlowReport> {
  return invokeIpc<CashFlowReport>(
    'get_cash_flow_report_cmd',
    { fromDate: fromDate || null, toDate: toDate || null },
    { label: 'get_cash_flow_report_cmd' }
  );
}

export function getTrialBalanceReportCmd(
  asOfDate?: string | null
): Promise<TrialBalanceReport> {
  return invokeIpc<TrialBalanceReport>(
    'get_trial_balance_report_cmd',
    { asOfDate: asOfDate || null },
    { label: 'get_trial_balance_report_cmd' }
  );
}

export function getFxRevaluationReportCmd(
  asOfDate?: string | null,
  fxRate?: number | null
): Promise<FxRevaluationReport> {
  return invokeIpc<FxRevaluationReport>(
    'get_fx_revaluation_report_cmd',
    { asOfDate: asOfDate || null, fxRate: fxRate || null },
    { label: 'get_fx_revaluation_report_cmd' }
  );
}

export function getHistoricalTrendsReportCmd(
  fromDate: string,
  toDate: string,
  fxRate?: number | null
): Promise<HistoricalTrendsReport> {
  return invokeIpc<HistoricalTrendsReport>(
    'get_historical_trends_report_cmd',
    { fromDate, toDate, fxRate: fxRate || null },
    { label: 'get_historical_trends_report_cmd' }
  );
}

export interface EnvelopeView {
  account_id: string;
  account_code: string;
  account_name: string;
  assigned: number;
  activity: number;
  available: number;
}

export interface BudgetMonthSummary {
  month: string;
  envelopes: EnvelopeView[];
  total_assigned: number;
  total_activity: number;
  to_be_budgeted: number;
}

export function getBudgetSummaryCmd(month: string): Promise<BudgetMonthSummary> {
  return invokeIpc<BudgetMonthSummary>(
    'get_budget_summary_cmd',
    { month },
    { label: 'get_budget_summary_cmd' }
  );
}




import { AppError } from './errors';
import { commands } from './bindings.gen';
import type {
  Account,
  AccountBalanceView,
  AccountRunningLedgerItem,
  AppStateView,
  AuditEntry,
  AuditIntegrityReport,
  AuditPage,
  BalanceSheetReport,
  BudgetAllocation,
  BudgetMonthSummary,
  CashFlowReport,
  CreateAccountInput,
  CreateJournalEntryInput,
  CreatePlanInput,
  CreateReconcileRuleInput,
  ReconcileRuleWithAccount,
  DashboardMetricsView,
  DueRecurringPlanView,
  FxRevaluationReport,
  HistoricalTrendsReport,
  JournalEntryView,
  LedgerTotalsView,
  MatchStatementOutput,
  MonthlyCashflowPoint,
  PaymentPlan,
  PlanProgressView,
  PostDueRecurringBatchInput,
  ProfitLossReport,
  ReconcileState,
  ReconciliationStatusView,
  RecordInstallmentInput,
  StatementRow,
  TrialBalanceReport,
  UpdateAccountInput,
  UpdateJournalEntryInput,
  UpdatePlanInput,
  UpsertBudgetInput,
  VaultHealthReport,
  VaultInspectionResult,
  VaultRegistryEntry,
} from './bindings.gen';

export * from './bindings.gen';

export type ReconcileStatus = ReconcileState;
export type PlanDto = PaymentPlan;
export type PlanProgressViewDto = PlanProgressView;

export async function unwrap<T, E>(
  promise: Promise<{ status: 'ok'; data: T } | { status: 'error'; error: E }>,
  timeoutMs = 60_000,
  label = 'IPC'
): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeoutPromise = new Promise<never>((_, reject) => {
    timer = setTimeout(() => {
      reject(new AppError('ERR_TIMEOUT', `Command '${label}' timed out after ${timeoutMs}ms`));
    }, timeoutMs);
  });

  try {
    const res = await Promise.race([promise, timeoutPromise]);
    if (res.status === 'ok') {
      return res.data;
    }
    throw AppError.fromUnknown(res.error);
  } catch (err) {
    throw AppError.fromUnknown(err, `Command '${label}' failed`);
  } finally {
    if (timer !== undefined) {
      clearTimeout(timer);
    }
  }
}

// =================== Auth & Vault Session ===================

export function getAppState(): Promise<AppStateView> {
  return unwrap(commands.getAppState(), 30_000, 'get_app_state');
}

export function unlockVault(password: string): Promise<AppStateView> {
  return unwrap(commands.unlock(password), 120_000, 'unlock');
}

export function lockVault(): Promise<AppStateView> {
  return unwrap(commands.lock(), 30_000, 'lock');
}

export function createVault(name: string, path: string): Promise<AppStateView> {
  return unwrap(commands.createVault(name, path), 30_000, 'create_vault');
}

export function createAccount(
  username: string,
  password: string,
  templateLanguage?: string,
  accountProfile?: string
): Promise<AppStateView> {
  return unwrap(
    commands.createAccount(username, password, templateLanguage ?? null, accountProfile ?? null),
    120_000,
    'create_account'
  );
}

export function inspectVaultFolder(path: string): Promise<VaultInspectionResult> {
  return unwrap(commands.inspectVaultFolder(path), 10_000, 'inspect_vault_folder');
}

export function importVault(path: string, password: string): Promise<AppStateView> {
  return unwrap(commands.importVault(path, password), 120_000, 'import_vault');
}

export function changePassword(oldPassword: string, newPassword: string): Promise<AppStateView> {
  return unwrap(commands.changePassword(oldPassword, newPassword), 120_000, 'change_password');
}

export function deleteVaultAndAccount(password: string): Promise<AppStateView> {
  return unwrap(commands.deleteVaultAndAccount(password), 30_000, 'delete_vault_and_account');
}

// =================== Security & Settings ===================

export function getBootId(): Promise<string> {
  return unwrap(commands.getBootId(), 10_000, 'get_boot_id');
}

export function setAutoLockMode(mode: string): Promise<AppStateView> {
  return unwrap(commands.setAutoLockMode(mode), 10_000, 'set_auto_lock_mode');
}

export function getKnownVaultsCmd(): Promise<VaultRegistryEntry[]> {
  return unwrap(commands.getKnownVaultsCmd(), 10_000, 'get_known_vaults_cmd');
}

export async function rememberKnownVaultCmd(entry: VaultRegistryEntry): Promise<void> {
  await unwrap(commands.rememberKnownVaultCmd(entry), 10_000, 'remember_known_vault_cmd');
}

export async function forgetKnownVaultCmd(idOrPath: string): Promise<void> {
  await unwrap(commands.forgetKnownVaultCmd(idOrPath), 10_000, 'forget_known_vault_cmd');
}

export function setActiveVault(path: string): Promise<AppStateView> {
  return unwrap(commands.setActiveVault(path), 10_000, 'set_active_vault');
}

export function renameUser(username: string): Promise<AppStateView> {
  return unwrap(commands.renameUser(username), 10_000, 'rename_user');
}

export function renameVault(name: string): Promise<AppStateView> {
  return unwrap(commands.renameVault(name), 10_000, 'rename_vault');
}

export async function openVaultFolder(): Promise<void> {
  await unwrap(commands.openVaultFolder(), 10_000, 'open_vault_folder');
}

export async function exportVaultBackupFolder(destDir: string): Promise<void> {
  await unwrap(commands.exportVaultBackupFolder(destDir), 30_000, 'export_vault_backup_folder');
}

export function diagnoseVaultHealthCmd(): Promise<VaultHealthReport> {
  return unwrap(commands.diagnoseVaultHealthCmd(), 30_000, 'diagnose_vault_health_cmd');
}

export function readStatementFileCmd(path: string): Promise<string> {
  return unwrap(commands.readStatementFileCmd(path), 30_000, 'read_statement_file_cmd');
}

export async function writeFileRawCmd(path: string, contents: number[]): Promise<void> {
  await unwrap(commands.writeFileRaw(path, contents), 30_000, 'write_file_raw');
}

export async function exportTextFileCmd(path: string, contents: string): Promise<void> {
  await unwrap(commands.exportTextFile(path, contents), 30_000, 'export_text_file');
}

// =================== Accounts ===================

export function createAccountCmd(input: CreateAccountInput): Promise<Account> {
  return unwrap(commands.createAccountCmd(input), 30_000, 'create_account_cmd');
}

export function updateAccountCmd(id: string, input: UpdateAccountInput): Promise<Account> {
  return unwrap(commands.updateAccountCmd(id, input), 30_000, 'update_account_cmd');
}

export async function deleteAccountCmd(id: string): Promise<void> {
  await unwrap(commands.deleteAccountCmd(id), 30_000, 'delete_account_cmd');
}

export function getAccountCmd(id: string): Promise<Account> {
  return unwrap(commands.getAccountCmd(id), 30_000, 'get_account_cmd');
}

export function listAccountsCmd(): Promise<AccountBalanceView[]> {
  return unwrap(commands.listAccountsCmd(), 30_000, 'list_accounts_cmd');
}

export function seedRootAccountsCmd(): Promise<Account[]> {
  return unwrap(commands.seedRootAccountsCmd(), 30_000, 'seed_root_accounts_cmd');
}

export function seedStarterAccountsCmd(language?: string, profile?: string): Promise<Account[]> {
  return unwrap(
    commands.seedStarterAccountsCmd(language ?? null, profile ?? null),
    30_000,
    'seed_starter_accounts_cmd'
  );
}

// =================== Ledger & Journal ===================

export function postJournalEntryCmd(input: CreateJournalEntryInput): Promise<JournalEntryView> {
  return unwrap(commands.postJournalEntryCmd(input), 30_000, 'post_journal_entry_cmd');
}

export function updateJournalEntryCmd(
  id: string,
  input: UpdateJournalEntryInput
): Promise<JournalEntryView> {
  return unwrap(commands.updateJournalEntryCmd(id, input), 30_000, 'update_journal_entry_cmd');
}

export async function deleteJournalEntryCmd(id: string): Promise<void> {
  await unwrap(commands.deleteJournalEntryCmd(id), 30_000, 'delete_journal_entry_cmd');
}

export function getJournalEntryCmd(id: string): Promise<JournalEntryView> {
  return unwrap(commands.getJournalEntryCmd(id), 30_000, 'get_journal_entry_cmd');
}

export function listJournalEntriesCmd(
  limit?: number,
  offset?: number
): Promise<JournalEntryView[]> {
  return unwrap(
    commands.listJournalEntriesCmd(limit ?? null, offset ?? null),
    30_000,
    'list_journal_entries_cmd'
  );
}

export function getAccountLedgerCmd(accountId: string): Promise<AccountRunningLedgerItem[]> {
  return unwrap(commands.getAccountLedgerCmd(accountId), 30_000, 'get_account_ledger_cmd');
}

export function getLedgerTotalsCmd(fxRate?: number): Promise<LedgerTotalsView> {
  return unwrap(commands.getLedgerTotalsCmd(fxRate ?? null), 30_000, 'get_ledger_totals_cmd');
}

export function getDashboardMetricsCmd(
  fxRate?: number,
  today?: string
): Promise<DashboardMetricsView> {
  return unwrap(
    commands.getDashboardMetricsCmd(fxRate ?? null, today ?? null),
    30_000,
    'get_dashboard_metrics_cmd'
  );
}

export function getClosingDateCmd(): Promise<string | null> {
  return unwrap(commands.getClosingDateCmd(), 30_000, 'get_closing_date_cmd');
}

export async function setClosingDateCmd(closingDate: string | null): Promise<void> {
  await unwrap(commands.setClosingDateCmd(closingDate), 30_000, 'set_closing_date_cmd');
}

export function exportBeancountCmd(path?: string | null): Promise<string> {
  return unwrap(commands.exportBeancountCmd(path ?? null), 30_000, 'export_beancount_cmd');
}

// =================== Plans ===================

export function createPlanCmd(input: CreatePlanInput): Promise<PaymentPlan> {
  return unwrap(commands.createPlanCmd(input), 30_000, 'create_plan_cmd');
}

export function updatePlanCmd(id: string, input: UpdatePlanInput): Promise<PaymentPlan> {
  return unwrap(commands.updatePlanCmd(id, input), 30_000, 'update_plan_cmd');
}

export async function deletePlanCmd(id: string): Promise<void> {
  await unwrap(commands.deletePlanCmd(id), 30_000, 'delete_plan_cmd');
}

export function getPlanCmd(id: string): Promise<PaymentPlan> {
  return unwrap(commands.getPlanCmd(id), 30_000, 'get_plan_cmd');
}

export function listPlansCmd(): Promise<PaymentPlan[]> {
  return unwrap(commands.listPlansCmd(), 30_000, 'list_plans_cmd');
}

export function listPlansWithProgressCmd(): Promise<PlanProgressView[]> {
  return unwrap(commands.listPlansWithProgressCmd(), 30_000, 'list_plans_with_progress_cmd');
}

export function recordPlanInstallmentCmd(input: RecordInstallmentInput): Promise<JournalEntryView> {
  return unwrap(commands.recordPlanInstallmentCmd(input), 30_000, 'record_plan_installment_cmd');
}

export function getDueRecurringPlansCmd(asOfDate?: string | null): Promise<DueRecurringPlanView[]> {
  return unwrap(
    commands.getDueRecurringPlansCmd(asOfDate ?? null),
    30_000,
    'get_due_recurring_plans_cmd'
  );
}

export function postDueRecurringBatchCmd(
  input: PostDueRecurringBatchInput
): Promise<JournalEntryView[]> {
  return unwrap(commands.postDueRecurringBatchCmd(input), 30_000, 'post_due_recurring_batch_cmd');
}

// =================== Budget ===================

export function upsertBudgetCmd(input: UpsertBudgetInput): Promise<BudgetAllocation> {
  return unwrap(commands.upsertBudgetCmd(input), 30_000, 'upsert_budget_cmd');
}

export async function deleteBudgetCmd(id: string): Promise<void> {
  await unwrap(commands.deleteBudgetCmd(id), 30_000, 'delete_budget_cmd');
}

export function getBudgetSummaryCmd(month: string): Promise<BudgetMonthSummary> {
  return unwrap(commands.getBudgetSummaryCmd(month), 30_000, 'get_budget_summary_cmd');
}

// =================== Reconciliation ===================

export function getReconciliationStatusCmd(accountId: string): Promise<ReconciliationStatusView> {
  return unwrap(
    commands.getReconciliationStatusCmd(accountId),
    30_000,
    'get_reconciliation_status_cmd'
  );
}

export async function setPostingReconciledCmd(postingId: string, status: string): Promise<void> {
  await unwrap(
    commands.setPostingReconciledCmd(postingId, status),
    30_000,
    'set_posting_reconciled_cmd'
  );
}

export async function bulkSetPostingsReconciledCmd(
  postingIds: string[],
  status: string
): Promise<void> {
  await unwrap(
    commands.bulkSetPostingsReconciledCmd(postingIds, status),
    30_000,
    'bulk_set_postings_reconciled_cmd'
  );
}

export async function finishReconciliationCmd(
  accountId: string,
  postingIds: string[]
): Promise<void> {
  await unwrap(
    commands.finishReconciliationCmd(accountId, postingIds),
    30_000,
    'finish_reconciliation_cmd'
  );
}

export function matchStatementCmd(
  accountId: string,
  statements: StatementRow[],
  autoClear: boolean
): Promise<MatchStatementOutput> {
  return unwrap(
    commands.matchStatementCmd(accountId, statements, autoClear),
    30_000,
    'match_statement_cmd'
  );
}

export function listReconcileRulesCmd(): Promise<ReconcileRuleWithAccount[]> {
  return unwrap(commands.listReconcileRulesCmd(), 30_000, 'list_reconcile_rules_cmd');
}

export function createReconcileRuleCmd(
  input: CreateReconcileRuleInput
): Promise<ReconcileRuleWithAccount> {
  return unwrap(commands.createReconcileRuleCmd(input), 30_000, 'create_reconcile_rule_cmd');
}

export async function deleteReconcileRuleCmd(ruleId: string): Promise<void> {
  await unwrap(commands.deleteReconcileRuleCmd(ruleId), 30_000, 'delete_reconcile_rule_cmd');
}

export function evaluateReconcileRulesCmd(statements: StatementRow[]): Promise<StatementRow[]> {
  return unwrap(
    commands.evaluateReconcileRulesCmd(statements),
    30_000,
    'evaluate_reconcile_rules_cmd'
  );
}

// =================== Reports ===================

export function getProfitLossReportCmd(
  fromDate?: string | null,
  toDate?: string | null
): Promise<ProfitLossReport> {
  return unwrap(
    commands.getProfitLossReportCmd(fromDate ?? null, toDate ?? null),
    30_000,
    'get_profit_loss_report_cmd'
  );
}

export function getBalanceSheetReportCmd(asOfDate?: string | null): Promise<BalanceSheetReport> {
  return unwrap(
    commands.getBalanceSheetReportCmd(asOfDate ?? null),
    30_000,
    'get_balance_sheet_report_cmd'
  );
}

export function getCashFlowReportCmd(
  fromDate?: string | null,
  toDate?: string | null
): Promise<CashFlowReport> {
  return unwrap(
    commands.getCashFlowReportCmd(fromDate ?? null, toDate ?? null),
    30_000,
    'get_cash_flow_report_cmd'
  );
}

export function getTrialBalanceReportCmd(asOfDate?: string | null): Promise<TrialBalanceReport> {
  return unwrap(
    commands.getTrialBalanceReportCmd(asOfDate ?? null),
    30_000,
    'get_trial_balance_report_cmd'
  );
}

export function getFxRevaluationReportCmd(
  asOfDate?: string | null,
  fxRate?: number | null
): Promise<FxRevaluationReport> {
  return unwrap(
    commands.getFxRevaluationReportCmd(asOfDate ?? null, fxRate ?? null),
    30_000,
    'get_fx_revaluation_report_cmd'
  );
}

export function getHistoricalTrendsReportCmd(
  fromDate: string,
  toDate: string,
  fxRate?: number | null
): Promise<HistoricalTrendsReport> {
  return unwrap(
    commands.getHistoricalTrendsReportCmd(fromDate, toDate, fxRate ?? null),
    30_000,
    'get_historical_trends_report_cmd'
  );
}

export function getMonthlyCashflowSummaryCmd(
  months?: number | null
): Promise<MonthlyCashflowPoint[]> {
  return unwrap(
    commands.getMonthlyCashflowSummaryCmd(months ?? 6),
    30_000,
    'get_monthly_cashflow_summary_cmd'
  );
}

// =================== Audit ===================

export function getAuditLogCmd(page = 1, perPage = 50): Promise<AuditPage> {
  return unwrap(commands.getAuditLogCmd(page, perPage), 30_000, 'get_audit_log_cmd');
}

export function getEntityAuditLogCmd(entityType: string, entityId: string): Promise<AuditEntry[]> {
  return unwrap(
    commands.getEntityAuditLogCmd(entityType, entityId),
    30_000,
    'get_entity_audit_log_cmd'
  );
}

export function verifyAuditLogIntegrityCmd(): Promise<AuditIntegrityReport> {
  return unwrap(commands.verifyAuditLogIntegrityCmd(), 30_000, 'verify_audit_log_integrity_cmd');
}

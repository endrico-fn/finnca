import { getPref } from '$lib/core/state/prefs';
import {
  getFxRevaluationReportCmd,
  getHistoricalTrendsReportCmd,
  getProfitLossReportCmd,
  getBalanceSheetReportCmd,
  getCashFlowReportCmd,
  getTrialBalanceReportCmd,
  getBudgetSummaryCmd,
  type FxRevaluationItem,
  type FxRevaluationReport,
  type DailyTrendPoint,
  type HistoricalTrendsReport,
  type AccountReportRow,
  type ProfitLossReport,
  type BalanceSheetReport,
  type CashFlowActivityRow,
  type CashFlowReport,
  type TrialBalanceRow,
  type TrialBalanceReport,
  type BudgetMonthSummary,
} from '$lib/core/ipc/bindings';

export type {
  FxRevaluationItem,
  FxRevaluationReport,
  DailyTrendPoint,
  HistoricalTrendsReport,
  AccountReportRow,
  ProfitLossReport,
  BalanceSheetReport,
  CashFlowActivityRow,
  CashFlowReport,
  TrialBalanceRow,
  TrialBalanceReport,
  BudgetMonthSummary,
};

export type ReportTab =
  | 'bs'
  | 'pnl'
  | 'cashflow'
  | 'tb'
  | 'budget-actual'
  | 'debt'
  | 'trends'
  | 'fx';

export type ReportGroup = 'statements' | 'budget-debt' | 'analysis';

export interface DailyDataPoint {
  date: string;
  netWorth: number;
  assets: number;
  liabilities: number;
  liquidCash: number;
}

class ReportState {
  activeTab = $state<ReportTab>('bs');
  activeGroup = $state<ReportGroup>('statements');
  startDate = $state('');
  endDate = $state('');
  asOfDate = $state('');
  loading = $state(false);
  error = $state<string | null>(null);

  trialBalance = $state<TrialBalanceReport | null>(null);
  profitLoss = $state<ProfitLossReport | null>(null);
  balanceSheet = $state<BalanceSheetReport | null>(null);
  cashFlow = $state<CashFlowReport | null>(null);
  fxRevaluation = $state<FxRevaluationReport | null>(null);
  historicalTrends = $state<HistoricalTrendsReport | null>(null);
  budgetSummary = $state<BudgetMonthSummary | null>(null);

  historicalPoints = $derived<DailyDataPoint[]>(
    (this.historicalTrends?.points ?? []).map((p) => ({
      date: p.date,
      netWorth: p.net_worth,
      assets: p.assets,
      liabilities: p.liabilities,
      liquidCash: p.liquid_cash,
    }))
  );

  async loadTrialBalance(asOfDate?: string): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const date = asOfDate ?? this.asOfDate;
      this.trialBalance = await getTrialBalanceReportCmd(date || null);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  async loadProfitLoss(fromDate?: string, toDate?: string): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.profitLoss = await getProfitLossReportCmd(
        fromDate ?? (this.startDate || null),
        toDate ?? (this.endDate || null)
      );
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  async loadBalanceSheet(asOfDate?: string): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const date = asOfDate ?? this.asOfDate;
      this.balanceSheet = await getBalanceSheetReportCmd(date || null);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  async loadCashFlow(fromDate?: string, toDate?: string): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.cashFlow = await getCashFlowReportCmd(
        fromDate ?? (this.startDate || null),
        toDate ?? (this.endDate || null)
      );
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  async loadBudgetSummary(month?: string): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const targetMonth = month || new Date().toISOString().slice(0, 7);
      this.budgetSummary = await getBudgetSummaryCmd(targetMonth);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  async loadFxRevaluation(asOfDate?: string, fxRate?: number): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const activeRate = fxRate ?? getPref('finnca_fx_rate', 16000);
      this.fxRevaluation = await getFxRevaluationReportCmd(
        asOfDate ?? (this.asOfDate || null),
        activeRate
      );
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  async loadHistoricalTrends(
    fromDate?: string,
    toDate?: string,
    fxRate?: number
  ): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const activeRate = fxRate ?? getPref('finnca_fx_rate', 16000);
      this.historicalTrends = await getHistoricalTrendsReportCmd(
        fromDate ?? this.startDate,
        toDate ?? this.endDate,
        activeRate
      );
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  setTab(tab: ReportTab) {
    this.activeTab = tab;
  }
}

export const reportState = new ReportState();

import { fxState } from '$lib/core/state/fx.svelte';
import {
  getFxRevaluationReportCmd,
  getHistoricalTrendsReportCmd,
  getProfitLossReportCmd,
  getBalanceSheetReportCmd,
  getTrialBalanceReportCmd,
  getCashFlowReportCmd,
  getMonthlyCashflowSummaryCmd,
  type FxRevaluationItem,
  type FxRevaluationReport,
  type DailyTrendPoint,
  type HistoricalTrendsReport,
  type AccountReportRow,
  type ProfitLossReport,
  type BalanceSheetReport,
  type TrialBalanceRow,
  type TrialBalanceReport,
  type CashFlowReport,
  type CashFlowActivityRow,
  type MonthlyCashflowPoint,
} from '$lib/core/ipc/bindings';

export type {
  FxRevaluationItem,
  FxRevaluationReport,
  DailyTrendPoint,
  HistoricalTrendsReport,
  AccountReportRow,
  ProfitLossReport,
  BalanceSheetReport,
  TrialBalanceRow,
  TrialBalanceReport,
  CashFlowReport,
  CashFlowActivityRow,
  MonthlyCashflowPoint,
};

export type ReportTab =
  | 'bs'
  | 'pnl'
  | 'cashflow'
  | 'tb'
  | 'debt'
  | 'trends'
  | 'spending'
  | 'forecast'
  | 'fx'
  | 'networth'
  | 'income-exp';

export type ReportGroup =
  | 'statements'
  | 'liquidity'
  | 'forecast-intelligence'
  | 'forecast-debt'
  | 'intelligence'
  | 'budget-debt'
  | 'analysis'
  | 'overview';

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
  fxRevaluation = $state<FxRevaluationReport | null>(null);
  historicalTrends = $state<HistoricalTrendsReport | null>(null);
  cashFlow = $state<CashFlowReport | null>(null);
  monthlyCashflow = $state<MonthlyCashflowPoint[]>([]);

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
      const from = fromDate ?? (this.startDate || null);
      const to = toDate ?? (this.endDate || null);
      const [report, monthly] = await Promise.all([
        getCashFlowReportCmd(from, to),
        getMonthlyCashflowSummaryCmd(6),
      ]);
      this.cashFlow = report;
      this.monthlyCashflow = monthly;
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
      const activeRate = fxRate ?? fxState.rate;
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

  async loadHistoricalTrends(fromDate?: string, toDate?: string, fxRate?: number): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const activeRate = fxRate ?? fxState.rate;
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

  async loadMonthlyCashflow(from?: string, to?: string): Promise<void> {
    try {
      const months = from && to ? undefined : 12;
      const data = await getMonthlyCashflowSummaryCmd(months);
      this.monthlyCashflow = data;
    } catch {
      this.monthlyCashflow = [];
    }
  }

  setTab(tab: ReportTab) {
    this.activeTab = tab;
  }
}

export const reportState = new ReportState();

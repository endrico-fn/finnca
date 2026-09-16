import { invokeIpc } from '$lib/core/ipc/client';
import {
  getFxRevaluationReportCmd,
  getHistoricalTrendsReportCmd,
  type FxRevaluationItem,
  type FxRevaluationReport,
  type DailyTrendPoint,
  type HistoricalTrendsReport,
} from '$lib/core/ipc/bindings';

export type { FxRevaluationItem, FxRevaluationReport, DailyTrendPoint, HistoricalTrendsReport };
export type ReportTab = 'tb' | 'trends' | 'debt' | 'cashflow' | 'fx';

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

export interface DailyDataPoint {
  date: string;
  netWorth: number;
  assets: number;
  liabilities: number;
  liquidCash: number;
}

class ReportState {
  activeTab = $state<ReportTab>('tb');
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
      this.trialBalance = await invokeIpc<TrialBalanceReport>('get_trial_balance_report_cmd', {
        asOfDate: date || null,
      });
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
      this.profitLoss = await invokeIpc<ProfitLossReport>('get_profit_loss_report_cmd', {
        fromDate: fromDate ?? (this.startDate || null),
        toDate: toDate ?? (this.endDate || null),
      });
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
      this.balanceSheet = await invokeIpc<BalanceSheetReport>('get_balance_sheet_report_cmd', {
        asOfDate: date || null,
      });
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
      this.cashFlow = await invokeIpc<CashFlowReport>('get_cash_flow_report_cmd', {
        fromDate: fromDate ?? (this.startDate || null),
        toDate: toDate ?? (this.endDate || null),
      });
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
      this.fxRevaluation = await getFxRevaluationReportCmd(
        asOfDate ?? (this.asOfDate || null),
        fxRate ?? null
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
      this.historicalTrends = await getHistoricalTrendsReportCmd(
        fromDate ?? this.startDate,
        toDate ?? this.endDate,
        fxRate ?? null
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

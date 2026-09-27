import type { ReportGroup, ReportTab } from './report.svelte';

export type ReportGroupLabelKey =
  | 'reportGroupStatements'
  | 'reportGroupCashLiquidity'
  | 'reportGroupForecastDebt'
  | 'reportGroupIntelligenceFx'
  | 'reportGroupBudgetDebt'
  | 'reportGroupAnalysis'
  | 'reportGroupOverview';

export type ReportTabLabelKey =
  | 'balanceSheetTitle'
  | 'pnlTitle'
  | 'cashFlowTitle'
  | 'trialBalanceTitle'
  | 'debtReportTitle'
  | 'trendsTitle'
  | 'spendingTitle'
  | 'cashForecastTitle'
  | 'fxRevalTab'
  | 'networthProgressionTitle'
  | 'incomeExpChartTitle';

export interface ReportGroupMeta {
  id: ReportGroup;
  labelKey: ReportGroupLabelKey;
  defaultTab: ReportTab;
  tabs: ReportTab[];
}

export const REPORT_GROUPS: ReportGroupMeta[] = [
  {
    id: 'statements',
    labelKey: 'reportGroupStatements',
    defaultTab: 'tb',
    tabs: ['tb', 'pnl', 'bs'],
  },
  {
    id: 'liquidity',
    labelKey: 'reportGroupCashLiquidity',
    defaultTab: 'cashflow',
    tabs: ['cashflow', 'income-exp', 'spending'],
  },
  {
    id: 'forecast-debt',
    labelKey: 'reportGroupForecastDebt',
    defaultTab: 'forecast',
    tabs: ['forecast', 'debt'],
  },
  {
    id: 'intelligence',
    labelKey: 'reportGroupIntelligenceFx',
    defaultTab: 'trends',
    tabs: ['trends', 'fx', 'networth'],
  },
];

export const REPORT_TAB_LABELS: Record<ReportTab, ReportTabLabelKey> = {
  networth: 'networthProgressionTitle',
  'income-exp': 'incomeExpChartTitle',
  bs: 'balanceSheetTitle',
  pnl: 'pnlTitle',
  cashflow: 'cashFlowTitle',
  tb: 'trialBalanceTitle',
  debt: 'debtReportTitle',
  trends: 'trendsTitle',
  spending: 'spendingTitle',
  forecast: 'cashForecastTitle',
  fx: 'fxRevalTab',
};

export function reportGroupOf(tab: ReportTab): ReportGroup {
  return REPORT_GROUPS.find((g) => g.tabs.includes(tab))?.id ?? 'statements';
}

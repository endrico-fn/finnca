import type { ReportGroup, ReportTab } from './report.svelte';

export type ReportGroupLabelKey =
  | 'reportGroupStatements'
  | 'reportGroupCashLiquidity'
  | 'reportGroupForecastIntelligence'
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
  | 'fxRevaluationTitle'
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
    id: 'forecast-intelligence',
    labelKey: 'reportGroupForecastIntelligence',
    defaultTab: 'forecast',
    tabs: ['forecast', 'debt', 'trends', 'fx', 'networth'],
  },
];

export const REPORT_TAB_LABELS: Record<ReportTab, ReportTabLabelKey> = {
  tb: 'trialBalanceTitle',
  pnl: 'pnlTitle',
  bs: 'balanceSheetTitle',
  cashflow: 'cashFlowTitle',
  'income-exp': 'incomeExpChartTitle',
  spending: 'spendingTitle',
  forecast: 'cashForecastTitle',
  debt: 'debtReportTitle',
  trends: 'trendsTitle',
  fx: 'fxRevaluationTitle',
  networth: 'networthProgressionTitle',
};

export const REPORT_SHORTCUTS: Record<ReportTab, string> = {
  tb: '1',
  pnl: '2',
  bs: '3',
  cashflow: '4',
  'income-exp': '5',
  spending: '6',
  forecast: '7',
  debt: '8',
  trends: '9',
  fx: '0',
  networth: '-',
};

export const ORDERED_REPORT_TABS: ReportTab[] = [
  'tb',
  'pnl',
  'bs',
  'cashflow',
  'income-exp',
  'spending',
  'forecast',
  'debt',
  'trends',
  'fx',
  'networth',
];

export function reportGroupOf(tab: ReportTab): ReportGroup {
  return REPORT_GROUPS.find((g) => g.tabs.includes(tab))?.id ?? 'statements';
}

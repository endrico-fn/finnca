import { pickSaveFile } from '$lib/core/dialog';
import { exportTextFileCmd } from '$lib/core/ipc/bindings';
import type {
  TrialBalanceReport,
  BalanceSheetReport,
  ProfitLossReport,
  CashFlowReport,
} from '$lib/features/report/state/report.svelte';

export async function exportReportCSV(
  tab: string,
  historicalPoints: Array<{
    date: string;
    netWorth: number;
    assets: number;
    liabilities: number;
    liquidCash: number;
  }>,
  trialBalance?: TrialBalanceReport | null,
  defaultFilename: string = 'report.csv',
  csvFilterLabel: string = 'CSV Document',
  balanceSheet?: BalanceSheetReport | null,
  profitLoss?: ProfitLossReport | null,
  cashFlow?: CashFlowReport | null
): Promise<string | null> {
  const csvRows: string[] = [];

  if (tab === 'trends') {
    csvRows.push('DATE,NET_WORTH_IDR,ASSETS_IDR,LIABILITIES_IDR,LIQUID_CASH_IDR');
    for (const pt of historicalPoints) {
      csvRows.push(`${pt.date},${pt.netWorth},${pt.assets},${pt.liabilities},${pt.liquidCash}`);
    }
  } else if (tab === 'cashflow' && cashFlow) {
    csvRows.push('DATE,SECTION,CATEGORY,DESCRIPTION,CONTRA_ACCOUNT,AMOUNT_IDR');
    for (const r of cashFlow.operating_rows) {
      csvRows.push(
        `"${r.date}","OPERATING","${r.category}","${r.description}","${r.account_name}",${r.amount}`
      );
    }
    for (const r of cashFlow.investing_rows) {
      csvRows.push(
        `"${r.date}","INVESTING","${r.category}","${r.description}","${r.account_name}",${r.amount}`
      );
    }
    for (const r of cashFlow.financing_rows) {
      csvRows.push(
        `"${r.date}","FINANCING","${r.category}","${r.description}","${r.account_name}",${r.amount}`
      );
    }
    csvRows.push(`"","SUMMARY","STARTING_CASH","STARTING CASH","",${cashFlow.starting_cash}`);
    csvRows.push(
      `"","SUMMARY","OPERATING_CASH_FLOW","OPERATING CASH FLOW","",${cashFlow.operating_cash_flow}`
    );
    csvRows.push(
      `"","SUMMARY","INVESTING_CASH_FLOW","INVESTING CASH FLOW","",${cashFlow.investing_cash_flow}`
    );
    csvRows.push(
      `"","SUMMARY","FINANCING_CASH_FLOW","FINANCING CASH FLOW","",${cashFlow.financing_cash_flow}`
    );
    csvRows.push(`"","SUMMARY","NET_CASH_CHANGE","NET CASH CHANGE","",${cashFlow.net_cash_change}`);
    csvRows.push(`"","SUMMARY","ENDING_CASH","ENDING CASH","",${cashFlow.ending_cash}`);
  } else if (tab === 'bs' && balanceSheet) {
    csvRows.push('SECTION,CODE,ACCOUNT_NAME,AMOUNT_IDR');
    for (const r of balanceSheet.asset_rows) {
      csvRows.push(`"ASSET","${r.code}","${r.name}",${r.amount}`);
    }
    for (const r of balanceSheet.liability_rows) {
      csvRows.push(`"LIABILITY","${r.code}","${r.name}",${r.amount}`);
    }
    for (const r of balanceSheet.equity_rows) {
      csvRows.push(`"EQUITY","${r.code}","${r.name}",${r.amount}`);
    }
    csvRows.push(`"TOTAL_ASSETS","","TOTAL ASSETS",${balanceSheet.total_assets}`);
    csvRows.push(`"TOTAL_LIABILITIES","","TOTAL LIABILITIES",${balanceSheet.total_liabilities}`);
    csvRows.push(`"TOTAL_EQUITY","","TOTAL EQUITY",${balanceSheet.total_equity}`);
    csvRows.push(`"NET_INCOME","","NET INCOME",${balanceSheet.net_income}`);
    csvRows.push(`"DISCREPANCY","","DISCREPANCY",${balanceSheet.discrepancy}`);
  } else if (tab === 'pnl' && profitLoss) {
    csvRows.push('SECTION,CODE,ACCOUNT_NAME,AMOUNT_IDR');
    for (const r of profitLoss.income_rows) {
      csvRows.push(`"REVENUE","${r.code}","${r.name}",${r.amount}`);
    }
    for (const r of profitLoss.expense_rows) {
      csvRows.push(`"EXPENSE","${r.code}","${r.name}",${r.amount}`);
    }
    csvRows.push(`"TOTAL_REVENUES","","TOTAL REVENUES",${profitLoss.total_income}`);
    csvRows.push(`"TOTAL_EXPENSES","","TOTAL EXPENSES",${profitLoss.total_expenses}`);
    csvRows.push(`"NET_INCOME","","NET INCOME",${profitLoss.net_income}`);
  } else {
    csvRows.push('CODE,ACCOUNT_NAME,TYPE,DEBIT_IDR,CREDIT_IDR');
    const rows = trialBalance?.rows ?? [];
    for (const row of rows) {
      csvRows.push(`"${row.code}","${row.name}","${row.account_type}",${row.debit},${row.credit}`);
    }
    if (trialBalance) {
      csvRows.push(
        `,,,TOTAL_DEBIT,${trialBalance.total_debit},TOTAL_CREDIT,${trialBalance.total_credit}`
      );
    }
  }

  const csvContent = csvRows.join('\n');

  try {
    const selectedPath = await pickSaveFile(defaultFilename, [
      { name: csvFilterLabel, extensions: ['csv'] },
    ]);

    if (selectedPath) {
      await exportTextFileCmd(selectedPath, csvContent);
      return selectedPath;
    }
    return null;
  } catch {
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.setAttribute('href', url);
    link.setAttribute('download', defaultFilename);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
    return defaultFilename;
  }
}

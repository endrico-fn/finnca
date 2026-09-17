import { save } from '@tauri-apps/plugin-dialog';
import { writeFileRawCmd, exportTextFileCmd } from '$lib/core/ipc/bindings';
import pdfMake from 'pdfmake/build/pdfmake';
import Papa from 'papaparse';
import type { TDocumentDefinitions, Margins, Content } from 'pdfmake/interfaces';
import { pdfMakeVfs } from './vfsFonts';
import { APP_NAME } from '$lib/core/types';
import { formatMinorToDisplay } from '$lib/core/format/currency';
import type {
  TrialBalanceReport,
  ProfitLossReport,
  BalanceSheetReport,
  ReportTab,
} from '$lib/features/report/state/report.svelte';
import { i18n } from '$lib/core/i18n.svelte';

(pdfMake as unknown as { vfs: unknown; fonts: unknown }).vfs = pdfMakeVfs;
(pdfMake as unknown as { vfs: unknown; fonts: unknown }).fonts = {
  ProtoMono: {
    normal: 'ProtoMono.ttf',
    bold: 'ProtoMono.ttf',
    italics: 'ProtoMono.ttf',
    bolditalics: 'ProtoMono.ttf',
  },
};

const right = 'right' as const;

export interface StatementRow {
  date: string;
  description: string;
  debit: number;
  credit: number;
  balance: number;
}

export interface ReportPDFOptions {
  vaultName: string;
  reportTitle: string;
  period: string;
  tab: ReportTab;
  trialBalance?: TrialBalanceReport | null;
  profitLoss?: ProfitLossReport | null;
  balanceSheet?: BalanceSheetReport | null;
}

async function savePDF(docDefinition: TDocumentDefinitions, defaultPath: string) {
  const filePath = await save({
    filters: [{ name: 'PDF Document', extensions: ['pdf'] }],
    defaultPath,
  });
  if (!filePath) return;

  const pdfDocGenerator = pdfMake.createPdf(docDefinition);
  const base64 = await new Promise<string>((resolve) => {
    // @ts-expect-error - The types say 0 args, but the browser implementation requires a callback
    pdfDocGenerator.getBase64((data: string) => resolve(data));
  });

  const binaryString = window.atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  await writeFileRawCmd(filePath, Array.from(bytes));
}

function buildReportPDFBody(opts: ReportPDFOptions): Content[] {
  const fmt = (minor: number) => formatMinorToDisplay(minor, 'IDR');
  const { tab, trialBalance, profitLoss, balanceSheet } = opts;

  if (tab === 'tb' && trialBalance) {
    const tbRows = trialBalance.rows.map((r) => [
      { text: r.code, style: 'bodyRow' },
      { text: r.name, style: 'bodyRow' },
      { text: r.debit > 0 ? fmt(r.debit) : '', style: 'bodyRow', alignment: right },
      { text: r.credit > 0 ? fmt(r.credit) : '', style: 'bodyRow', alignment: right },
    ]);
    tbRows.push([
      { text: '', style: 'bodyRow' },
      { text: i18n.t.totals, style: 'total' },
      { text: fmt(trialBalance.total_debit), style: 'total', alignment: right },
      { text: fmt(trialBalance.total_credit), style: 'total', alignment: right },
    ]);
    return [
      {
        table: {
          headerRows: 1,
          widths: [60, '*', 120, 120],
          body: [
            [
              { text: i18n.t.colCode, style: 'tableHeader' },
              { text: i18n.t.name, style: 'tableHeader' },
              { text: i18n.t.debit, style: 'tableHeader', alignment: right },
              { text: i18n.t.credit, style: 'tableHeader', alignment: right },
            ],
            ...tbRows,
          ],
        },
        layout: 'lightHorizontalLines',
      },
    ];
  }

  if (profitLoss) {
    const incomeRows = profitLoss.income_rows.map((r) => [
      { text: `  ${r.code}  ${r.name}`, style: 'bodyRow' },
      { text: fmt(Math.abs(r.amount)), style: 'bodyRow', alignment: right },
    ]);
    const expenseRows = profitLoss.expense_rows.map((r) => [
      { text: `  ${r.code}  ${r.name}`, style: 'bodyRow' },
      { text: fmt(Math.abs(r.amount)), style: 'bodyRow', alignment: right },
    ]);
    const margin4: Margins = [0, 4, 0, 4];
    return [
      { text: i18n.t.revenues, style: 'section', margin: [0, 0, 0, 4] as Margins },
      {
        table: {
          widths: ['*', 120],
          body: incomeRows.length
            ? incomeRows
            : [
                [
                  { text: '  —', style: 'bodyRow' },
                  { text: '0', style: 'bodyRow', alignment: right },
                ],
              ],
        },
        layout: 'noBorders',
      },
      { text: ' ', margin: margin4 },
      { text: i18n.t.operationalExpenses, style: 'section', margin: [0, 4, 0, 4] as Margins },
      {
        table: {
          widths: ['*', 120],
          body: expenseRows.length
            ? expenseRows
            : [
                [
                  { text: '  —', style: 'bodyRow' },
                  { text: '0', style: 'bodyRow', alignment: right },
                ],
              ],
        },
        layout: 'noBorders',
      },
      {
        canvas: [{ type: 'line', x1: 0, y1: 4, x2: 515, y2: 4, lineWidth: 0.5, lineColor: '#333' }],
      },
      {
        columns: [
          { text: i18n.t.netIncome, style: 'total' },
          { text: fmt(profitLoss.net_income), style: 'total', alignment: right },
        ],
        margin: [0, 8, 0, 4] as Margins,
      },
    ];
  }

  if (balanceSheet) {
    const sections = [
      { label: i18n.t.assetsTitle, rows: balanceSheet.asset_rows },
      { label: i18n.t.liabilitiesTitle, rows: balanceSheet.liability_rows },
      { label: i18n.t.equityTitle, rows: balanceSheet.equity_rows },
    ];
    const rows: Content[] = [];
    for (const sec of sections) {
      rows.push({ text: sec.label, style: 'section', margin: [0, 6, 0, 4] as Margins });
      const accountRows = sec.rows.map((r) => [
        { text: `  ${r.code}  ${r.name}`, style: 'bodyRow' },
        { text: fmt(Math.abs(r.amount)), style: 'bodyRow', alignment: right },
      ]);
      if (accountRows.length) {
        rows.push({ table: { widths: ['*', 120], body: accountRows }, layout: 'noBorders' });
      }
    }
    rows.push({
      canvas: [{ type: 'line', x1: 0, y1: 4, x2: 515, y2: 4, lineWidth: 0.5, lineColor: '#333' }],
    });
    rows.push({
      columns: [
        {
          text: balanceSheet.is_balanced
            ? `A = L + E  ✓ ${i18n.t.auditBalanced}`
            : `A = L + E  ✗ ${i18n.t.auditUnbalanced}`,
          style: 'total',
        },
        { text: fmt(balanceSheet.total_assets), style: 'total', alignment: right },
      ],
      margin: [0, 8, 0, 0] as Margins,
    });
    return rows;
  }

  return [{ text: i18n.t.reportNoPdf.replace('{tab}', tab), style: 'bodyRow' }];
}

export async function exportReportPDF(opts: ReportPDFOptions) {
  try {
    const margin4: Margins = [0, 4, 0, 4];
    const docDefinition: TDocumentDefinitions = {
      pageSize: 'A4',
      pageOrientation: 'landscape',
      pageMargins: [50, 50, 50, 50],
      defaultStyle: { font: 'ProtoMono', fontSize: 9 },
      content: [
        {
          columns: [
            {
              stack: [
                { text: APP_NAME, style: 'brandTitle' },
                { text: i18n.t.exportReportTitle, style: 'brandTitle' },
              ],
              width: 'auto',
            },
            { text: '', width: '*' },
            {
              stack: [
                { text: opts.vaultName.toUpperCase(), style: 'brandTitle', alignment: right },
              ],
              width: 'auto',
            },
          ],
          margin: [0, 0, 0, 16] as Margins,
        },
        {
          canvas: [{ type: 'line', x1: 0, y1: 0, x2: 742, y2: 0, lineWidth: 1, lineColor: '#222' }],
        },
        {
          columns: [
            { text: opts.reportTitle.toUpperCase(), style: 'reportTitle' },
            {
              text: `${i18n.t.exportPeriod}: ${opts.period || i18n.t.allTime}`,
              style: 'reportTitle',
              alignment: right,
            },
          ],
          margin: [0, 10, 0, 20] as Margins,
        },
        ...buildReportPDFBody(opts),
      ],
      styles: {
        brandTitle: { fontSize: 14, bold: true, color: '#111' },
        reportTitle: { fontSize: 11, bold: true, color: '#111' },
        section: { fontSize: 10, bold: true, color: '#111' },
        total: { fontSize: 10, bold: true, color: '#111' },
        bodyRow: { fontSize: 9, color: '#333' },
        tableHeader: { bold: true, fontSize: 8, fillColor: '#eeeeee', margin: margin4 },
      },
    };

    const slug = opts.reportTitle.replace(/\s+/g, '_').toLowerCase();
    await savePDF(docDefinition, `finnca_${slug}_${opts.period || 'alltime'}.pdf`);
  } catch (err) {
    console.error('Export Report PDF error:', err);
  }
}

export async function exportAccountStatementCSV(
  account: { name: string; currency: string },
  rows: StatementRow[],
  period: string
) {
  try {
    const filePath = await save({
      filters: [{ name: 'CSV Document', extensions: ['csv'] }],
      defaultPath: `${account.name}_Statement_${period.replace(/\s/g, '_')}.csv`,
    });
    if (!filePath) return;

    const csvData = rows.map((row) => ({
      [i18n.t.date]: row.date,
      [i18n.t.description]: row.description,
      [i18n.t.debit]: row.debit !== 0 ? formatMinorToDisplay(row.debit, account.currency) : '',
      [i18n.t.credit]: row.credit !== 0 ? formatMinorToDisplay(row.credit, account.currency) : '',
      [i18n.t.colBalance]: formatMinorToDisplay(row.balance, account.currency),
    }));

    const csvStr = Papa.unparse(csvData);
    const bytes = new TextEncoder().encode(csvStr);
    await writeFileRawCmd(filePath, Array.from(bytes));
  } catch (err) {
    console.error('Export CSV error:', err);
  }
}

export async function exportReportCSV(
  tab: string,
  historicalPoints: Array<{ date: string; netWorth: number; assets: number; liabilities: number; liquidCash: number }>,
  trialBalance?: TrialBalanceReport | null,
  defaultFilename: string = 'report.csv',
  csvFilterLabel: string = 'CSV Document',
  balanceSheet?: BalanceSheetReport | null,
  profitLoss?: ProfitLossReport | null
): Promise<string | null> {
  const csvRows: string[] = [];

  if (tab === 'trends') {
    csvRows.push('DATE,NET_WORTH_IDR,ASSETS_IDR,LIABILITIES_IDR,LIQUID_CASH_IDR');
    for (const pt of historicalPoints) {
      csvRows.push(`${pt.date},${pt.netWorth},${pt.assets},${pt.liabilities},${pt.liquidCash}`);
    }
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
      csvRows.push(
        `"${row.code}","${row.name}","${row.account_type}",${row.debit},${row.credit}`
      );
    }
    if (trialBalance) {
      csvRows.push(
        `,,,TOTAL_DEBIT,${trialBalance.total_debit},TOTAL_CREDIT,${trialBalance.total_credit}`
      );
    }
  }

  const csvContent = csvRows.join('\n');

  try {
    const selectedPath = await save({
      defaultPath: defaultFilename,
      filters: [{ name: csvFilterLabel, extensions: ['csv'] }],
    });

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


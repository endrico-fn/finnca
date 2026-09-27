import { pickSaveFile } from '$lib/core/dialog';
import { writeFileRawCmd } from '$lib/core/ipc/bindings';
import pdfMake from 'pdfmake/build/pdfmake';
import type { TDocumentDefinitions, Margins, Content } from 'pdfmake/interfaces';
import { pdfMakeVfs } from './vfsFonts';
import { APP_NAME, APP_SLUG } from '$lib/core/types';
import { formatMinorToDisplay } from '$lib/core/format/currency';
import type {
  TrialBalanceReport,
  ProfitLossReport,
  BalanceSheetReport,
  CashFlowReport,
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

const PDF_INK = {
  lineSoft: '#333',
  lineStrong: '#222',
  textStrong: '#111',
  textBody: '#333',
  headerFill: '#eeeeee',
} as const;

export interface ReportPDFOptions {
  vaultName: string;
  reportTitle: string;
  period: string;
  tab: ReportTab;
  trialBalance?: TrialBalanceReport | null;
  profitLoss?: ProfitLossReport | null;
  balanceSheet?: BalanceSheetReport | null;
  cashFlow?: CashFlowReport | null;
}

async function savePDF(docDefinition: TDocumentDefinitions, defaultPath: string) {
  const filePath = await pickSaveFile(defaultPath, [{ name: 'PDF Document', extensions: ['pdf'] }]);
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
  const { tab, trialBalance, profitLoss, balanceSheet, cashFlow } = opts;

  if (tab === 'cashflow' && cashFlow) {
    const allRows = [
      ...cashFlow.operating_rows.map((r) => [
        { text: r.date, style: 'bodyRow' },
        { text: 'OPERATING', style: 'bodyRow' },
        { text: r.description, style: 'bodyRow' },
        { text: r.account_name, style: 'bodyRow' },
        { text: (r.amount >= 0 ? '+' : '') + fmt(r.amount), style: 'bodyRow', alignment: right },
      ]),
      ...cashFlow.investing_rows.map((r) => [
        { text: r.date, style: 'bodyRow' },
        { text: 'INVESTING', style: 'bodyRow' },
        { text: r.description, style: 'bodyRow' },
        { text: r.account_name, style: 'bodyRow' },
        { text: (r.amount >= 0 ? '+' : '') + fmt(r.amount), style: 'bodyRow', alignment: right },
      ]),
      ...cashFlow.financing_rows.map((r) => [
        { text: r.date, style: 'bodyRow' },
        { text: 'FINANCING', style: 'bodyRow' },
        { text: r.description, style: 'bodyRow' },
        { text: r.account_name, style: 'bodyRow' },
        { text: (r.amount >= 0 ? '+' : '') + fmt(r.amount), style: 'bodyRow', alignment: right },
      ]),
    ];

    return [
      {
        table: {
          widths: ['*', 120],
          body: [
            [
              { text: i18n.t.startingCash, style: 'bodyRow' },
              { text: fmt(cashFlow.starting_cash), style: 'bodyRow', alignment: right },
            ],
            [
              { text: i18n.t.operatingCashFlow, style: 'bodyRow' },
              {
                text:
                  (cashFlow.operating_cash_flow >= 0 ? '+' : '') +
                  fmt(cashFlow.operating_cash_flow),
                style: 'bodyRow',
                alignment: right,
              },
            ],
            [
              { text: i18n.t.investingCashFlow, style: 'bodyRow' },
              {
                text:
                  (cashFlow.investing_cash_flow >= 0 ? '+' : '') +
                  fmt(cashFlow.investing_cash_flow),
                style: 'bodyRow',
                alignment: right,
              },
            ],
            [
              { text: i18n.t.financingCashFlow, style: 'bodyRow' },
              {
                text:
                  (cashFlow.financing_cash_flow >= 0 ? '+' : '') +
                  fmt(cashFlow.financing_cash_flow),
                style: 'bodyRow',
                alignment: right,
              },
            ],
            [
              { text: i18n.t.netCashFlow, style: 'total' },
              {
                text: (cashFlow.net_cash_change >= 0 ? '+' : '') + fmt(cashFlow.net_cash_change),
                style: 'total',
                alignment: right,
              },
            ],
            [
              { text: i18n.t.endingCash, style: 'total' },
              { text: fmt(cashFlow.ending_cash), style: 'total', alignment: right },
            ],
          ],
        },
        layout: 'lightHorizontalLines',
        margin: [0, 0, 0, 12] as Margins,
      },
      {
        table: {
          headerRows: 1,
          widths: [65, 75, '*', 90, 85],
          body: [
            [
              { text: i18n.t.colDate, style: 'tableHeader' },
              { text: i18n.t.category, style: 'tableHeader' },
              { text: i18n.t.description, style: 'tableHeader' },
              { text: i18n.t.contraAccount, style: 'tableHeader' },
              { text: i18n.t.netChange, style: 'tableHeader', alignment: right },
            ],
            ...allRows,
          ],
        },
        layout: 'lightHorizontalLines',
      },
    ];
  }

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
        canvas: [
          {
            type: 'line',
            x1: 0,
            y1: 4,
            x2: 515,
            y2: 4,
            lineWidth: 0.5,
            lineColor: PDF_INK.lineSoft,
          },
        ],
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
      canvas: [
        { type: 'line', x1: 0, y1: 4, x2: 515, y2: 4, lineWidth: 0.5, lineColor: PDF_INK.lineSoft },
      ],
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
          canvas: [
            {
              type: 'line',
              x1: 0,
              y1: 0,
              x2: 742,
              y2: 0,
              lineWidth: 1,
              lineColor: PDF_INK.lineStrong,
            },
          ],
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
        brandTitle: { fontSize: 14, bold: true, color: PDF_INK.textStrong },
        reportTitle: { fontSize: 11, bold: true, color: PDF_INK.textStrong },
        section: { fontSize: 10, bold: true, color: PDF_INK.textStrong },
        total: { fontSize: 10, bold: true, color: PDF_INK.textStrong },
        bodyRow: { fontSize: 9, color: PDF_INK.textBody },
        tableHeader: { bold: true, fontSize: 8, fillColor: PDF_INK.headerFill, margin: margin4 },
      },
    };

    const slug = opts.reportTitle.replace(/\s+/g, '_').toLowerCase();
    await savePDF(docDefinition, `${APP_SLUG}_${slug}_${opts.period || 'alltime'}.pdf`);
  } catch (err) {
    console.error('Export Report PDF error:', err);
  }
}

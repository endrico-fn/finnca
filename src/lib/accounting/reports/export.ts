import { save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import * as pdfMake from 'pdfmake/build/pdfmake';
import Papa from 'papaparse';
import type { TDocumentDefinitions } from 'pdfmake/interfaces';
import { pdfMakeVfs } from '../../vfs_fonts';
import { APP_NAME } from '../../types';
import type { Account, VaultData } from '../types';
import { fromMinor, convertMinor, accountBalanceMinorFiltered, accountBalanceMinor } from '../finance';
import { incomeStatement, balanceSheet, trialBalance, totalDebitsCredits } from './statements';

(pdfMake as any).vfs = pdfMakeVfs;
(pdfMake as any).fonts = {
  ProtoMono: {
    normal: 'ProtoMono.ttf',
    bold: 'ProtoMono.ttf',
    italics: 'ProtoMono.ttf',
    bolditalics: 'ProtoMono.ttf',
  },
};

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
  tab: 'pnl' | 'bs' | 'tb' | 'trends' | 'debt' | 'cashflow' | 'fx';
  vault: VaultData;
  from: string;
  to: string;
  pnl: { income: number; expense: number; net: number };
  bs: { assets: number; liabilities: number; equity: number; netIncome: number; balanced: boolean };
  tb: { account: Account; debit: number; credit: number }[];
  totals: { debit: number; credit: number };
  incomeAccounts: Account[];
  expenseAccounts: Account[];
  assetAccounts: Account[];
  liabilityAccounts: Account[];
  equityAccounts: Account[];
  childrenMap: Map<string, string[]>;
  fxRate: number;
}

async function savePDF(docDefinition: TDocumentDefinitions, defaultPath: string) {
  const filePath = await save({
    filters: [{ name: 'PDF Document', extensions: ['pdf'] }],
    defaultPath,
  });
  if (!filePath) return;

  const pdfDocGenerator = pdfMake.createPdf(docDefinition);
  const base64 = await new Promise<string>((resolve) => {
    // @ts-ignore - The types say 0 args, but the browser implementation requires a callback
    pdfDocGenerator.getBase64((data: string) => resolve(data));
  });

  const binaryString = window.atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  await invoke('write_file_raw', { path: filePath, contents: Array.from(bytes) });
}

function buildReportPDFBody(opts: ReportPDFOptions): any[] {
  const fmt = (minor: number) => fromMinor('IDR', minor).toLocaleString('id-ID');
  const { tab, vault, from, to, fxRate } = opts;

  if (tab === 'pnl') {
    const pnl = incomeStatement(vault, from || undefined, to || undefined);
    const incomeRows = opts.incomeAccounts.map((a) => {
      const bal = accountBalanceMinorFiltered(a.id, vault, from || undefined, to || undefined, opts.childrenMap);
      return [
        { text: `  ${a.code}  ${a.name}`, style: 'bodyRow' },
        { text: fmt(convertMinor(Math.abs(bal), a.currency, 'IDR', fxRate)), style: 'bodyRow', alignment: 'right' },
      ];
    });
    const expenseRows = opts.expenseAccounts.map((a) => {
      const bal = accountBalanceMinorFiltered(a.id, vault, from || undefined, to || undefined, opts.childrenMap);
      return [
        { text: `  ${a.code}  ${a.name}`, style: 'bodyRow' },
        { text: fmt(convertMinor(Math.abs(bal), a.currency, 'IDR', fxRate)), style: 'bodyRow', alignment: 'right' },
      ];
    });
    const margin4: [number, number, number, number] = [0, 4, 0, 4];
    return [
      { text: 'REVENUES', style: 'section', margin: [0, 0, 0, 4] as any },
      {
        table: { widths: ['*', 120], body: incomeRows.length ? incomeRows : [[{ text: '  —', style: 'bodyRow' }, { text: '0', style: 'bodyRow', alignment: 'right' }]] },
        layout: 'noBorders',
      },
      { text: ' ', margin: margin4 as any },
      { text: 'EXPENSES', style: 'section', margin: [0, 4, 0, 4] as any },
      {
        table: { widths: ['*', 120], body: expenseRows.length ? expenseRows : [[{ text: '  —', style: 'bodyRow' }, { text: '0', style: 'bodyRow', alignment: 'right' }]] },
        layout: 'noBorders',
      },
      { canvas: [{ type: 'line', x1: 0, y1: 4, x2: 515, y2: 4, lineWidth: 0.5, lineColor: '#333' }] },
      {
        columns: [
          { text: 'NET INCOME', style: 'total' },
          { text: fmt(pnl.net), style: 'total', alignment: 'right' },
        ],
        margin: [0, 8, 0, 4] as any,
      },
      {
        columns: [
          { text: 'PROFIT MARGIN', style: 'bodyRow' },
          { text: pnl.income > 0 ? `${((pnl.net / pnl.income) * 100).toFixed(1)}%` : '—', style: 'bodyRow', alignment: 'right' },
        ],
      },
    ];
  }

  if (tab === 'bs') {
    const bs = balanceSheet(vault, to || undefined, opts.childrenMap);
    const sections = [
      { label: 'ASSETS', accounts: opts.assetAccounts },
      { label: 'LIABILITIES', accounts: opts.liabilityAccounts },
      { label: 'EQUITY', accounts: opts.equityAccounts },
    ];
    const rows: object[] = [];
    for (const sec of sections) {
      rows.push({ text: sec.label, style: 'section', margin: [0, 6, 0, 4] as any });
      const accountRows = sec.accounts.map((a) => {
        const bal = accountBalanceMinor(a.id, vault, opts.childrenMap);
        return [
          { text: `  ${a.code}  ${a.name}`, style: 'bodyRow' },
          { text: fmt(convertMinor(Math.abs(bal), a.currency, 'IDR', fxRate)), style: 'bodyRow', alignment: 'right' },
        ];
      });
      if (accountRows.length) {
        rows.push({ table: { widths: ['*', 120], body: accountRows }, layout: 'noBorders' });
      }
    }
    rows.push({ canvas: [{ type: 'line', x1: 0, y1: 4, x2: 515, y2: 4, lineWidth: 0.5, lineColor: '#333' }] });
    rows.push({
      columns: [
        { text: bs.balanced ? 'A = L + E  ✓ BALANCED' : 'A = L + E  ✗ UNBALANCED', style: 'total' },
        { text: fmt(bs.assets), style: 'total', alignment: 'right' },
      ],
      margin: [0, 8, 0, 0] as any,
    });
    return rows;
  }

  if (tab === 'tb') {
    const tb = trialBalance(vault, opts.childrenMap);
    const totals = totalDebitsCredits(tb);
    const tbRows = tb.map((r) => [
      { text: r.account.code, style: 'bodyRow' },
      { text: r.account.name, style: 'bodyRow' },
      { text: r.debit > 0 ? fmt(r.debit) : '', style: 'bodyRow', alignment: 'right' },
      { text: r.credit > 0 ? fmt(r.credit) : '', style: 'bodyRow', alignment: 'right' },
    ]);
    tbRows.push([
      { text: '', style: 'bodyRow' },
      { text: 'TOTAL', style: 'total' },
      { text: fmt(totals.debit), style: 'total', alignment: 'right' },
      { text: fmt(totals.credit), style: 'total', alignment: 'right' },
    ]);
    return [
      {
        table: {
          headerRows: 1,
          widths: [40, '*', 100, 100],
          body: [
            [
              { text: 'CODE', style: 'tableHeader' },
              { text: 'ACCOUNT', style: 'tableHeader' },
              { text: 'DEBIT', style: 'tableHeader', alignment: 'right' },
              { text: 'CREDIT', style: 'tableHeader', alignment: 'right' },
            ],
            ...tbRows,
          ],
        },
        layout: 'lightHorizontalLines',
      },
    ];
  }

  return [{ text: `Report type "${tab}" does not support PDF export.`, style: 'bodyRow' }];
}

export async function exportReportPDF(opts: ReportPDFOptions) {
  try {
    const margin4: [number, number, number, number] = [0, 4, 0, 4];
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
                { text: 'EXPORT REPORT', style: 'brandTitle' },
              ],
              width: 'auto',
            },
            { text: '', width: '*' },
            {
              stack: [
                { text: opts.vaultName.toUpperCase(), style: 'brandTitle', alignment: 'right' },
              ],
              width: 'auto',
            },
          ],
          margin: [0, 0, 0, 16] as any,
        },
        { canvas: [{ type: 'line', x1: 0, y1: 0, x2: 742, y2: 0, lineWidth: 1, lineColor: '#222' }] },
        {
          columns: [
            { text: opts.reportTitle.toUpperCase(), style: 'reportTitle' },
            { text: `PERIOD: ${opts.period || 'ALL TIME'}`, style: 'reportTitle', alignment: 'right' },
          ],
          margin: [0, 10, 0, 20] as any,
        },
        ...buildReportPDFBody(opts),
      ],
      styles: {
        brandTitle: { fontSize: 14, bold: true, color: '#111' },
        reportTitle: { fontSize: 11, bold: true, color: '#111' },
        section: { fontSize: 10, bold: true, color: '#111' },
        total: { fontSize: 10, bold: true, color: '#111' },
        bodyRow: { fontSize: 9, color: '#333' },
        tableHeader: { bold: true, fontSize: 8, fillColor: '#eeeeee', margin: margin4 as any },
      },
    };

    const slug = opts.reportTitle.replace(/\s+/g, '_').toLowerCase();
    await savePDF(docDefinition, `finnca_${slug}_${opts.period || 'alltime'}.pdf`);
  } catch (err) {
    console.error('Export Report PDF error:', err);
  }
}

export async function exportAccountStatementCSV(
  account: Account,
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
      Date: row.date,
      Description: row.description,
      Debit: row.debit !== 0 ? fromMinor(account.currency, row.debit) : '',
      Credit: row.credit !== 0 ? fromMinor(account.currency, row.credit) : '',
      Balance: fromMinor(account.currency, row.balance),
    }));

    const csvStr = Papa.unparse(csvData);
    const bytes = new TextEncoder().encode(csvStr);
    await invoke('write_file_raw', { path: filePath, contents: Array.from(bytes) });
  } catch (err) {
    console.error('Export CSV error:', err);
  }
}

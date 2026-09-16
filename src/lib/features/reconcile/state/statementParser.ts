import Papa from 'papaparse';
import { toMinor } from '$lib/core/format/currency';
import type { StatementRow } from './reconcile.svelte';

export function parseCsvStatement(
  fileContent: string,
  currency: string,
  isDebit: boolean
): StatementRow[] {
  const parsed = Papa.parse<Record<string, string>>(fileContent, {
    header: true,
    skipEmptyLines: true,
  });

  const statements: StatementRow[] = [];
  for (const row of parsed.data) {
    const keys = Object.keys(row);
    const dateKey =
      keys.find(
        (k) =>
          k.toLowerCase().includes('date') ||
          k.toLowerCase().includes('tgl') ||
          k.toLowerCase().includes('tanggal')
      ) || keys[0];
    const amountKey =
      keys.find(
        (k) =>
          k.toLowerCase().includes('amount') ||
          k.toLowerCase().includes('mutasi') ||
          k.toLowerCase().includes('nominal')
      ) || keys[1];
    const descKey =
      keys.find(
        (k) =>
          k.toLowerCase().includes('desc') ||
          k.toLowerCase().includes('keterangan') ||
          k.toLowerCase().includes('uraian')
      ) || keys[2];

    if (dateKey && amountKey && row[dateKey] && row[amountKey]) {
      const dateStr = row[dateKey].trim();
      const cleanAmt = row[amountKey].replace(/[^0-9.-]/g, '');
      const val = parseFloat(cleanAmt);
      if (!isNaN(val)) {
        const minor = toMinor(currency, val);
        statements.push({
          date: dateStr,
          amount: isDebit ? minor : -minor,
          description: descKey ? row[descKey]?.trim() : undefined,
        });
      }
    }
  }

  return statements;
}

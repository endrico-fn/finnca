import Papa from 'papaparse';
import { parseStringAmountToMinor } from '$lib/core/format/currency';
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

    const debitKey = keys.find((k) => /debit|debet|keluar|pengeluaran|withdrawal/i.test(k.trim()));
    const creditKey = keys.find((k) => /credit|kredit|masuk|penerimaan|deposit/i.test(k.trim()));
    const typeKey = keys.find((k) => /type|tipe|cr\/db|d\/k/i.test(k.trim()));
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

    if (!dateKey || !row[dateKey]) continue;
    const dateStr = row[dateKey].trim();
    if (!dateStr) continue;

    if (debitKey && creditKey) {
      const debitRaw = row[debitKey] || '';
      const creditRaw = row[creditKey] || '';
      const debitMinor = Math.abs(parseStringAmountToMinor(debitRaw, currency));
      const creditMinor = Math.abs(parseStringAmountToMinor(creditRaw, currency));

      if (debitMinor > 0) {
        statements.push({
          date: dateStr,
          amount: isDebit ? -debitMinor : debitMinor,
          description: descKey ? row[descKey]?.trim() : undefined,
        });
      } else if (creditMinor > 0) {
        statements.push({
          date: dateStr,
          amount: isDebit ? creditMinor : -creditMinor,
          description: descKey ? row[descKey]?.trim() : undefined,
        });
      }
      continue;
    }

    if (amountKey && row[amountKey]) {
      const rawText = row[amountKey].trim();
      const parsedMinor = Math.abs(parseStringAmountToMinor(rawText, currency));
      if (parsedMinor > 0) {
        const isParens = rawText.startsWith('(') && rawText.endsWith(')');
        const typeVal = typeKey ? (row[typeKey] || '').trim().toUpperCase() : '';
        const isDbIndicator =
          rawText.toUpperCase().includes('DB') ||
          typeVal === 'DB' ||
          typeVal === 'D' ||
          isParens ||
          rawText.startsWith('-');
        const isCrIndicator =
          rawText.toUpperCase().includes('CR') ||
          typeVal === 'CR' ||
          typeVal === 'C' ||
          typeVal === 'K';

        let finalSignedAmount: number;
        if (isDbIndicator) {
          finalSignedAmount = isDebit ? -parsedMinor : parsedMinor;
        } else if (isCrIndicator) {
          finalSignedAmount = isDebit ? parsedMinor : -parsedMinor;
        } else {
          finalSignedAmount = isDebit
            ? rawText.includes('-')
              ? -parsedMinor
              : parsedMinor
            : rawText.includes('-')
              ? parsedMinor
              : -parsedMinor;
        }

        statements.push({
          date: dateStr,
          amount: finalSignedAmount,
          description: descKey ? row[descKey]?.trim() : undefined,
        });
      }
    }
  }

  return statements;
}

export type BankAdjustmentType = 'FEE' | 'INTEREST' | null;

export interface BankFeeHeuristic {
  isFeeOrInterest: boolean;
  type: BankAdjustmentType;
  suggestedAccountKeyword: string;
}

export function detectBankAdjustment(description?: string, amount?: number): BankFeeHeuristic {
  if (!description) {
    return { isFeeOrInterest: false, type: null, suggestedAccountKeyword: '' };
  }
  const desc = description.toLowerCase();

  const isFeePattern =
    /\b(adm|admin|administrasi|biaya\s*adm|biaya\s*admin|biaya\s*kartu|biaya\s*bulanan|biaya\s*rekening|fee|monthly\s*fee|bank\s*charge|pajak|pajak\s*bunga|tax|pph)\b/i.test(
      desc
    );
  const isInterestPattern = /\b(bunga|jasa\s*giro|interest)\b/i.test(desc);

  if (amount !== undefined && amount > 0 && isInterestPattern) {
    return {
      isFeeOrInterest: true,
      type: 'INTEREST',
      suggestedAccountKeyword: 'bunga',
    };
  }

  if (amount !== undefined && amount < 0 && isFeePattern) {
    return {
      isFeeOrInterest: true,
      type: 'FEE',
      suggestedAccountKeyword: 'adm',
    };
  }

  if (isFeePattern) {
    return {
      isFeeOrInterest: true,
      type: 'FEE',
      suggestedAccountKeyword: 'adm',
    };
  }

  if (isInterestPattern) {
    return {
      isFeeOrInterest: true,
      type: 'INTEREST',
      suggestedAccountKeyword: 'bunga',
    };
  }

  return { isFeeOrInterest: false, type: null, suggestedAccountKeyword: '' };
}

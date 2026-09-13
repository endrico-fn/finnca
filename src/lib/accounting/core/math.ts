import { DEFAULT_FX_RATE } from '../types';
import type { Currency } from '../types';

export function uid(): string {
  try {
    return crypto.randomUUID();
  } catch {
    try {
      const bytes = new Uint8Array(16);
      crypto.getRandomValues(bytes);
      bytes[6] = (bytes[6] & 0x0f) | 0x40;
      bytes[8] = (bytes[8] & 0x3f) | 0x80;
      return [...bytes]
        .map((b, i) => {
          const hex = b.toString(16).padStart(2, '0');
          return [4, 6, 8, 10].includes(i) ? `-${hex}` : hex;
        })
        .join('');
    } catch {
      return Math.random().toString(36).slice(2, 9) + Date.now().toString(36);
    }
  }
}

export function roundHalfToEven(num: number, decimals: number = 0): number {
  const factor = 10 ** decimals;
  const n = +(decimals ? num * factor : num).toFixed(8);
  const i = Math.floor(n);
  const f = n - i;
  const e = 1e-8;
  const r = f > 0.5 - e && f < 0.5 + e ? (i % 2 === 0 ? i : i + 1) : Math.round(n);
  return decimals ? r / factor : r;
}

export function toMinor(currency: Currency, major: number): number {
  if (currency === 'USD') return roundHalfToEven(major * 100);
  if (currency === 'IDR') return roundHalfToEven(major);
  throw new Error(`Unsupported currency: ${currency}`);
}

export function parseStringAmountToMinor(str: string, currency: Currency): number {
  if (!str) return 0;
  let clean = str.trim();
  const isNegative = clean.startsWith('-');
  clean = clean.replace(/^-/, '');

  if (currency === 'IDR') {
    const digitsOnly = clean.replace(/[^0-9]/g, '');
    if (!digitsOnly) return 0;
    const val = parseInt(digitsOnly, 10);
    return isNegative ? -val : val;
  }

  if (currency === 'USD') {
    if (clean.includes(',') && clean.includes('.')) {
      clean = clean.replace(/,/g, '');
    } else if (clean.includes(',') && !clean.includes('.')) {
      clean = clean.replace(',', '.');
    }
    const cleanNum = clean.replace(/[^0-9.]/g, '');
    if (!cleanNum) return 0;
    const parts = cleanNum.split('.');
    const majorStr = parts[0] || '0';
    const minorStr = (parts[1] || '').padEnd(2, '0').slice(0, 2);
    const major = parseInt(majorStr, 10);
    const minor = parseInt(minorStr, 10);
    const total = major * 100 + minor;
    return isNegative ? -total : total;
  }

  throw new Error(`Unsupported currency: ${currency}`);
}

export function fromMinor(currency: Currency, minor: number): number {
  if (currency === 'USD') return minor / 100;
  if (currency === 'IDR') return minor;
  throw new Error(`Unsupported currency: ${currency}`);
}

const DC_DEBIT_TOKENS = ['db', 'd', 'debit', 'debet'];
const DC_CREDIT_TOKENS = ['cr', 'c', 'credit', 'kredit'];

export function classifyDcType(raw: string): 'db' | 'cr' | null {
  const token = raw.trim().toLowerCase();
  if (DC_DEBIT_TOKENS.includes(token)) return 'db';
  if (DC_CREDIT_TOKENS.includes(token)) return 'cr';
  return null;
}

export function parseBankAmountToMinor(str: string, currency: Currency): number {
  if (!str) return 0;
  let clean = str.trim();
  if (!clean) return 0;
  let negative = false;
  if (clean.startsWith('(') && clean.endsWith(')')) {
    negative = true;
    clean = clean.slice(1, -1).trim();
  }
  if (clean.startsWith('-')) {
    negative = true;
    clean = clean.slice(1).trim();
  } else if (clean.startsWith('+')) {
    clean = clean.slice(1).trim();
  }
  clean = clean.replace(/[^0-9.,]/g, '');
  if (!clean) return 0;
  const lastDot = clean.lastIndexOf('.');
  const lastComma = clean.lastIndexOf(',');
  let normalized: string;
  if (lastDot >= 0 && lastComma >= 0) {
    if (lastComma > lastDot) {
      normalized = clean.replace(/\./g, '').replace(',', '.');
    } else {
      normalized = clean.replace(/,/g, '');
    }
  } else if (lastComma >= 0) {
    const tail = clean.length - lastComma - 1;
    normalized = tail === 2 ? clean.replace(',', '.') : clean.replace(/,/g, '');
  } else if (lastDot >= 0) {
    const groups = clean.split('.');
    if (groups.length > 2) {
      normalized = clean.replace(/\./g, '');
    } else if (groups[1].length === 3 && currency === 'IDR') {
      normalized = clean.replace(/\./g, '');
    } else {
      normalized = clean;
    }
  } else {
    normalized = clean;
  }
  const value = parseFloat(normalized);
  if (isNaN(value)) return 0;
  const minor = toMinor(currency, value);
  return negative ? -minor : minor;
}

export function formatIDR(minor: number): string {
  const n = fromMinor('IDR', minor);
  return 'Rp ' + n.toLocaleString('en-US', { maximumFractionDigits: 0 });
}

export function formatUSD(minor: number): string {
  const n = fromMinor('USD', minor);
  return (
    '$' +
    n.toLocaleString('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })
  );
}

export function formatMoney(minor: number, currency: Currency): string {
  if (currency === 'USD') return formatUSD(minor);
  return formatIDR(minor);
}

export function convertMinor(
  minor: number,
  from: Currency,
  to: Currency,
  fxRate: number = DEFAULT_FX_RATE
): number {
  if (from === to) return minor;

  if (from === 'IDR' && to === 'USD') {
    if (!fxRate || fxRate <= 0) return 0;
    return roundHalfToEven((minor * 100) / fxRate);
  }

  if (from === 'USD' && to === 'IDR') {
    return roundHalfToEven((minor * fxRate) / 100);
  }

  throw new Error(`Unsupported currency conversion: ${from} to ${to}`);
}

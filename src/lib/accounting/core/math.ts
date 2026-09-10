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
      return [...bytes].map((b, i) => {
        const hex = b.toString(16).padStart(2, '0');
        return [4, 6, 8, 10].includes(i) ? `-${hex}` : hex;
      }).join('');
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
  const clean = str.replace(/[^0-9.]/g, '');
  if (!clean) return 0;
  const parts = clean.split('.');
  const majorStr = parts[0] || '0';
  const minorStr = (parts[1] || '').padEnd(2, '0').slice(0, 2);
  
  const major = parseInt(majorStr, 10);
  const minor = parseInt(minorStr, 10);
  
  if (currency === 'USD') return major * 100 + minor;
  if (currency === 'IDR') return major;
  throw new Error(`Unsupported currency: ${currency}`);
}

export function fromMinor(currency: Currency, minor: number): number {
  if (currency === 'USD') return minor / 100;
  if (currency === 'IDR') return minor;
  throw new Error(`Unsupported currency: ${currency}`);
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

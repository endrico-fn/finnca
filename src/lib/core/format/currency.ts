import { getPref } from '$lib/core/state/prefs';
import { evaluateFinancialExpression, hasMathExpression } from './mathExpression';

export type SupportedCurrency = 'IDR' | 'USD' | string;

export interface FormatCurrencyOptions {
  locale?: string;
  showSign?: boolean;
  compact?: boolean;
}

function getFormatters() {
  const isDot = getPref('finnca_num_format', 'comma') === 'dot';
  return {
    idr: new Intl.NumberFormat(isDot ? 'id-ID' : 'en-US', {
      style: 'currency',
      currency: 'IDR',
      maximumFractionDigits: 0,
    }),
    usd: new Intl.NumberFormat(isDot ? 'id-ID' : 'en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }),
    isDot,
  };
}

/**
 * Format integer minor units (e.g. 1000000 IDR or 1500 USD cents = $15.00)
 * to a standardized localized display string.
 */
export function formatMinorToDisplay(
  minorUnits: number | bigint,
  currency: SupportedCurrency = 'IDR',
  options: FormatCurrencyOptions = {}
): string {
  const numeric = typeof minorUnits === 'bigint' ? Number(minorUnits) : minorUnits;
  const isNegative = numeric < 0;
  const absUnits = Math.abs(numeric);

  let formatted: string;
  const currUpper = currency.toUpperCase();
  const formatters = getFormatters();

  if (currUpper === 'USD') {
    const major = absUnits / 100;
    formatted = formatters.usd.format(major);
  } else if (currUpper === 'IDR') {
    formatted = formatters.idr.format(absUnits);
  } else {
    try {
      const fallbackLocale = options.locale ?? (formatters.isDot ? 'id-ID' : 'en-US');
      const formatter = new Intl.NumberFormat(fallbackLocale, {
        style: 'currency',
        currency: currUpper,
      });
      formatted = formatter.format(absUnits / 100);
    } catch {
      const targetLocale = options.locale ?? (formatters.isDot ? 'id-ID' : 'en-US');
      const majorFallback = absUnits / 100;
      formatted = `${currUpper} ${majorFallback.toLocaleString(targetLocale, {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      })}`;
    }
  }

  if (isNegative) {
    return `-${formatted}`;
  }
  if (options.showSign && numeric > 0) {
    return `+${formatted}`;
  }
  return formatted;
}

export function formatIDR(minorUnits: number | bigint): string {
  return formatMinorToDisplay(minorUnits, 'IDR');
}

export function formatUSD(minorUnits: number | bigint): string {
  return formatMinorToDisplay(minorUnits, 'USD');
}

export function getCurrencyPrefix(currency: SupportedCurrency = 'IDR'): string {
  const upper = (currency || 'IDR').toUpperCase();
  switch (upper) {
    case 'IDR':
      return 'Rp';
    case 'USD':
      return '$';
    case 'EUR':
      return '€';
    case 'GBP':
      return '£';
    case 'JPY':
    case 'CNY':
      return '¥';
    case 'SGD':
      return 'S$';
    case 'AUD':
      return 'A$';
    default:
      return upper;
  }
}

export function getCurrencyFactor(currency: SupportedCurrency = 'IDR'): number {
  const c = (currency || 'IDR').trim().toUpperCase();
  switch (c) {
    case 'IDR':
    case 'JPY':
    case 'KRW':
    case 'VND':
    case 'CLP':
    case 'HUF':
    case 'PYG':
    case 'RWF':
    case 'UGX':
    case 'BIF':
    case 'DJF':
    case 'GNF':
    case 'KMF':
    case 'XAF':
    case 'XOF':
    case 'XPF':
      return 1;
    default:
      return 100;
  }
}

export function formatMinorGrouping(
  minorUnits: number,
  currency: SupportedCurrency = 'IDR',
  locale?: string
): string {
  const isDot = getPref('finnca_num_format', 'comma') === 'dot';
  const targetLocale = locale ?? (isDot ? 'id-ID' : 'en-US');
  const major = fromMinor(currency, minorUnits);
  const factor = getCurrencyFactor(currency);
  return major.toLocaleString(targetLocale, {
    minimumFractionDigits: factor === 1 ? 0 : 2,
    maximumFractionDigits: factor === 1 ? 0 : 2,
  });
}

export function fromMinor(currency: SupportedCurrency, minorUnits: number): number {
  const factor = getCurrencyFactor(currency);
  return factor === 1 ? minorUnits : minorUnits / factor;
}

export function toMinor(currency: SupportedCurrency, majorUnits: number): number {
  const factor = getCurrencyFactor(currency);
  return Math.round(majorUnits * factor);
}

export function parseStringAmountToMinor(str: string, currency: SupportedCurrency = 'IDR'): number {
  if (!str) return 0;
  let clean = str.trim();

  if (hasMathExpression(clean)) {
    const evaluated = evaluateFinancialExpression(clean, currency);
    if (evaluated !== null) {
      return toMinor(currency, evaluated);
    }
  }

  const isNegative = clean.startsWith('-');
  clean = clean.replace(/^-/, '');

  const factor = getCurrencyFactor(currency);
  if (factor === 1) {
    const digitsOnly = clean.replace(/[^0-9]/g, '');
    if (!digitsOnly) return 0;
    const val = parseInt(digitsOnly, 10);
    return isNegative ? -val : val;
  }

  // 2-decimal currencies (USD, EUR, GBP, etc.)
  const lastComma = clean.lastIndexOf(',');
  const lastDot = clean.lastIndexOf('.');
  if (lastComma !== -1 && lastDot !== -1) {
    if (lastComma > lastDot) {
      // European / Indonesian: 1.500,50 -> remove dots, replace comma with dot
      clean = clean.replace(/\./g, '').replace(',', '.');
    } else {
      // US / UK: 1,500.50 -> remove commas
      clean = clean.replace(/,/g, '');
    }
  } else if (clean.includes(',') && !clean.includes('.')) {
    if (/^\d{1,3}(,\d{3})+$/.test(clean)) {
      clean = clean.replace(/,/g, '');
    } else {
      clean = clean.replace(',', '.');
    }
  } else if (clean.includes('.') && !clean.includes(',')) {
    // 1.000.000, 10.000 -> thousand separators; 12.50 / 1.5 stay decimal
    if (/^\d{1,3}(\.\d{3})+$/.test(clean) && !/^\d+\.\d{1,2}$/.test(clean)) {
      clean = clean.replace(/\./g, '');
    }
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

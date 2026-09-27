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
    usd: new Intl.NumberFormat(isDot ? 'de-DE' : 'en-US', {
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
      formatted = `${currUpper} ${absUnits.toLocaleString()}`;
    }
  }

  if (isNegative) {
    return options.showSign ? `-${formatted}` : `-${formatted}`;
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

export function formatMinorGrouping(
  minorUnits: number,
  currency: SupportedCurrency = 'IDR',
  locale?: string
): string {
  const isDot = getPref('finnca_num_format', 'comma') === 'dot';
  const targetLocale = locale ?? (isDot ? 'id-ID' : 'en-US');
  const major = fromMinor(currency, minorUnits);
  return major.toLocaleString(targetLocale, {
    maximumFractionDigits: currency.toUpperCase() === 'USD' ? 2 : 0,
  });
}

export function fromMinor(currency: SupportedCurrency, minorUnits: number): number {
  if (currency.toUpperCase() === 'USD') {
    return minorUnits / 100;
  }
  return minorUnits;
}

export function toMinor(currency: SupportedCurrency, majorUnits: number): number {
  if (currency.toUpperCase() === 'USD') {
    return Math.round(majorUnits * 100);
  }
  return Math.round(majorUnits);
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

  const currUpper = currency.toUpperCase();
  if (currUpper === 'IDR') {
    const digitsOnly = clean.replace(/[^0-9]/g, '');
    if (!digitsOnly) return 0;
    const val = parseInt(digitsOnly, 10);
    return isNegative ? -val : val;
  }

  if (currUpper === 'USD') {
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

  const digitsOnly = clean.replace(/[^0-9]/g, '');
  if (!digitsOnly) return 0;
  const val = parseInt(digitsOnly, 10);
  return isNegative ? -val : val;
}

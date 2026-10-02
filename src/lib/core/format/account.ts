import type { Account, AccountType } from '$lib/core/ipc/bindings';
import { i18n } from '$lib/core/i18n.svelte';

export const ACCOUNT_TYPES: AccountType[] = ['ASSET', 'LIABILITY', 'EQUITY', 'INCOME', 'EXPENSE'];

export const ACCOUNT_TYPE_COLOR: Record<AccountType, string> = {
  ASSET: 'var(--color-asset)',
  LIABILITY: 'var(--color-liability)',
  EQUITY: 'var(--color-equity)',
  INCOME: 'var(--color-income)',
  EXPENSE: 'var(--color-expense)',
};

export const THEME_SWATCH_TOKENS = {
  ASSET: 'var(--color-asset)',
  LIABILITY: 'var(--color-liability)',
  EQUITY: 'var(--color-equity)',
  INCOME: 'var(--color-income)',
  EXPENSE: 'var(--color-expense)',
  TEAL: 'var(--color-teal)',
  LIGHT: 'var(--color-text-white)',
  DARK: 'var(--color-bg-app)',
} as const;

export const THEME_SWATCH_HEX: Record<string, string> = {
  [THEME_SWATCH_TOKENS.TEAL]: '#2dd4bf',
  [THEME_SWATCH_TOKENS.LIGHT]: '#ffffff',
  [THEME_SWATCH_TOKENS.DARK]: '#0c0d0e',
  [THEME_SWATCH_TOKENS.ASSET]: '#fbbf24',
  [THEME_SWATCH_TOKENS.LIABILITY]: '#38bdf8',
  [THEME_SWATCH_TOKENS.EQUITY]: '#c084fc',
  [THEME_SWATCH_TOKENS.INCOME]: '#34d399',
  [THEME_SWATCH_TOKENS.EXPENSE]: '#f43f5e',
};

export const DEFAULT_SWATCH_TOKEN = THEME_SWATCH_TOKENS.TEAL;

export const ACCOUNT_TYPE_BG: Record<AccountType, string> = {
  ASSET: 'bg-asset',
  LIABILITY: 'bg-liability',
  EQUITY: 'bg-equity',
  INCOME: 'bg-income',
  EXPENSE: 'bg-expense',
};

export const ACCOUNT_TYPE_TEXT: Record<AccountType, string> = {
  ASSET: 'text-asset',
  LIABILITY: 'text-liability',
  EQUITY: 'text-equity',
  INCOME: 'text-income',
  EXPENSE: 'text-expense',
};

export function accountTypeLabel(type: AccountType): string {
  switch (type) {
    case 'ASSET':
      return i18n.t.accTypeAsset;
    case 'LIABILITY':
      return i18n.t.accTypeLiability;
    case 'EQUITY':
      return i18n.t.accTypeEquity;
    case 'INCOME':
      return i18n.t.accTypeIncome;
    case 'EXPENSE':
      return i18n.t.accTypeExpense;
    default:
      return type;
  }
}

const TYPE_STEMS: Record<AccountType, string[]> = {
  ASSET: ['asset', 'assets', 'aset'],
  LIABILITY: ['liabilit', 'liabilitas', 'kewajiban', 'hutang', 'utang'],
  EQUITY: ['equit', 'ekuitas', 'modal'],
  INCOME: ['incom', 'revenue', 'pendapatan', 'penghasilan'],
  EXPENSE: ['expens', 'beban', 'biaya', 'pengeluaran'],
};

function isRootTypeEquivalent(name: string, type: AccountType): boolean {
  const clean = name.trim().toLowerCase();
  const rootLabel = accountTypeLabel(type).trim().toLowerCase();
  const typeKey = type.toLowerCase();
  if (clean === rootLabel || clean === typeKey) return true;
  const allowed = TYPE_STEMS[type] ?? [];
  return allowed.some((stem) => clean.startsWith(stem));
}

export function getAccountBreadcrumb(acc: Account, byId: Map<string, Account>): string[] {
  const parts: string[] = [acc.name];
  let cur: Account | undefined = acc;
  while (cur?.parent_id && byId.has(cur.parent_id)) {
    const parent: Account = byId.get(cur.parent_id)!;
    parts.unshift(parent.name);
    cur = parent;
  }
  const rootLabel = accountTypeLabel(acc.account_type);
  if (parts.length > 0 && !isRootTypeEquivalent(parts[0], acc.account_type)) {
    parts.unshift(rootLabel);
  }
  return parts;
}

export function getAccountPath(acc: Account, byId: Map<string, Account>): string {
  return getAccountBreadcrumb(acc, byId).join(' > ');
}

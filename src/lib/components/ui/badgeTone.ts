import type { AccountType } from '$lib/core/ipc/bindings';

export type BadgeTone =
  | 'ok'
  | 'warn'
  | 'err'
  | 'neutral'
  | 'teal'
  | 'asset'
  | 'liability'
  | 'equity'
  | 'income'
  | 'expense';

export type BadgeSize = 's' | 'm' | 'l';

export const BADGE_TONES: Record<BadgeTone, string> = {
  ok: 'badge-ok',
  warn: 'badge-warn',
  err: 'badge-err',
  neutral: 'border-line/80 bg-bg-card/50 text-text-dim',
  teal: 'text-teal border-teal/40 bg-teal/10',
  asset: 'text-asset border-asset/40 bg-asset/10',
  liability: 'text-liability border-liability/40 bg-liability/10',
  equity: 'text-equity border-equity/40 bg-equity/10',
  income: 'text-income border-income/40 bg-income/10',
  expense: 'text-expense border-expense/40 bg-expense/10',
};

export const BADGE_SIZES: Record<BadgeSize, string> = {
  s: 'px-1 py-px text-smaller leading-none font-bold tracking-wider',
  m: 'px-1.5 py-0.5 text-smaller leading-none font-bold tracking-wider',
  l: 'px-2 py-1 text-small leading-none font-bold tracking-wider',
};

export const ACCOUNT_TONE: Record<AccountType, BadgeTone> = {
  ASSET: 'asset',
  LIABILITY: 'liability',
  EQUITY: 'equity',
  INCOME: 'income',
  EXPENSE: 'expense',
};

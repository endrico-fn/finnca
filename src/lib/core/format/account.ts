import type { AccountType } from '$lib/core/ipc/bindings';
import { i18n } from '$lib/core/i18n.svelte';

export const ACCOUNT_TYPES: AccountType[] = ['ASSET', 'LIABILITY', 'EQUITY', 'INCOME', 'EXPENSE'];

export const ACCOUNT_TYPE_COLOR: Record<AccountType, string> = {
  ASSET: 'var(--color-asset, #eab308)',
  LIABILITY: 'var(--color-liability, #3b82f6)',
  EQUITY: 'var(--color-equity, #a855f7)',
  INCOME: 'var(--color-income, #3eb16f)',
  EXPENSE: 'var(--color-expense, #d5305f)',
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

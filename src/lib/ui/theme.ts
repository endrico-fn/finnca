import type { NotificationType } from '$lib/notifications/store.svelte';

export { ACCOUNT_TYPE_COLOR } from '$lib/accounting/types';

export function getNotificationAccent(type: NotificationType): string {
  switch (type) {
    case 'DUE_DATE':
      return 'var(--color-expense)';
    case 'FX_ALERT':
      return 'var(--color-income)';
    case 'EXPENSE_SPIKE':
    case 'CASHFLOW_DEFICIT':
      return 'var(--color-warning)';
    case 'LEDGER_INTEGRITY':
      return 'var(--color-expense)';
    default:
      return 'var(--color-teal)';
  }
}

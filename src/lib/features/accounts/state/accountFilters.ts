import type { AccountType } from '$lib/core/ipc/bindings';
import type { AccountRowItem } from '../components/AccountTable.svelte';

export function filterAccountRows(
  accounts: AccountRowItem[],
  searchQuery: string,
  selectedType: AccountType | 'ALL',
  showHidden: boolean,
  placeholderOnly: boolean
): AccountRowItem[] {
  const q = searchQuery.trim().toLowerCase();
  return accounts
    .filter((acc) => {
      if (!showHidden && acc.hidden) return false;
      if (placeholderOnly && !acc.placeholder) return false;
      if (selectedType !== 'ALL' && acc.account_type !== selectedType) return false;
      if (q) {
        return (
          acc.code.toLowerCase().includes(q) ||
          acc.fullPath.toLowerCase().includes(q) ||
          (acc.description ?? '').toLowerCase().includes(q)
        );
      }
      return true;
    })
    .sort((a, b) => a.code.localeCompare(b.code, undefined, { numeric: true }));
}

export function computeAccountTypeCounts(
  accounts: AccountRowItem[],
  searchQuery: string,
  showHidden: boolean,
  placeholderOnly: boolean,
  selectedType: AccountType | 'ALL'
): { typeCounts: Record<AccountType, number> & { ALL: number }; phCount: number } {
  const q = searchQuery.trim().toLowerCase();
  const baseHiddenSearch = accounts.filter((acc) => {
    if (!showHidden && acc.hidden) return false;
    if (q) {
      return (
        acc.code.toLowerCase().includes(q) ||
        acc.fullPath.toLowerCase().includes(q) ||
        (acc.description ?? '').toLowerCase().includes(q)
      );
    }
    return true;
  });

  const baseForCounts = baseHiddenSearch.filter((acc) => !placeholderOnly || acc.placeholder);

  const typeCounts: Record<AccountType, number> & { ALL: number } = {
    ALL: baseForCounts.length,
    ASSET: 0,
    LIABILITY: 0,
    EQUITY: 0,
    INCOME: 0,
    EXPENSE: 0,
  };

  for (const acc of baseForCounts) {
    if (typeCounts[acc.account_type] !== undefined) {
      typeCounts[acc.account_type] += 1;
    }
  }

  const phCount = baseHiddenSearch.filter(
    (acc) => (selectedType === 'ALL' || acc.account_type === selectedType) && acc.placeholder
  ).length;

  return { typeCounts, phCount };
}

import type { Account, AccountType } from '$lib/core/ipc/bindings';
import { accountsState } from '$lib/features/accounts/state/accounts.svelte';

export function isDebitNormalType(t: string | undefined): boolean {
  return t === 'ASSET' || t === 'EXPENSE';
}

export function resolveOpeningEquity(
  accounts: Account[],
  currency: string,
  excludeId?: string
): Account | undefined {
  const pool = accounts.filter(
    (a) => !a.placeholder && a.id !== excludeId && a.currency === currency
  );
  return (
    pool.find((a) => a.code === '3110') ??
    pool.find((a) => a.code === '3010') ??
    pool.find(
      (a) =>
        a.account_type === 'EQUITY' &&
        (a.name.toLowerCase().includes('opening balance') ||
          a.name.toLowerCase().includes('modal awal') ||
          a.name.toLowerCase().includes('saldo awal'))
    ) ??
    [...pool]
      .filter((a) => a.account_type === 'EQUITY')
      .sort((a, b) => a.code.localeCompare(b.code))[0]
  );
}

function nextFreeEquityCode(taken: Set<string>, currency: string): string {
  const candidates = ['3110', '3010', '3111', '3112', '3113', `3110-${currency}`, `3010-${currency}`];
  for (const c of candidates) {
    if (!taken.has(c)) return c;
  }
  let n = 3114;
  while (taken.has(String(n))) n += 1;
  return String(n);
}

export async function ensureOpeningEquity(
  currency: string,
  excludeId: string | undefined,
  autoName: string
): Promise<Account> {
  const existing = resolveOpeningEquity(accountsState.accounts, currency, excludeId);
  if (existing) return existing;
  const parent =
    accountsState.accounts.find((a) => a.code === '3100' && a.placeholder) ??
    accountsState.accounts.find((a) => a.code === '3000' && a.placeholder) ??
    null;
  const taken = new Set(accountsState.accounts.map((a) => a.code));
  return await accountsState.create({
    code: nextFreeEquityCode(taken, currency),
    name: autoName,
    account_type: 'EQUITY' as AccountType,
    parent_id: parent ? parent.id : null,
    currency,
    placeholder: false,
    hidden: false,
    description: null,
    color: null,
  });
}

export function openingSplitAmounts(isDebitNorm: boolean, minor: number): [number, number] {
  return isDebitNorm ? [minor, -minor] : [-minor, minor];
}

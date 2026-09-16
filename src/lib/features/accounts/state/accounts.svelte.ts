import {
  listAccountsCmd,
  createAccountCmd,
  updateAccountCmd,
  deleteAccountCmd,
  seedRootAccountsCmd,
  seedStarterAccountsCmd,
  type Account,
  type AccountBalanceView,
  type CreateAccountInput,
  type UpdateAccountInput,
} from '$lib/core/ipc/bindings';
import { AppError } from '$lib/core/ipc/errors';
import { eventBus } from '$lib/core/events/eventBus.svelte';

export { ACCOUNT_TYPES, ACCOUNT_TYPE_COLOR, accountTypeLabel } from '$lib/core/format/account';

export function getAccountPath(acc: Account, byId: Map<string, Account>): string {
  const parts: string[] = [acc.name];
  let cur: Account | undefined = acc;
  while (cur?.parent_id && byId.has(cur.parent_id)) {
    const parent: Account = byId.get(cur.parent_id)!;
    parts.unshift(parent.name);
    cur = parent;
  }
  const rootType =
    acc.account_type === 'ASSET'
      ? 'Assets'
      : acc.account_type === 'LIABILITY'
        ? 'Liabilities'
        : acc.account_type === 'EQUITY'
          ? 'Equity'
          : acc.account_type === 'INCOME'
            ? 'Income'
            : 'Expenses';
  if (parts.length > 0 && parts[0].toLowerCase() !== rootType.toLowerCase()) {
    parts.unshift(rootType);
  }
  return parts.join(' > ');
}

export function getAccountCleanPath(accId: string, byId: Map<string, Account>): string {
  const acc = byId.get(accId);
  if (!acc) return '';
  return getAccountPath(acc, byId);
}


class AccountsStore {
  items = $state<AccountBalanceView[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);

  accounts = $derived(this.items.map((i) => i.account));
  accountsById = $derived(new Map(this.items.map((i) => [i.account.id, i.account])));
  accountsByCode = $derived(new Map(this.items.map((i) => [i.account.code, i.account])));
  balancesById = $derived(
    new Map(
      this.items.map((i) => [
        i.account.id,
        { direct: i.direct_balance, recursive: i.recursive_balance },
      ])
    )
  );


  async load(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.items = await listAccountsCmd();
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
      throw e;
    } finally {
      this.loading = false;
    }
  }

  async create(input: CreateAccountInput): Promise<Account> {
    this.loading = true;
    this.error = null;
    try {
      const acc = await createAccountCmd(input);
      await this.load();
      eventBus.emit('accounts:changed', undefined);
      return acc;
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
      throw e;
    } finally {
      this.loading = false;
    }
  }

  async update(id: string, input: UpdateAccountInput): Promise<Account> {
    this.loading = true;
    this.error = null;
    try {
      const acc = await updateAccountCmd(id, input);
      await this.load();
      eventBus.emit('accounts:changed', undefined);
      return acc;
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
      throw e;
    } finally {
      this.loading = false;
    }
  }

  async delete(id: string): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      await deleteAccountCmd(id);
      await this.load();
      eventBus.emit('accounts:changed', undefined);
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
      throw e;
    } finally {
      this.loading = false;
    }
  }

  async seedRoots(): Promise<Account[]> {
    this.loading = true;
    this.error = null;
    try {
      const seeded = await seedRootAccountsCmd();
      await this.load();
      eventBus.emit('accounts:changed', undefined);
      return seeded;
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
      throw e;
    } finally {
      this.loading = false;
    }
  }

  async seedStarterAccounts(language?: 'en' | 'id' | string): Promise<Account[]> {
    this.loading = true;
    this.error = null;
    try {
      const seeded = await seedStarterAccountsCmd(language);
      await this.load();
      eventBus.emit('accounts:changed', undefined);
      return seeded;
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
      throw e;
    } finally {
      this.loading = false;
    }
  }

  clear(): void {
    this.items = [];
    this.error = null;
    this.loading = false;
  }
}

export const accountsState = new AccountsStore();

eventBus.on('vault:unlocked', () => {
  accountsState.load().catch(() => {});
});
eventBus.on('vault:locked', () => {
  accountsState.clear();
});
eventBus.on('accounts:changed', () => {
  accountsState.load().catch(() => {});
});

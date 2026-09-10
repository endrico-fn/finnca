import type { VaultData, Account, Transaction } from '$lib/accounting/types';
import { loadVault, saveVault } from '$lib/accounting/vault-service';
import { validateTransaction, todayString } from '$lib/accounting/finance';
import { DEFAULT_FX_RATE } from '$lib/accounting/types';
import { buildChildrenMap } from '$lib/accounting/ledger/accounts';

class LedgerStore {
  data = $state<VaultData | null>(null);
  loading = $state(true);
  error = $state('');
  private loadPromise: Promise<void> | null = null;
  private saveQueue: Promise<void> = Promise.resolve();
  fxRate = $derived(this.data?.fxRate ?? DEFAULT_FX_RATE);

  accounts = $derived(this.data?.accounts ?? []);
  transactions = $derived(this.data?.transactions ?? []);
  accountsById = $derived(new Map(this.accounts.map((a) => [a.id, a])));
  childrenMap = $derived(buildChildrenMap(this.accounts));

  async load() {
    if (this.loadPromise) return this.loadPromise;
    this.loading = true;
    this.error = '';
    this.loadPromise = loadVault()
      .then((d) => {
        this.data = d;
      })
      .catch((e) => {
        this.error = String(e);
      })
      .finally(() => {
        this.loading = false;
        this.loadPromise = null;
      });
    return this.loadPromise;
  }

  onSaveCallback: ((snapshot: import('$lib/accounting/types').VaultData) => void) | null = null;

  async save() {
    if (!this.data) return;
    const snapshot = $state.snapshot(this.data);
    const task = this.saveQueue.then(() => saveVault(snapshot));
    this.saveQueue = task.catch(() => {});
    await task;
    if (this.onSaveCallback) {
      this.onSaveCallback(snapshot);
    }
  }

  clear() {
    this.data = null;
    this.error = '';
    this.loading = true;
    this.loadPromise = null;
  }

  async upsertAccount(acc: Account) {
    if (!this.data) return;
    const idx = this.data.accounts.findIndex((a) => a.id === acc.id);
    if (idx >= 0) this.data.accounts[idx] = acc;
    else this.data.accounts.push(acc);
    await this.save();
  }

  async deleteAccount(id: string) {
    if (!this.data) return;
    const hasChildren = this.data.accounts.some((a) => a.parentId === id);
    if (hasChildren) throw new Error('Cannot delete account with children');
    const used = this.data.transactions.some((t) => t.splits.some((s) => s.accountId === id));
    if (used) throw new Error('Cannot delete account used in transactions');
    this.data.accounts = this.data.accounts.filter((a) => a.id !== id);
    await this.save();
  }

  async upsertTransaction(tx: Transaction) {
    if (!this.data) return;
    const err = validateTransaction(tx, this.accountsById);
    if (err) throw new Error(err);
    const idx = this.data.transactions.findIndex((t) => t.id === tx.id);
    if (idx >= 0) this.data.transactions[idx] = tx;
    else this.data.transactions.push(tx);
    this.data.transactions = [...this.data.transactions].sort((a, b) => b.date.localeCompare(a.date));
    await this.save();
  }

  async deleteTransaction(id: string) {
    if (!this.data) return;
    this.data.transactions = this.data.transactions.filter((t) => t.id !== id);
    await this.save();
  }

  private upsertFxHistory(rate: number): boolean {
    if (!this.data) return false;
    const today = todayString();
    if (!this.data.fxHistory) this.data.fxHistory = [];
    const idx = this.data.fxHistory.findIndex((h) => h.date === today);
    if (idx >= 0) {
      if (this.data.fxHistory[idx].rate === rate) return false;
      this.data.fxHistory[idx].rate = rate;
    } else {
      this.data.fxHistory.push({ date: today, rate });
      if (this.data.fxHistory.length > 365) {
        this.data.fxHistory = this.data.fxHistory.slice(-365);
      }
    }
    return true;
  }

  async setFxRate(rate: number) {
    if (!this.data) return;
    if (rate <= 0) throw new Error('Invalid FX rate');
    this.data.fxRate = rate;
    this.upsertFxHistory(rate);
    await this.save();
  }

  async recordFxHistory(liveRate: number) {
    if (!this.data) return;
    if (liveRate <= 1000) return;
    if (!this.upsertFxHistory(liveRate)) return;
    await this.save();
  }

  plans = $derived(this.data?.plans ?? []);
  budgets = $derived(this.data?.budgets ?? []);

  async upsertBudget(budget: import('./types').BudgetAllocation) {
    if (!this.data) return;
    if (!this.data.budgets) this.data.budgets = [];
    const idx = this.data.budgets.findIndex((b) => b.id === budget.id);
    if (idx >= 0) this.data.budgets[idx] = budget;
    else this.data.budgets.push(budget);
    await this.save();
  }

  async deleteBudget(id: string) {
    if (!this.data || !this.data.budgets) return;
    this.data.budgets = this.data.budgets.filter((b) => b.id !== id);
    await this.save();
  }

  async upsertPlan(plan: import('./types').PaymentPlan) {
    if (!this.data) return;
    if (!this.data.plans) this.data.plans = [];
    const idx = this.data.plans.findIndex((p) => p.id === plan.id);
    if (idx >= 0) this.data.plans[idx] = plan;
    else this.data.plans.push(plan);
    await this.save();
  }

  async deletePlan(id: string) {
    if (!this.data || !this.data.plans) return;
    this.data.plans = this.data.plans.filter((p) => p.id !== id);
    await this.save();
  }
}

export const ledger = new LedgerStore();

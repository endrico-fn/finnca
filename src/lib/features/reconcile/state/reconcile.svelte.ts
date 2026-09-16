import { SvelteMap } from 'svelte/reactivity';
import { invokeIpc } from '$lib/core/ipc/client';

export interface PostingReconcileView {
  posting_id: string;
  entry_id: string;
  date: string;
  description: string;
  amount: number;
  memo: string | null;
  reconciled: string;
  reconciled_at: number | null;
}

export interface ReconciliationStatusView {
  account_id: string;
  reconciled_balance: number;
  cleared_balance: number;
  uncleared_balance: number;
  total_balance: number;
  uncleared_postings: PostingReconcileView[];
}

export interface StatementRow {
  date: string;
  amount: number;
  description?: string;
}

export interface MatchResult {
  statement_index: number;
  posting_id: string;
  matched_amount: number;
}

export interface MatchStatementOutput {
  matches: MatchResult[];
  unmatched_statement_indices: number[];
  matched_count: number;
  unmatched_count: number;
}

class ReconcileState {
  selectedAccountId = $state<string>('');
  targetBalanceStr = $state<string>('');
  targetBalanceMinor = $state<number>(0);
  loading = $state<boolean>(false);
  error = $state<string | null>(null);

  status = $state<ReconciliationStatusView | null>(null);
  clearedMap = $state(new SvelteMap<string, boolean>());

  reconciledBalance = $derived(this.status?.reconciled_balance ?? 0);

  clearedBalance = $derived.by(() => {
    if (!this.status) return 0;
    let sum = this.status.reconciled_balance;
    for (const p of this.status.uncleared_postings) {
      if (this.clearedMap.get(p.posting_id)) {
        sum += p.amount;
      }
    }
    return sum;
  });

  unclearedPostings = $derived(this.status?.uncleared_postings ?? []);
  clearedCount = $derived.by(() => {
    let count = 0;
    for (const v of this.clearedMap.values()) {
      if (v) count++;
    }
    return count;
  });

  async selectAccount(accountId: string): Promise<void> {
    this.selectedAccountId = accountId;
    this.clearedMap.clear();
    this.targetBalanceStr = '';
    this.targetBalanceMinor = 0;
    if (!accountId) {
      this.status = null;
      return;
    }
    await this.loadStatus();
  }

  async loadStatus(): Promise<void> {
    if (!this.selectedAccountId) return;
    this.loading = true;
    this.error = null;
    try {
      this.status = await invokeIpc<ReconciliationStatusView>('get_reconciliation_status_cmd', {
        accountId: this.selectedAccountId,
      });
      for (const p of this.status.uncleared_postings) {
        if (p.reconciled === 'c') {
          this.clearedMap.set(p.posting_id, true);
        }
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  toggleCleared(postingId: string): void {
    const cur = this.clearedMap.get(postingId) ?? false;
    this.clearedMap.set(postingId, !cur);
  }

  selectAll(): void {
    if (!this.status) return;
    for (const p of this.status.uncleared_postings) {
      this.clearedMap.set(p.posting_id, true);
    }
  }

  clearAll(): void {
    this.clearedMap.clear();
  }

  async finish(): Promise<void> {
    if (!this.selectedAccountId) return;
    const clearedIds: string[] = [];
    for (const [id, isCleared] of this.clearedMap.entries()) {
      if (isCleared) clearedIds.push(id);
    }
    if (clearedIds.length === 0) return;

    this.loading = true;
    try {
      await invokeIpc('finish_reconciliation_cmd', {
        accountId: this.selectedAccountId,
        postingIds: clearedIds,
      });
      this.clearedMap.clear();
      this.targetBalanceStr = '';
      this.targetBalanceMinor = 0;
      await this.loadStatus();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      this.loading = false;
    }
  }

  async matchStatement(
    statements: StatementRow[],
    autoClear = true
  ): Promise<MatchStatementOutput> {
    if (!this.selectedAccountId) throw new Error('No account selected');
    this.loading = true;
    try {
      const res = await invokeIpc<MatchStatementOutput>('match_statement_cmd', {
        accountId: this.selectedAccountId,
        statements,
        autoClear,
      });
      await this.loadStatus();
      return res;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      this.loading = false;
    }
  }

  reset(): void {
    this.selectedAccountId = '';
    this.targetBalanceStr = '';
    this.targetBalanceMinor = 0;
    this.clearedMap.clear();
    this.status = null;
  }
}

export const reconcileState = new ReconcileState();

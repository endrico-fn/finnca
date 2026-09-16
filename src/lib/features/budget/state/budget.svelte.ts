import { invokeIpc } from '$lib/core/ipc/client';

export interface EnvelopeView {
  account_id: string;
  account_code: string;
  account_name: string;
  assigned: number;
  activity: number;
  available: number;
}

export interface BudgetMonthSummary {
  month: string;
  envelopes: EnvelopeView[];
  total_assigned: number;
  total_activity: number;
  to_be_budgeted: number;
}

export interface EnvelopeData {
  accountId: string;
  accountCode: string;
  accountName: string;
  assigned: number;
  activity: number;
  available: number;
}

export interface MonthCalculation {
  envelopes: EnvelopeData[];
  totalAssigned: number;
  totalActivity: number;
  toBeBudgeted: number;
}

class BudgetState {
  selectedMonth = $state(new Date().toISOString().slice(0, 7));
  loading = $state(false);
  error = $state<string | null>(null);
  summary = $state<BudgetMonthSummary | null>(null);

  monthCalculation: MonthCalculation = $derived.by(() => {
    if (!this.summary) {
      return {
        envelopes: [] as EnvelopeData[],
        totalAssigned: 0,
        totalActivity: 0,
        toBeBudgeted: 0,
      };
    }
    return {
      envelopes: this.summary.envelopes.map((e) => ({
        accountId: e.account_id,
        accountCode: e.account_code,
        accountName: e.account_name,
        assigned: e.assigned,
        activity: e.activity,
        available: e.available,
      })),
      totalAssigned: this.summary.total_assigned,
      totalActivity: this.summary.total_activity,
      toBeBudgeted: this.summary.to_be_budgeted,
    };
  });

  async load() {
    this.loading = true;
    this.error = null;
    try {
      this.summary = await invokeIpc<BudgetMonthSummary>('get_budget_summary_cmd', {
        month: this.selectedMonth,
      });
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  async setMonth(month: string) {
    this.selectedMonth = month;
    await this.load();
  }

  async assignBudget(accountId: string, amount: number) {
    await invokeIpc('upsert_budget_cmd', {
      input: {
        month: this.selectedMonth,
        account_id: accountId,
        amount,
      },
    });
    await this.load();
  }

  async deleteBudget(id: string) {
    await invokeIpc('delete_budget_cmd', { id });
    await this.load();
  }
}

export const budgetState = new BudgetState();

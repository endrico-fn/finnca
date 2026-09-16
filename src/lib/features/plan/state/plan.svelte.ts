import { SvelteMap, SvelteDate } from 'svelte/reactivity';
import { invokeIpc } from '$lib/core/ipc/client';
import type { JournalEntryView } from '$lib/core/ipc/bindings';
import type { CalendarDay } from '$lib/core/format/date';
import { matchesMonthlyDay } from '$lib/core/format/date';

export type PlanType = 'RECEIVABLE' | 'PAYABLE' | 'RECURRING';
export type PlanFrequency = 'DAILY' | 'WEEKLY' | 'MONTHLY';
export type PlanStatus = 'ACTIVE' | 'COMPLETED' | 'OVERDUE' | 'ARCHIVED';

export interface PaymentPlan {
  id: string;
  title: string;
  type: PlanType;
  status?: PlanStatus;
  totalAmount: number;
  installmentAmount: number;
  frequency: PlanFrequency;
  startDate: string;
  dueDate?: string;
  dayOfMonth?: number;
  fromAccountId: string;
  toAccountId: string;
  notes?: string;
  createdAt: string;
}

export interface PlanDto {
  id: string;
  title: string;
  plan_type: PlanType;
  status: PlanStatus;
  total_amount: number;
  installment_amount: number;
  frequency: PlanFrequency;
  start_date: string;
  due_date: string | null;
  day_of_month: number | null;
  from_account_id: string;
  to_account_id: string;
  notes: string | null;
  created_at: number;
}

export interface PlanProgressViewDto {
  plan: PlanDto;
  paid_amount: number;
  remaining_amount: number;
  progress_percent: number;
  is_settled: boolean;
  installments_paid_count: number;
}

export interface PlanProgressSummary {
  paidAmount: number;
  remainingAmount: number;
  progressPercent: number;
  isSettled: boolean;
  installmentsPaidCount: number;
}

export interface CreatePlanPayload {
  title: string;
  plan_type: PlanType;
  status?: PlanStatus;
  total_amount: number;
  installment_amount: number;
  frequency: PlanFrequency;
  start_date: string;
  due_date?: string | null;
  day_of_month?: number | null;
  from_account_id: string;
  to_account_id: string;
  notes?: string | null;
}

export interface DateEvents {
  txs: JournalEntryView[];
  plans: PaymentPlan[];
}

export function buildEventsByDate(
  calendarDays: CalendarDay[],
  entries: JournalEntryView[],
  plans: PaymentPlan[]
): Map<string, DateEvents> {
  const map = new SvelteMap<string, DateEvents>();

  for (const entry of entries) {
    if (!map.has(entry.date)) map.set(entry.date, { txs: [], plans: [] });
    map.get(entry.date)!.txs.push(entry);
  }

  for (const p of plans) {
    for (const d of calendarDays) {
      if (!d.isCurrentMonth) continue;
      const dayNum = parseInt(d.dateStr.slice(8, 10), 10);
      if (p.startDate && d.dateStr < p.startDate) continue;

      let match = false;
      if (p.dueDate === d.dateStr) match = true;

      if (!match) {
        if (p.frequency === 'DAILY') match = true;
        else if (p.frequency === 'WEEKLY') {
          const [y, m, day] = (p.startDate || d.dateStr).split('-').map(Number);
          const startLocalDay = new SvelteDate(y, m - 1, day, 12).getDay();
          const [cy, cm, cday] = d.dateStr.split('-').map(Number);
          const currentLocalDay = new SvelteDate(cy, cm - 1, cday, 12).getDay();
          if (startLocalDay === currentLocalDay) match = true;
        } else if (p.frequency === 'MONTHLY') {
          const [py, pm] = d.dateStr.split('-').map(Number);
          if (p.dayOfMonth != null && matchesMonthlyDay(py, pm, p.dayOfMonth, dayNum)) match = true;
        }
      }

      if (match) {
        if (!map.has(d.dateStr)) map.set(d.dateStr, { txs: [], plans: [] });
        map.get(d.dateStr)!.plans.push(p);
      }
    }
  }

  return map;
}

export { isPlanPostedOnDate, suggestInstallmentOptions } from '../planUtils';

class PlanState {
  items = $state<PlanProgressViewDto[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);

  planFilter = $state<string>('ALL');
  editingPlan = $state<PaymentPlan | null>(null);
  modalOpen = $state<boolean>(false);

  plans = $derived.by(() => {
    return this.items.map((item): PaymentPlan => ({
      id: item.plan.id,
      title: item.plan.title,
      type: item.plan.plan_type,
      status: item.plan.status,
      totalAmount: item.plan.total_amount,
      installmentAmount: item.plan.installment_amount,
      frequency: item.plan.frequency,
      startDate: item.plan.start_date,
      dueDate: item.plan.due_date ?? undefined,
      dayOfMonth: item.plan.day_of_month ?? undefined,
      fromAccountId: item.plan.from_account_id,
      toAccountId: item.plan.to_account_id,
      notes: item.plan.notes ?? undefined,
      createdAt: new Date(item.plan.created_at * 1000).toISOString(),
    }));
  });

  progressByPlanId = $derived.by(() => {
    const map = new SvelteMap<string, PlanProgressSummary>();
    for (const item of this.items) {
      map.set(item.plan.id, {
        paidAmount: item.paid_amount,
        remainingAmount: item.remaining_amount,
        progressPercent: item.progress_percent,
        isSettled: item.is_settled,
        installmentsPaidCount: item.installments_paid_count,
      });
    }
    return map;
  });

  getProgress(planId: string): PlanProgressSummary | undefined {
    return this.progressByPlanId.get(planId);
  }

  async load(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.items = await invokeIpc<PlanProgressViewDto[]>('list_plans_with_progress_cmd');
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  openCreate() {
    this.editingPlan = null;
    this.modalOpen = true;
  }

  openEdit(plan: PaymentPlan) {
    this.editingPlan = plan;
    this.modalOpen = true;
  }

  closeModal() {
    this.editingPlan = null;
    this.modalOpen = false;
  }

  async createPlan(input: CreatePlanPayload): Promise<void> {
    await invokeIpc('create_plan_cmd', { input });
    await this.load();
    this.closeModal();
  }

  async deletePlan(id: string): Promise<void> {
    await invokeIpc('delete_plan_cmd', { id });
    await this.load();
  }
}

export const planState = new PlanState();

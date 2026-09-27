import {
  listPlansWithProgressCmd,
  getLedgerTotalsCmd,
  listAccountsCmd,
  getBudgetSummaryCmd,
  type Account,
  type AccountBalanceView,
} from '$lib/core/ipc/bindings';
import { notificationState } from '$lib/core/state/notification.svelte';
import { getPref } from '$lib/core/state/prefs';
import { formatMinorToDisplay } from '$lib/core/format/currency';
import { todayString } from '$lib/core/format/date';
import type { TranslationDict } from '$lib/core/i18n/types';

const evaluatedKeys = new Set<string>();

export async function evaluateSmartNotifications(t: TranslationDict): Promise<void> {
  const notifToggles = getPref('finnca_notif_toggles', {
    due: true,
    fx: true,
    spike: true,
    integrity: true,
  });

  const todayStr = todayString();
  const currentMonth = todayStr.slice(0, 7);

  try {
    let accountViews: AccountBalanceView[] = [];
    try {
      accountViews = await listAccountsCmd();
    } catch {
      accountViews = [];
    }
    const accountsMap = new Map<string, Account>(
      accountViews.map((av) => [av.account.id, av.account])
    );

    if (notifToggles.due !== false) {
      const plans = await listPlansWithProgressCmd();
      for (const prog of plans) {
        const p = prog.plan;
        if (p.status === 'ARCHIVED' || prog.is_settled) continue;

        const planCurr =
          accountsMap.get(p.from_account_id)?.currency ||
          accountsMap.get(p.to_account_id)?.currency ||
          'IDR';

        const amountFormatted = formatMinorToDisplay(p.installment_amount, planCurr);
        const term = p.plan_type === 'RECEIVABLE' ? t.notifReceivable : t.notifPayableBill;

        let isDueToday = false;
        if (p.frequency === 'DAILY') {
          isDueToday = true;
        } else if (p.frequency === 'WEEKLY') {
          const [tY, tM, tD] = todayStr.split('-').map(Number);
          const todayDay = new Date(tY, tM - 1, tD).getDay();
          const [sY, sM, sD] = (p.start_date || todayStr).split('-').map(Number);
          const startDay = new Date(sY, sM - 1, sD).getDay();
          if (todayDay === startDay) isDueToday = true;
        } else if (p.frequency === 'MONTHLY' && p.day_of_month === new Date().getDate()) {
          isDueToday = true;
        } else if (p.due_date === todayStr) {
          isDueToday = true;
        }

        if (isDueToday) {
          const key = `due-today:${p.id}:${todayStr}`;
          if (!evaluatedKeys.has(key)) {
            notificationState.addNotification({
              type: 'DUE_DATE',
              priority: 'high',
              title: t.notifDueTodayTitle.replace('{term}', term),
              message: t.notifDueTodayMsg
                .replace('{desc}', p.title)
                .replace('{amount}', amountFormatted),
              actionHref: '/app/plan',
              actionLabel: t.plan,
            });
            evaluatedKeys.add(key);
          }
        } else if (p.due_date) {
          const diffDays = Math.round(
            (new Date(p.due_date).getTime() - new Date(todayStr).getTime()) / (1000 * 60 * 60 * 24)
          );

          if (diffDays > 0 && diffDays <= 3) {
            const key = `due-soon:${p.id}:${todayStr}`;
            if (!evaluatedKeys.has(key)) {
              notificationState.addNotification({
                type: 'DUE_DATE',
                priority: 'medium',
                title: t.notifDueSoonTitle.replace('{term}', term),
                message: t.notifDueSoonMsg
                  .replace('{desc}', p.title)
                  .replace('{amount}', amountFormatted)
                  .replace('{days}', String(diffDays))
                  .replace('{dueDate}', p.due_date),
                actionHref: '/app/plan',
                actionLabel: t.plan,
              });
              evaluatedKeys.add(key);
            }
          } else if (diffDays < 0) {
            const key = `overdue:${p.id}:${todayStr}`;
            if (!evaluatedKeys.has(key)) {
              notificationState.addNotification({
                type: 'DUE_DATE',
                priority: 'high',
                title: t.notifOverdueTitle.replace('{term}', term),
                message: t.notifOverdueMsg
                  .replace('{desc}', p.title)
                  .replace('{amount}', amountFormatted)
                  .replace('{days}', String(Math.abs(diffDays))),
                actionHref: '/app/plan',
                actionLabel: t.plan,
              });
              evaluatedKeys.add(key);
            }
          }
        }

        if (prog.progress_percent >= 90 && prog.progress_percent < 100) {
          const key = `plan-prog:${p.id}:${todayStr}`;
          if (!evaluatedKeys.has(key)) {
            notificationState.addNotification({
              type: 'DUE_DATE',
              priority: 'low',
              title: t.planNearCompleteTitle,
              message: t.planNearCompleteMsg
                .replace('{title}', p.title)
                .replace('{percent}', String(prog.progress_percent)),
              actionHref: '/app/plan',
              actionLabel: t.plan,
            });
            evaluatedKeys.add(key);
          }
        }
      }
    }

    if (notifToggles.spike !== false) {
      const budgetSummary = await getBudgetSummaryCmd(currentMonth).catch(() => null);

      if (budgetSummary && budgetSummary.envelopes) {
        for (const env of budgetSummary.envelopes) {
          if (env.assigned <= 0) continue;

          if (env.available < 0) {
            const key = `budget-over:${env.account_id}:${currentMonth}`;
            if (!evaluatedKeys.has(key)) {
              notificationState.addNotification({
                type: 'EXPENSE_SPIKE',
                priority: 'high',
                title: t.notifExpenseSpikeTitle,
                message: `${env.account_name} — ${t.badgeDestructive}: ${formatMinorToDisplay(Math.abs(env.available), 'IDR')}`,
                actionHref: '/app/budget',
                actionLabel: t.budget,
              });
              evaluatedKeys.add(key);
            }
          } else if (env.activity / env.assigned >= 0.85) {
            const key = `budget-near:${env.account_id}:${currentMonth}`;
            const pct = Math.round((env.activity / env.assigned) * 100);
            if (!evaluatedKeys.has(key)) {
              notificationState.addNotification({
                type: 'EXPENSE_SPIKE',
                priority: 'medium',
                title: t.budget,
                message: `${env.account_name} (${pct}%) — ${formatMinorToDisplay(env.available, 'IDR')} ${t.remainingPayables}`,
                actionHref: '/app/budget',
                actionLabel: t.budget,
              });
              evaluatedKeys.add(key);
            }
          }
        }
      }
    }

    if (notifToggles.integrity !== false) {
      const totals = await getLedgerTotalsCmd().catch(() => null);
      if (totals) {
        if (!totals.is_balance_sheet_aligned) {
          const key = `integrity-unaligned:${todayStr}`;
          if (!evaluatedKeys.has(key)) {
            notificationState.addNotification({
              type: 'LEDGER_INTEGRITY',
              priority: 'high',
              title: t.notifUnbalancedTitle,
              message: t.notifUnbalancedMsg.replace('{count}', '1'),
              detail: t.notifUnbalancedDetail,
              actionHref: '/app/journal',
              actionLabel: t.notifOpenJournal,
            });
            evaluatedKeys.add(key);
          }
        }

        if (totals.net_income < 0 && Math.abs(totals.net_income) > 0) {
          const key = `cashflow-def:${currentMonth}`;
          if (!evaluatedKeys.has(key)) {
            notificationState.addNotification({
              type: 'CASHFLOW_DEFICIT',
              priority: 'medium',
              title: t.notifCashflowDeficitTitle,
              message: t.notifCashflowDeficitMsg.replace(
                '{amount}',
                formatMinorToDisplay(Math.abs(totals.net_income), 'IDR')
              ),
              actionHref: '/app/reports',
              actionLabel: t.notifViewPnl,
            });
            evaluatedKeys.add(key);
          }
        }
      }
    }
  } catch (err) {
    console.error('Failed to run smart notifications evaluator:', err);
  }
}

import type { VaultData } from '$lib/accounting/types';
import { DEFAULT_FX_RATE } from '$lib/accounting/types';
import {
  isBalanced,
  formatIDR,
  formatUSD,
  incomeStatement,
  accountBalanceMinor,
  buildChildrenMap,
  calculatePlanProgress,
  todayString,
  diffCalendarDays,
} from '$lib/accounting/finance';
import { fetchLiveFxRate } from '$lib/fx/fx-service';
import { ledger } from '$lib/accounting/store.svelte';
import { i18n } from '$lib/i18n.svelte';

export type NotificationType =
  | 'DUE_DATE'
  | 'FX_ALERT'
  | 'EXPENSE_SPIKE'
  | 'CASHFLOW_DEFICIT'
  | 'INACTIVITY'
  | 'LEDGER_INTEGRITY';

export interface AppNotification {
  id: string;
  type: NotificationType;
  title: string;
  message: string;
  detail?: string;
  timestamp: string;
  read: boolean;
  priority: 'low' | 'medium' | 'high';
  actionHref?: string;
  actionLabel?: string;
}

class NotificationStore {
  notifications = $state<AppNotification[]>([]);
  activeToasts = $state<AppNotification[]>([]);
  drawerOpen = $state(false);
  lastFxCheck = $state<number>(0);
  isFetchingFx = $state(false);

  get unreadCount() {
    return this.notifications.filter((n) => !n.read).length;
  }

  private isSnoozed(key: string): boolean {
    if (typeof localStorage === 'undefined') return false;
    try {
      const raw = localStorage.getItem('finnca_snoozed_notifs');
      if (!raw) return false;
      const dict = JSON.parse(raw);
      if (dict[key] && Number(dict[key]) > Date.now()) return true;
    } catch {}
    return false;
  }

  snoozeNotification(id: string) {
    const notif = this.notifications.find((n) => n.id === id);
    if (!notif) return;
    const key = `${notif.type}:${notif.title}:${notif.message}`;
    if (typeof localStorage !== 'undefined') {
      try {
        const raw = localStorage.getItem('finnca_snoozed_notifs');
        const dict = raw ? JSON.parse(raw) : {};
        dict[key] = Date.now() + 24 * 60 * 60 * 1000;
        localStorage.setItem('finnca_snoozed_notifs', JSON.stringify(dict));
      } catch {}
    }
    this.notifications = this.notifications.filter((n) => n.id !== id);
    this.dismissToast(id);
  }

  addNotification(notif: Omit<AppNotification, 'id' | 'timestamp' | 'read'>) {
    const key = `${notif.type}:${notif.title}:${notif.message}`;
    if (this.isSnoozed(key)) return;

    // Avoid duplicate notifications with same title and message
    const exists = this.notifications.some(
      (n) => n.type === notif.type && n.title === notif.title && n.message === notif.message
    );
    if (exists) return;

    const fullNotif: AppNotification = {
      ...notif,
      id: 'notif-' + crypto.randomUUID(),
      timestamp: new Date().toISOString(),
      read: false,
    };

    // Prepend to notifications list
    this.notifications = [fullNotif, ...this.notifications];

    // Push to active toast banner
    this.activeToasts = [...this.activeToasts, fullNotif];

    // Auto-dismiss after 5.5 seconds
    setTimeout(() => {
      this.dismissToast(fullNotif.id);
    }, 5500);
  }

  dismissToast(id: string) {
    this.activeToasts = this.activeToasts.filter((t) => t.id !== id);
  }

  markAsRead(id: string) {
    const idx = this.notifications.findIndex((n) => n.id === id);
    if (idx >= 0) this.notifications[idx].read = true;
  }

  markAllAsRead() {
    for (const n of this.notifications) n.read = true;
  }

  clearAll() {
    this.notifications = [];
    this.activeToasts = [];
  }

  toggleDrawer() {
    this.drawerOpen = !this.drawerOpen;
  }

  closeDrawer() {
    this.drawerOpen = false;
  }

  /* ─── Smart Analysis Engine ─────────────────────────────────────── */
  analyzeVault(vault: VaultData) {
    if (!vault) return;
    const todayISO = todayString();

    // Read user notification toggles from settings/localStorage
    let toggles = { due: true, fx: true, spike: true, integrity: true };
    if (typeof localStorage !== 'undefined') {
      try {
        const saved = JSON.parse(localStorage.getItem('finnca_notif_toggles') ?? 'null');
        if (saved) toggles = { ...toggles, ...saved };
      } catch {}
    }

    // 1. Ledger Integrity Check
    if (toggles.integrity) {
      const unbalanced = vault.transactions.filter((t) => !isBalanced(t));
      if (unbalanced.length > 0) {
        this.addNotification({
          type: 'LEDGER_INTEGRITY',
          title: i18n.t.notifUnbalancedTitle,
          message: i18n.t.notifUnbalancedMsg.replace('{count}', String(unbalanced.length)),
          detail: i18n.t.notifUnbalancedDetail,
          priority: 'high',
          actionHref: '/app/journal',
          actionLabel: i18n.t.notifOpenJournal,
        });
      }
    }

    // 2. Due Date Monitoring (Payables & Receivables)
    if (toggles.due) {
      const byId = new Map(vault.accounts.map((a) => [a.id, a]));
      for (const tx of vault.transactions) {
        if (tx.dueDate && !tx.settled) {
          const diffDays = diffCalendarDays(tx.dueDate, todayISO);

          const isPayable = tx.splits.some((s) => byId.get(s.accountId)?.type === 'LIABILITY');
          const isReceivable = tx.splits.some(
            (s) => byId.get(s.accountId)?.type === 'ASSET' && s.amount > 0
          );
          const termLabel = isPayable
            ? i18n.t.notifPayableBill
            : isReceivable
              ? i18n.t.notifReceivable
              : i18n.t.notifDueDate;

          const totalAmount = tx.splits
            .filter((s) => s.amount > 0)
            .reduce((a, c) => a + c.amount, 0);
          const amtStr = tx.currency === 'USD' ? formatUSD(totalAmount) : formatIDR(totalAmount);

          if (diffDays < 0) {
            // Overdue
            this.addNotification({
              type: 'DUE_DATE',
              title: i18n.t.notifOverdueTitle.replace('{term}', termLabel),
              message: i18n.t.notifOverdueMsg
                .replace('{desc}', tx.description)
                .replace('{amount}', amtStr)
                .replace('{days}', String(Math.abs(diffDays))),
              detail: `${i18n.t.notifDueDate}: ${tx.dueDate}`,
              priority: 'high',
              actionHref: '/app/journal',
              actionLabel: i18n.t.notifViewTx,
            });
          } else if (diffDays === 0) {
            // Today
            this.addNotification({
              type: 'DUE_DATE',
              title: i18n.t.notifDueTodayTitle.replace('{term}', termLabel),
              message: i18n.t.notifDueTodayMsg
                .replace('{desc}', tx.description)
                .replace('{amount}', amtStr),
              priority: 'high',
              actionHref: '/app/journal',
              actionLabel: i18n.t.notifViewTx,
            });
          } else if (diffDays <= 3) {
            // 1 - 3 days ahead
            this.addNotification({
              type: 'DUE_DATE',
              title: i18n.t.notifDueSoonTitle.replace('{term}', termLabel),
              message: i18n.t.notifDueSoonMsg
                .replace('{desc}', tx.description)
                .replace('{amount}', amtStr)
                .replace('{days}', String(diffDays))
                .replace('{dueDate}', tx.dueDate ?? ''),
              priority: 'medium',
              actionHref: '/app/journal',
              actionLabel: i18n.t.notifViewTx,
            });
          }
        }
      }

      // Monitor Active Debt & Receivable Plans
      if (vault.plans && vault.plans.length > 0) {
        for (const plan of vault.plans) {
          const prog = calculatePlanProgress(plan, vault);
          if (prog.isSettled) {
            this.addNotification({
              type: 'DUE_DATE',
              title: i18n.t.planSettledNotifTitle,
              message: `${plan.title} — 100% (${formatIDR(plan.totalAmount)})`,
              detail: i18n.t.planSettledNotifDetail,
              priority: 'low',
              actionHref: '/app/plan',
              actionLabel: i18n.t.viewPlanBtn,
            });
          } else if (plan.dueDate) {
            const diffDays = diffCalendarDays(plan.dueDate, todayISO);
            const planTerm =
              plan.type === 'RECEIVABLE' ? i18n.t.planTermReceivable : i18n.t.planTermPayable;

            if (diffDays < 0) {
              this.addNotification({
                type: 'DUE_DATE',
                title: i18n.t.planOverdueTitle.replace('{term}', planTerm),
                message: i18n.t.planOverdueMsg
                  .replace('{title}', plan.title)
                  .replace('{amount}', formatIDR(prog.remainingAmount))
                  .replace('{days}', String(Math.abs(diffDays))),
                priority: 'high',
                actionHref: '/app/plan',
                actionLabel: i18n.t.viewPlanBtn,
              });
            } else if (diffDays <= 2) {
              this.addNotification({
                type: 'DUE_DATE',
                title: i18n.t.planDueTitle.replace('{term}', planTerm),
                message: (diffDays === 0 ? i18n.t.planDueMsgToday : i18n.t.planDueMsgDays)
                  .replace('{title}', plan.title)
                  .replace('{amount}', formatIDR(prog.remainingAmount))
                  .replace('{days}', String(diffDays)),
                priority: 'high',
                actionHref: '/app/plan',
                actionLabel: i18n.t.viewPlanBtn,
              });
            }
          }
        }
      }
    }

    // 3. Cash Flow Deficit Check (Current Month)
    const currentMonth = todayISO.slice(0, 7);
    const pnl = incomeStatement(vault, `${currentMonth}-01`, `${currentMonth}-31`);
    if (pnl.expense > pnl.income && pnl.expense > 0) {
      const deficit = pnl.expense - pnl.income;
      this.addNotification({
        type: 'CASHFLOW_DEFICIT',
        title: i18n.t.notifCashflowDeficitTitle,
        message: i18n.t.notifCashflowDeficitMsg.replace('{amount}', formatIDR(deficit)),
        detail: `${i18n.t.income}: ${formatIDR(pnl.income)} | ${i18n.t.expense}: ${formatIDR(pnl.expense)}`,
        priority: 'medium',
        actionHref: '/app/reports',
        actionLabel: i18n.t.notifViewPnl,
      });
    }

    // 4. Inactivity Reminder
    if (vault.transactions.length > 0) {
      const sortedTxs = [...vault.transactions].sort((a, b) => b.date.localeCompare(a.date));
      const lastDate = sortedTxs[0].date;
      const daysSince = diffCalendarDays(todayISO, lastDate);
      if (daysSince >= 3) {
        this.addNotification({
          type: 'INACTIVITY',
          title: i18n.t.notifInactivityTitle,
          message: i18n.t.notifInactivityMsg.replace('{days}', String(daysSince)),
          detail: i18n.t.notifInactivityDetail,
          priority: 'low',
          actionHref: '/app/journal',
          actionLabel: i18n.t.newTransaction,
        });
      }
    }

    // 5. Expense Spike Analysis
    if (toggles.spike) {
      const byId = new Map(vault.accounts.map((a) => [a.id, a]));
      const todayExpenses = vault.transactions
        .filter((t) => t.date === todayISO)
        .flatMap((t) => t.splits)
        .filter((s) => byId.get(s.accountId)?.type === 'EXPENSE' && s.amount > 0)
        .reduce((a, c) => a + c.amount, 0);

      if (todayExpenses > 0) {
        const pastDate = new Date();
        pastDate.setDate(pastDate.getDate() - 30);
        const thirtyDaysAgo = todayString(pastDate);
        const pastExpenses = vault.transactions
          .filter((t) => t.date >= thirtyDaysAgo && t.date < todayISO)
          .flatMap((t) => t.splits)
          .filter((s) => byId.get(s.accountId)?.type === 'EXPENSE' && s.amount > 0)
          .reduce((a, c) => a + c.amount, 0);

        const dailyAvg = pastExpenses / 30;
        if (dailyAvg > 0 && todayExpenses > dailyAvg * 2.5 && todayExpenses > 500_000) {
          this.addNotification({
            type: 'EXPENSE_SPIKE',
            title: i18n.t.notifExpenseSpikeTitle,
            message: i18n.t.notifExpenseSpikeMsg.replace('{amount}', formatIDR(todayExpenses)),
            priority: 'medium',
            actionHref: '/app/journal',
            actionLabel: i18n.t.notifViewTx,
          });
        }
      }
    }

    // 6. Duplicate Transaction Detection
    if (vault.transactions.length > 1) {
      const pastDate = new Date();
      pastDate.setDate(pastDate.getDate() - 30);
      const thirtyDaysAgo = todayString(pastDate);
      const recentTxs = vault.transactions.filter((t) => t.date >= thirtyDaysAgo);

      const seen = new Set<string>();
      let dupFound = false;

      for (const tx of recentTxs) {
        // Create a unique fingerprint: Date + Accounts + Amounts
        // Exclude ID and description variations
        const splitsFingerprint = tx.splits
          .map((s) => `${s.accountId}:${s.amount}`)
          .sort()
          .join('|');

        const fingerprint = `${tx.date}|${splitsFingerprint}`;

        if (seen.has(fingerprint) && tx.splits.length > 0) {
          dupFound = true;
          // Calculate amount for display (sum of debits)
          const totalAmt = tx.splits
            .filter((s) => s.amount > 0)
            .reduce((sum, s) => sum + s.amount, 0);
          const amtStr = tx.currency === 'USD' ? formatUSD(totalAmt) : formatIDR(totalAmt);

          this.addNotification({
            type: 'LEDGER_INTEGRITY',
            title: i18n.t.notifDupTxTitle,
            message: i18n.t.notifDupTxMsg
              .replace('{date}', tx.date)
              .replace('{amount}', amtStr),
            detail: i18n.t.notifDupTxDetail,
            priority: 'medium',
            actionHref: `/app/journal?search=${encodeURIComponent(tx.date)}`,
            actionLabel: i18n.t.notifDupTxCheck,
          });
          break; // Show one warning at a time to avoid spam
        }
        seen.add(fingerprint);
      }
    }
  }

  /* ─── Live USD Exchange Rate Auto-Check (One-Shot Online) ─────── */
  async checkLiveFxRate(
    vault: VaultData | null,
    force = false
  ): Promise<{
    success: boolean;
    rate?: number;
    diff?: number;
    changePct?: number;
  }> {
    if (!vault) return { success: false };
    if (this.isFetchingFx) return { success: false };
    const now = Date.now();
    // Throttle: once every 10 minutes unless forced
    if (!force && now - this.lastFxCheck < 600_000) return { success: false };
    this.lastFxCheck = now;
    this.isFetchingFx = true;

    try {
      const liveRate = await fetchLiveFxRate();
      if (liveRate !== null) {
        const currentRate = vault.fxRate ?? DEFAULT_FX_RATE;
        const pctChange = ((liveRate - currentRate) / currentRate) * 100;
        const diffRp = liveRate - currentRate;

        if (typeof localStorage !== 'undefined') {
          localStorage.setItem('finnca_last_fx_sync', new Date().toISOString());
        }

        // Calculate USD Asset Impact
        const cmap = buildChildrenMap(vault.accounts);
        let totalUsdMinor = 0;
        for (const a of vault.accounts) {
          if (a.currency === 'USD' && a.type === 'ASSET' && !a.placeholder) {
            totalUsdMinor += accountBalanceMinor(a.id, vault, cmap);
          }
        }
        const usdMajor = totalUsdMinor / 100;
        const impactIdr = Math.round(usdMajor * diffRp);

        // Check user notification toggles
        let fxToggle = true;
        if (typeof localStorage !== 'undefined') {
          try {
            const saved = JSON.parse(localStorage.getItem('finnca_notif_toggles') ?? 'null');
            if (saved && saved.fx === false) fxToggle = false;
          } catch {}
        }

        // Trigger notification if rate moved (increase or decrease)
        if (fxToggle && Math.abs(pctChange) >= 0.5) {
          const isUp = diffRp > 0;
          const sign = isUp ? '+' : '';
          const arrow = isUp ? '▲' : '▼';
          const alertTitle = isUp
            ? `${i18n.t.fxAlertIncrease} ${arrow} (+${pctChange.toFixed(2)}%)`
            : `${i18n.t.fxAlertDecrease} ${arrow} (${pctChange.toFixed(2)}%)`;

          let impactMsg = '';
          if (usdMajor > 0) {
            impactMsg = i18n.t.notifFxImpact
              .replace('{usd}', formatUSD(totalUsdMinor))
              .replace('{sign}', isUp ? '+' : '')
              .replace('{impact}', formatIDR(Math.abs(impactIdr)));
          }

          this.addNotification({
            type: 'FX_ALERT',
            title: alertTitle,
            message: i18n.t.notifFxMoveMsg
              .replace('{rate}', formatIDR(liveRate))
              .replace('{diff}', `${sign}${formatIDR(Math.abs(diffRp))}`)
              .replace('{impact}', impactMsg),
            detail: `${i18n.t.fxAutoUpdatedMsg.replace('{rate}', formatIDR(liveRate)).replace('{diff}', `${sign}${formatIDR(Math.abs(diffRp))}`)} Previous: ${formatIDR(currentRate)}.`,
            priority: Math.abs(pctChange) >= 1.0 ? 'high' : 'medium',
            actionHref: '/app/setting',
            actionLabel: i18n.t.settings,
          });
        }

        // Automatically update vault rate and record history (zero manual input needed)
        if (diffRp !== 0) {
          await ledger.setFxRate(liveRate);
        } else {
          await ledger.recordFxHistory(liveRate);
        }

        return {
          success: true,
          rate: liveRate,
          diff: diffRp,
          changePct: pctChange,
        };
      }
      return { success: false };
    } catch {
      return { success: false };
    } finally {
      this.isFetchingFx = false;
    }
  }
}

export const notifStore = new NotificationStore();

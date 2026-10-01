import { getPref, setPref } from '$lib/core/state/prefs';
export type NotificationType =
  | 'DUE_DATE'
  | 'FX_ALERT'
  | 'EXPENSE_SPIKE'
  | 'CASHFLOW_DEFICIT'
  | 'LEDGER_INTEGRITY'
  | 'INFO'
  | 'GREETING';

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


export function getNotificationBorderClass(type: NotificationType): string {
  switch (type) {
    case 'DUE_DATE':
    case 'LEDGER_INTEGRITY':
      return 'border-l-expense';
    case 'FX_ALERT':
      return 'border-l-income';
    case 'EXPENSE_SPIKE':
    case 'CASHFLOW_DEFICIT':
      return 'border-l-warning';
    case 'GREETING':
    default:
      return 'border-l-teal';
  }
}

class NotificationState {
  notifications = $state<AppNotification[]>([]);
  activeToasts = $state<AppNotification[]>([]);
  drawerOpen = $state(false);
  dismissCallback: ((id: string) => void) | null = null;

  toggleDrawer() {
    this.drawerOpen = !this.drawerOpen;
  }

  openDrawer() {
    this.drawerOpen = true;
  }

  closeDrawer() {
    this.drawerOpen = false;
  }

  get unreadCount() {
    return this.notifications.filter((n) => !n.read).length;
  }

  private isSnoozed(key: string): boolean {
    const dict = getPref('finnca_snoozed_notifs', {});
    return Boolean(dict[key] && Number(dict[key]) > Date.now());
  }

  snoozeNotification(id: string) {
    const notif = this.notifications.find((n) => n.id === id);
    if (!notif) return;
    const key = `${notif.type}:${notif.title}:${notif.message}`;
    const dict = getPref('finnca_snoozed_notifs', {});
    dict[key] = Date.now() + 24 * 60 * 60 * 1000;
    setPref('finnca_snoozed_notifs', dict);
    this.notifications = this.notifications.filter((n) => n.id !== id);
    this.dismissToast(id);
  }

  addNotification(notif: Omit<AppNotification, 'id' | 'timestamp' | 'read'>) {
    const key = `${notif.type}:${notif.title}:${notif.message}`;
    if (this.isSnoozed(key)) return;

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

    this.notifications = [fullNotif, ...this.notifications];
    this.activeToasts = [...this.activeToasts, fullNotif];

    setTimeout(() => {
      if (this.dismissCallback) {
        this.dismissCallback(fullNotif.id);
      } else {
        this.dismissToast(fullNotif.id);
      }
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
}

export const notificationState = new NotificationState();

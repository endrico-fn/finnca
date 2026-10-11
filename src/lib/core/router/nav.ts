import type { TranslationDict } from '$lib/core/i18n/types';

export interface NavItem {
  href: string;
  label: string;
}

export function getAppNavItems(t: TranslationDict): NavItem[] {
  return [
    { href: '/app', label: t.dashboard },
    { href: '/app/accounts', label: t.account },
    { href: '/app/journal', label: t.journal },
    { href: '/app/budget', label: t.budget },
    { href: '/app/reports', label: t.report },
    { href: '/app/reconcile', label: t.reconcile },
    { href: '/app/plan', label: t.plan },
    { href: '/app/audit', label: t.auditLogTitle },
    { href: '/app/setting', label: t.settings },
  ];
}

import type { Account, JournalEntryView, CreateJournalEntryInput } from '$lib/core/ipc/bindings';
import { accountTypeLabel } from '$lib/core/format/account';
import { formatIDR, formatMinorGrouping } from '$lib/core/format/currency';
import { todayString } from '$lib/core/format/date';
import type { TranslationDict } from '$lib/core/i18n/types';
import type { NavItem } from '$lib/core/router/nav';

export type { NavItem };

export interface PaletteItem {
  label: string;
  sub: string;
  category: string;
  action: () => void;
}

export function searchPalette(options: {
  query: string;
  nav: NavItem[];
  accounts: Account[];
  entries: JournalEntryView[];
  t: TranslationDict;
  onQuickTxDraft: (draft: CreateJournalEntryInput) => void;
  onLock: () => void;
  onTransfer: () => void;
  onNavigate: (href: string, param?: Record<string, string>) => void;
  onClose: () => void;
}): PaletteItem[] {
  const {
    query,
    nav,
    accounts,
    entries,
    t,
    onQuickTxDraft,
    onLock,
    onTransfer,
    onNavigate,
    onClose,
  } = options;
  const q = query.toLowerCase().trim();
  const results: PaletteItem[] = [];

  for (const item of nav) {
    if (!q || item.label.toLowerCase().includes(q) || item.href.includes(q)) {
      results.push({
        label: item.label,
        sub: t.cmdGoTo.replace('{page}', item.href),
        category: t.cmdCategoryNav,
        action: () => {
          onNavigate(item.href);
          onClose();
        },
      });
    }
  }

  if (!q || 'lock vault'.includes(q) || 'kunci'.includes(q)) {
    results.push({
      label: t.cmdLockVault,
      sub: t.cmdLockVaultSub,
      category: t.cmdCategorySecurity,
      action: () => {
        onLock();
        onClose();
      },
    });
  }

  if (!q || 'transfer'.includes(q) || 'pemindahan'.includes(q)) {
    results.push({
      label: t.transfersTitle,
      sub: t.quickTransferTitle,
      category: t.cmdCategoryQuickAdd,
      action: () => {
        onTransfer();
        onClose();
      },
    });
  }

  for (const acc of accounts.filter((a) => !a.placeholder)) {
    if (!q || acc.code.toLowerCase().includes(q) || acc.name.toLowerCase().includes(q)) {
      results.push({
        label: `${acc.code} — ${acc.name}`,
        sub: `${accountTypeLabel(acc.account_type)} • ${acc.currency}`,
        category: t.cmdCategoryAccounts,
        action: () => {
          onNavigate('/app/accounts/[code]', { code: acc.code });
          onClose();
        },
      });
    }
  }

  const shorthandMatch = q.match(/^\+\s*(.+)$/);
  if (shorthandMatch) {
    const rest = shorthandMatch[1].trim();
    const amtMatch = rest.match(/(\d+(?:[.,]\d+)?)\s*(k|rb|m|jt)?\b/i);
    let parsedAmt = 0;
    let desc = rest;
    if (amtMatch) {
      const strVal = amtMatch[1].replace(',', '.');
      const unit = (amtMatch[2] || '').toLowerCase();
      let multiplier = 1;
      if (unit === 'k' || unit === 'rb') multiplier = 1_000;
      else if (unit === 'm' || unit === 'jt') multiplier = 1_000_000;

      const parts = strVal.split('.');
      const majorStr = parts[0] || '0';
      const minorStr = (parts[1] || '').padEnd(6, '0').slice(0, 6);

      const major = parseInt(majorStr, 10) * multiplier;
      const minor = Math.round((parseInt(minorStr, 10) * multiplier) / 1_000_000);

      parsedAmt = major + minor;
      desc = rest.replace(amtMatch[0], '').replace(/\s+/g, ' ').trim();
    }

    if (parsedAmt > 0 && desc) {
      let fromAccId = '';
      let toAccId = '';
      for (const acc of accounts.filter((a) => !a.placeholder)) {
        const accLower = acc.name.toLowerCase();
        if (
          desc.toLowerCase().includes(`dari ${accLower}`) ||
          desc.toLowerCase().includes(`from ${accLower}`)
        ) {
          fromAccId = acc.id;
        }
        if (
          desc.toLowerCase().includes(`ke ${accLower}`) ||
          desc.toLowerCase().includes(`to ${accLower}`)
        ) {
          toAccId = acc.id;
        }
      }

      results.unshift({
        label: t.cmdParsedTransaction
          .replace('{desc}', desc)
          .replace('{amt}', formatIDR(parsedAmt)),
        sub: t.cmdQuickEntryPrompt,
        category: t.cmdCategoryQuickAdd,
        action: () => {
          onQuickTxDraft({
            id: crypto.randomUUID(),
            date: todayString(),
            description: desc,
            notes: '',
            reference_no: null,
            due_date: null,
            currency: 'IDR',
            fx_rate: null,
            postings: [
              {
                id: crypto.randomUUID(),
                account_id: fromAccId,
                amount: -parsedAmt,
                reconcile: 'n',
              },
              {
                id: crypto.randomUUID(),
                account_id: toAccId,
                amount: parsedAmt,
                reconcile: 'n',
              },
            ],
          });
          onClose();
        },
      });
    }
  }

  const quickAddMatch = q.match(/^(\d+)\s+(.*)$/);
  if (quickAddMatch) {
    const amtStr = quickAddMatch[1];
    const desc = quickAddMatch[2].trim();
    if (amtStr && desc) {
      const parsedAmt = parseInt(amtStr, 10);
      results.push({
        label: t.cmdRecordTxPrompt
          .replace('{amount}', formatMinorGrouping(parsedAmt, 'IDR'))
          .replace('{desc}', desc),
        sub: t.cmdRecordTxSub,
        category: t.cmdCategoryQuickAdd,
        action: () => {
          onQuickTxDraft({
            id: crypto.randomUUID(),
            date: todayString(),
            description: desc,
            notes: '',
            reference_no: null,
            due_date: null,
            currency: 'IDR',
            fx_rate: null,
            postings: [
              {
                id: crypto.randomUUID(),
                account_id: '',
                amount: -parsedAmt,
                reconcile: 'n',
              },
              {
                id: crypto.randomUUID(),
                account_id: '',
                amount: parsedAmt,
                reconcile: 'n',
              },
            ],
          });
          onClose();
        },
      });
    }
  }

  if (q.length > 2) {
    let foundCount = 0;
    for (const tx of entries) {
      if (
        tx.description.toLowerCase().includes(q) ||
        (tx.reference_no && tx.reference_no.toLowerCase().includes(q)) ||
        (tx.notes && tx.notes.toLowerCase().includes(q))
      ) {
        const totalAmt = tx.postings
          .filter((p) => p.amount > 0)
          .reduce((sum, p) => sum + p.amount, 0);
        results.push({
          label: tx.description,
          sub: `${tx.date} • ${tx.currency} ${formatMinorGrouping(totalAmt, tx.currency)}`,
          category: t.cmdCategoryTransaction,
          action: () => {
            onNavigate(`/app/journal?search=${encodeURIComponent(tx.description)}`);
            onClose();
          },
        });
        foundCount++;
        if (foundCount >= 5) break;
      }
    }
  }

  return results.slice(0, 12);
}

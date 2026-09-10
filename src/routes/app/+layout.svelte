<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { goto } from '$app/navigation';
  import { store } from '$lib/stores/app-store.svelte';
  import { ledger } from '$lib/accounting/store.svelte';
  import { notifStore } from '$lib/notifications/store.svelte';
  import NotificationToast from '$lib/components/NotificationToast.svelte';
  import NotificationDrawer from '$lib/components/NotificationDrawer.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { page } from '$app/state';
  import { Icon } from '$lib/components/ui';
  import VaultSwitcher from '$lib/components/VaultSwitcher.svelte';
  import { APP_NAME } from '$lib/types';

  let { children } = $props();

  // ── Smart Search / Command Palette (Ctrl + K) ──
  let commandPaletteOpen = $state(false);
  let paletteQuery = $state('');
  let selectedPaletteIdx = $state(0);

  import type { Transaction } from '$lib/accounting/types';
  import TransactionEditor from '$lib/components/TransactionEditor.svelte';
  import { ModalShell } from '$lib/components/ui';
  import {
    balanceSheet,
    isBalanced,
    formatIDR,
    todayString,
    roundHalfToEven,
  } from '$lib/accounting/finance';

  let quickTxDraft = $state<Transaction | null>(null);
  let healthModalOpen = $state(false);

  const healthStats = $derived.by(() => {
    if (!ledger.data) return null;
    const bs = balanceSheet(ledger.data, undefined, ledger.childrenMap);
    const discrepancy = bs.assets - (bs.liabilities + bs.equity + bs.netIncome);
    const unbalancedTxCount = ledger.data.transactions.filter((tx) => !isBalanced(tx)).length;
    return {
      ...bs,
      discrepancy,
      unbalancedTxCount,
    };
  });

  // ── Auto-Lock Inactivity Timer ──
  // Mode "on-reboot": gunakan boot_id Linux untuk fast-unlock — cek via Rust get_boot_id vs config.boot_id
  let lastActivity = Date.now();
  let getInactivityTimeout = () => {
    if (typeof localStorage === 'undefined') return 15 * 60 * 1000;
    const raw = localStorage.getItem('finnca_lock_timeout');
    if (raw === 'never') return Infinity;
    const minutes = parseInt(raw ?? '15', 10);
    return (isNaN(minutes) || minutes < 1 ? 15 : minutes) * 60 * 1000;
  };
  let bootIdChecked = $state(false);

  function resetInactivity() {
    lastActivity = Date.now();
  }

  async function checkBootIdFastUnlock() {
    if (bootIdChecked) return;
    bootIdChecked = true;
    const mode = store.appState?.settings.auto_lock_mode;
    if (mode !== 'on-reboot') return;
    const savedBootId = store.appState?.settings.boot_id;
    if (!savedBootId) return;
    try {
      const { getBootId } = await import('$lib/api');
      const currentBootId = await getBootId();
      if (currentBootId.trim() !== savedBootId.trim()) {
        // Reboot detected — force lock
        await doLock();
      }
      // same boot_id → stay unlocked (fast-unlock)
    } catch {
      // non-Linux or read failed — fallback to always-lock behavior
    }
  }

  onMount(() => {
    (async () => {
      try {
        await store.refresh();
      } catch {
        goto('/login', { replaceState: true });
        return;
      }
      if (!store.appState?.unlocked) {
        goto('/login', { replaceState: true });
      } else {
        await ledger.load();
        if (ledger.data) {
          notifStore.analyzeVault(ledger.data);
          notifStore.checkLiveFxRate(ledger.data);
        }
        await checkBootIdFastUnlock();
      }
    })();

    // Inactivity heartbeat timer — skip when on-reboot + same boot_id (fast-unlock active)
    const interval = setInterval(() => {
      const mode = store.appState?.settings.auto_lock_mode;
      const isOnRebootFastUnlock = mode === 'on-reboot';
      if (isOnRebootFastUnlock) return; // no inactivity lock in fast-unlock mode
      if (store.appState?.unlocked && Date.now() - lastActivity > getInactivityTimeout()) {
        doLock();
      }

      // Periodic one-shot live FX rate check (internally throttled to max once per 10 mins)
      if (store.appState?.unlocked && ledger.data) {
        notifStore.checkLiveFxRate(ledger.data);
      }
    }, 10000);

    const handleOnline = () => {
      if (store.appState?.unlocked && ledger.data) {
        notifStore.checkLiveFxRate(ledger.data);
      }
    };

    // Global Keydown Handler for Command Palette (Ctrl + K) & Escape
    const handleGlobalKey = (e: KeyboardEvent) => {
      resetInactivity();
      if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K')) {
        e.preventDefault();
        commandPaletteOpen = !commandPaletteOpen;
        paletteQuery = '';
        selectedPaletteIdx = 0;
      } else if (e.key === 'Escape' && commandPaletteOpen) {
        commandPaletteOpen = false;
      }
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('keydown', handleGlobalKey);
    window.addEventListener('mousemove', resetInactivity);
    window.addEventListener('mousedown', resetInactivity);
    window.addEventListener('scroll', resetInactivity, true);
    window.addEventListener('touchstart', resetInactivity);

    return () => {
      clearInterval(interval);
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('keydown', handleGlobalKey);
      window.removeEventListener('mousemove', resetInactivity);
      window.removeEventListener('mousedown', resetInactivity);
      window.removeEventListener('scroll', resetInactivity, true);
      window.removeEventListener('touchstart', resetInactivity);
    };
  });

  // Filtered command palette results
  const paletteResults = $derived.by(() => {
    const q = paletteQuery.toLowerCase().trim();
    const results: Array<{
      label: string;
      sub: string;
      category: string;
      action: () => void;
    }> = [];

    // System Navigation
    for (const item of nav) {
      if (!q || item.label.toLowerCase().includes(q) || item.href.includes(q)) {
        results.push({
          label: item.label,
          sub: i18n.t.cmdGoTo.replace('{page}', item.href),
          category: i18n.t.cmdCategoryNav,
          action: () => {
            goto(item.href);
            commandPaletteOpen = false;
          },
        });
      }
    }

    // Quick Actions
    if (!q || 'lock vault'.includes(q) || 'kunci'.includes(q)) {
      results.push({
        label: i18n.t.cmdLockVault,
        sub: i18n.t.cmdLockVaultSub,
        category: i18n.t.cmdCategorySecurity,
        action: () => {
          doLock();
          commandPaletteOpen = false;
        },
      });
    }

    // Accounts Search — goto detail page by code
    for (const acc of ledger.accounts.filter((a) => !a.placeholder)) {
      if (!q || acc.code.toLowerCase().includes(q) || acc.name.toLowerCase().includes(q)) {
        results.push({
          label: `${acc.code} — ${acc.name}`,
          sub: `${acc.type} • ${acc.currency}`,
          category: i18n.t.cmdCategoryAccounts,
          action: () => {
            goto(`/app/accounts/${acc.code}`);
            commandPaletteOpen = false;
          },
        });
      }
    }

    // Shorthand Quick Entry: starts with '+'
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
        for (const acc of ledger.accounts.filter((a) => !a.placeholder)) {
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
          label: i18n.t.cmdParsedTransaction
            .replace('{desc}', desc)
            .replace('{amt}', formatIDR(parsedAmt)),
          sub: i18n.t.cmdQuickEntryPrompt,
          category: i18n.t.cmdCategoryQuickAdd,
          action: () => {
            quickTxDraft = {
              id: '',
              date: todayString(),
              description: desc,
              currency: 'IDR',
              splits: [
                {
                  id: '',
                  accountId: fromAccId,
                  amount: -parsedAmt,
                  reconcile: 'n',
                },
                {
                  id: '',
                  accountId: toAccId,
                  amount: parsedAmt,
                  reconcile: 'n',
                },
              ],
            };
            commandPaletteOpen = false;
          },
        });
      }
    }

    // Quick Add Transaction (if query starts with a number)
    const quickAddMatch = q.match(/^(\d+)\s+(.*)$/);
    if (quickAddMatch) {
      const amtStr = quickAddMatch[1];
      const desc = quickAddMatch[2].trim();
      if (amtStr && desc) {
        const parsedAmt = parseInt(amtStr, 10);
        results.push({
          label: i18n.t.cmdRecordTxPrompt
            .replace('{amount}', parsedAmt.toLocaleString(i18n.locale === 'id' ? 'id-ID' : 'en-US'))
            .replace('{desc}', desc),
          sub: i18n.t.cmdRecordTxSub,
          category: i18n.t.cmdCategoryQuickAdd,
          action: () => {
            quickTxDraft = {
              id: '',
              date: todayString(),
              description: desc,
              currency: 'IDR',
              splits: [
                {
                  id: '',
                  accountId: '',
                  amount: -parsedAmt,
                  reconcile: 'n',
                },
                {
                  id: '',
                  accountId: '',
                  amount: parsedAmt,
                  reconcile: 'n',
                },
              ],
            };
            commandPaletteOpen = false;
          },
        });
      }
    }

    // Transactions Search
    if (q.length > 2) {
      let foundCount = 0;
      for (const tx of ledger.transactions) {
        if (
          tx.description.toLowerCase().includes(q) ||
          (tx.notes && tx.notes.toLowerCase().includes(q))
        ) {
          const totalAmt = tx.splits
            .filter((s) => s.amount > 0)
            .reduce((sum, s) => sum + s.amount, 0);
          results.push({
            label: tx.description,
            sub: `${tx.date} • ${tx.currency} ${totalAmt.toLocaleString(i18n.locale === 'id' ? 'id-ID' : 'en-US')}`,
            category: i18n.t.cmdCategoryTransaction,
            action: () => {
              goto(`/app/journal?search=${encodeURIComponent(tx.description)}`);
              commandPaletteOpen = false;
            },
          });
          foundCount++;
          if (foundCount >= 5) break;
        }
      }
    }

    return results.slice(0, 12);
  });

  onMount(() => {
    ledger.onSaveCallback = (snapshot) => {
      notifStore.analyzeVault(snapshot);
    };
    if (ledger.data) notifStore.analyzeVault(ledger.data);
    
    return () => {
      ledger.onSaveCallback = null;
    };
  });

  $effect(() => {
    const _locale = i18n.locale;
    untrack(() => {
      if (ledger.data) {
        notifStore.notifications = [];
        notifStore.activeToasts = [];
        notifStore.analyzeVault(ledger.data);
      }
    });
  });

  async function doLock() {
    await store.lock();
    goto('/login');
  }

  const nav = $derived([
    { href: '/app', label: i18n.t.dashboard },
    { href: '/app/accounts', label: i18n.t.account },
    { href: '/app/journal', label: i18n.t.journal },
    { href: '/app/budget', label: i18n.t.budget },
    { href: '/app/reports', label: i18n.t.report },
    { href: '/app/reconcile', label: i18n.t.reconcile },
    { href: '/app/plan', label: i18n.t.plan },
    { href: '/app/setting', label: i18n.t.settings },
  ]);

  const isActive = (href: string) =>
    page.url.pathname === href || (href !== '/app' && page.url.pathname.startsWith(href));
</script>

<svelte:head>
  <title>{APP_NAME}</title>
</svelte:head>

<div class="bg-bg-app text-text-base flex h-screen flex-col overflow-hidden">
  <!-- ── TOP BAR ── -->
  <header
    class="border-line bg-bg-app flex h-(--layout-topbar) shrink-0 items-center justify-between border-b-2 px-4"
  >
    <div class="flex items-center gap-2">
      <div class="font-proto flex items-center text-[12px] font-bold tracking-widest uppercase">
        <span class="text-text-strong">{APP_NAME}</span>
        <span class="text-text-muted mx-2 text-[10px]">/</span>
        {#if store.appState?.vault_path}
          {@const parts = store.appState.vault_path.split(/[/\\]/).filter((s) => !!s).slice(-2)}
          {#each parts as p, i}
            <span class="text-text-dim text-[10px]">{p}</span>
            {#if i < parts.length - 1}
              <span class="text-text-muted mx-2 text-[10px]">/</span>
            {/if}
          {/each}
        {:else}
          <span class="text-text-dim text-[10px]">
            vault-{store.appState?.vault_name?.toLowerCase().replace(/\s+/g, '-') ?? 'default'}
          </span>
        {/if}
      </div>
      <span class="bg-income ml-1 size-1.5 animate-pulse rounded-none" title="Vault Online"></span>

      {#if healthStats}
        <button
          type="button"
          onclick={() => (healthModalOpen = true)}
          class="flex cursor-pointer items-center gap-1.5 border px-2 py-0.5 transition-colors {healthStats.balanced &&
          healthStats.unbalancedTxCount === 0
            ? 'border-income/40 bg-income/5 hover:bg-income/10 text-income'
            : 'border-expense/50 bg-expense/10 hover:bg-expense/20 text-expense'}"
          title={i18n.t.ledgerHealthTitle}
        >
          <span
            class="size-1.5 rounded-none {healthStats.balanced &&
            healthStats.unbalancedTxCount === 0
              ? 'bg-income'
              : 'bg-expense animate-ping'}"
          ></span>
          <span class="font-proto text-[9px] font-bold tracking-wider">
            {healthStats.balanced && healthStats.unbalancedTxCount === 0
              ? `[OK] ${i18n.t.ledgerBalanced}`
              : `[WARN] ${i18n.t.ledgerImbalance}: ${formatIDR(Math.abs(healthStats.discrepancy))}`}
          </span>
        </button>
      {/if}
    </div>

    <div class="flex items-center gap-6">
      <button
        type="button"
        onclick={doLock}
        title={i18n.t.lockVault}
        class="text-text-icon hover:text-text-strong inline-flex h-7 w-7 items-center justify-center transition-colors"
        aria-label={i18n.t.lockVault}
      >
        <Icon name="lock" size={18} />
      </button>

      <!-- Notification Bell Button (with unread badge) -->
      <button
        type="button"
        onclick={() => notifStore.toggleDrawer()}
        class="text-text-icon hover:text-text-strong relative flex h-7 min-w-7 items-center justify-center gap-1.5 transition-colors"
        title={i18n.t.notifications}
        aria-label={i18n.t.notifications}
      >
        <Icon name="bell" size={18} />
        <span
          class="font-proto text-[12px] {notifStore.unreadCount > 0
            ? 'text-expense font-bold'
            : 'text-text-muted'}"
        >
          {notifStore.unreadCount > 0
            ? notifStore.unreadCount
            : notifStore.notifications.length > 0
              ? notifStore.notifications.length
              : 0}
        </span>
      </button>

      <!-- Settings Gear -->
      <a
        href="/app/setting"
        title={i18n.t.settings}
        class="text-text-icon hover:text-text-strong inline-flex transition-colors"
      >
        <Icon name="gear" size={18} />
      </a>
    </div>
  </header>

  <div class="relative flex flex-1 overflow-hidden">
    <aside class="border-line bg-bg-app flex w-(--layout-sidebar) shrink-0 flex-col border-r-2">
      <div class="border-line relative z-30 shrink-0 border-b p-2">
        <VaultSwitcher />
      </div>

      <nav class="flex-1 space-y-3 overflow-y-auto px-4 py-3">
        {#each nav as item (item.href)}
          {@const active = isActive(item.href)}
          <a
            href={item.href}
            class="btn-nav flex h-9.25 w-full items-center px-3.5 text-[13px] tracking-[0.08em] uppercase {active
              ? 'active'
              : ''}"
          >
            {item.label}
          </a>
        {/each}
      </nav>

      {#if healthStats}
        <div class="border-line bg-bg-card/40 shrink-0 border-t p-3">
          <button
            type="button"
            onclick={() => (healthModalOpen = true)}
            class="group w-full cursor-pointer text-left"
          >
            <div class="mb-1 flex items-center justify-between">
              <span class="font-proto text-text-dim text-[9px] font-bold tracking-wider uppercase">
                {i18n.t.ledgerHealthTitle}
              </span>
              <span
                class="font-proto border px-1.5 py-0.5 text-[9px] font-bold {healthStats.balanced &&
                healthStats.unbalancedTxCount === 0
                  ? 'border-income/40 text-income bg-income/10'
                  : 'border-expense/50 text-expense bg-expense/10'}"
              >
                {healthStats.balanced && healthStats.unbalancedTxCount === 0
                  ? healthStats.unbalancedTxCount > 0
                    ? i18n.t.ledgerImbalance
                    : `[OK] ${i18n.t.ledgerBalanced}`
                  : `[WARN] ${i18n.t.ledgerImbalance}`}
              </span>
            </div>
            <div class="font-proto text-text-muted flex items-center justify-between text-[10px]">
              <span>{i18n.t.ledgerFormula}</span>
              {#if healthStats.unbalancedTxCount > 0}
                <span class="text-expense text-[9px] font-bold">
                  {i18n.t.unbalancedTransactionsCount.replace(
                    '{count}',
                    String(healthStats.unbalancedTxCount)
                  )}
                </span>
              {/if}
            </div>
          </button>
        </div>
      {/if}
    </aside>

    <main class="bg-bg-app flex flex-1 flex-col overflow-hidden">
      {@render children()}
    </main>

    <NotificationToast />

    <NotificationDrawer />
  </div>

  {#if commandPaletteOpen}
    <div
      class="bg-overlay fixed inset-0 z-50 flex items-start justify-center p-4 pt-24 font-mono select-none"
    >
      <div class="sharp-card border-teal/60 flex w-full max-w-xl flex-col gap-3 p-4">
        <div class="flex items-center gap-3 pb-3">
          <span class="text-income text-sm font-bold">›</span>
          <!-- svelte-ignore a11y_autofocus -->
          <input
            bind:value={paletteQuery}
            placeholder={i18n.t.cmdPalettePlaceholder}
            class="text-text-strong placeholder:text-text-muted w-full bg-transparent text-[13px] focus:outline-none"
            autofocus
            onkeydown={(e) => {
              if (e.key === 'ArrowDown') {
                e.preventDefault();
                selectedPaletteIdx = (selectedPaletteIdx + 1) % Math.max(1, paletteResults.length);
              } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                selectedPaletteIdx =
                  (selectedPaletteIdx - 1 + paletteResults.length) %
                  Math.max(1, paletteResults.length);
              } else if (e.key === 'Enter' && paletteResults[selectedPaletteIdx]) {
                e.preventDefault();
                paletteResults[selectedPaletteIdx].action();
              }
            }}
          />
          <span class="text-text-dim border-line shrink-0 border px-1.5 py-0.5 text-[10px]"
            >ESC</span
          >
        </div>

        <div class="max-h-80 space-y-1 overflow-y-auto pr-1">
          {#if paletteResults.length === 0}
            <div class="text-text-dim p-4 text-center text-[11px]">
              {i18n.t.cmdPaletteNoMatch.replace('{query}', paletteQuery)}
            </div>
          {:else}
            {#each paletteResults as item, idx (item.label + item.sub)}
              <button
                type="button"
                onclick={item.action}
                onmouseenter={() => (selectedPaletteIdx = idx)}
                class="flex w-full items-center justify-between border p-2 text-left transition-colors
  {selectedPaletteIdx === idx
                  ? 'bg-bg-row-active border-teal/40'
                  : 'bg-bg-app hover:border-line border-transparent'}"
              >
                <div>
                  <div class="flex items-center gap-2">
                    <span class="bg-line text-text-base px-1 text-[9px] font-bold"
                      >{item.category}</span
                    >
                    <span
                      class="text-[12px] font-medium {selectedPaletteIdx === idx
                        ? 'text-income'
                        : 'text-text-strong'}">{item.label}</span
                    >
                  </div>
                  <p class="text-text-muted mt-0.5 ml-1 text-[10px]">
                    {item.sub}
                  </p>
                </div>
                {#if selectedPaletteIdx === idx}
                  <span class="text-teal font-proto text-[10px]">↵ ENTER</span>
                {/if}
              </button>
            {/each}
          {/if}
        </div>

        <div
          class="border-line text-text-muted flex items-center justify-between border-t pt-2 text-[10px]"
        >
          <span>{i18n.t.cmdPaletteFooter}</span>
          <span>{i18n.t.cmdPaletteTitle}</span>
        </div>
      </div>
    </div>
  {/if}

  {#if quickTxDraft}
    <ModalShell
      bind:open={
        () => !!quickTxDraft,
        (v) => {
          if (!v) quickTxDraft = null;
        }
      }
      title={i18n.t.quickTxModalTitle}
      maxWidth="max-w-4xl"
    >
      <TransactionEditor
        tx={quickTxDraft}
        onSave={async (savedTx) => {
          await ledger.upsertTransaction(savedTx);
          notifStore.addNotification({
            type: 'LEDGER_INTEGRITY',
            priority: 'low',
            title: i18n.t.quickTxSavedNotifTitle,
            message: i18n.t.quickTxSavedNotifMsg,
          });
          quickTxDraft = null;
        }}
        onCancel={() => (quickTxDraft = null)}
      />
    </ModalShell>
  {/if}

  {#if healthModalOpen && healthStats}
    <ModalShell
      bind:open={
        () => healthModalOpen,
        (v) => {
          healthModalOpen = v;
        }
      }
      title={i18n.t.ledgerHealthTitle}
      maxWidth="max-w-lg"
    >
      <div class="space-y-4 p-4 font-mono select-none">
        <div
          class="flex items-center justify-between border p-3 {healthStats.balanced &&
          healthStats.unbalancedTxCount === 0
            ? 'border-income/40 bg-income/5'
            : 'border-expense/40 bg-expense/5'}"
        >
          <div class="flex items-center gap-2">
            <span
              class="size-2 rounded-none {healthStats.balanced &&
              healthStats.unbalancedTxCount === 0
                ? 'bg-income'
                : 'bg-expense animate-ping'}"
            ></span>
            <span
              class="font-proto text-[12px] font-bold tracking-wider {healthStats.balanced &&
              healthStats.unbalancedTxCount === 0
                ? 'text-income'
                : 'text-expense'}"
            >
              {healthStats.balanced && healthStats.unbalancedTxCount === 0
                ? i18n.t.ledgerBalanced
                : i18n.t.ledgerImbalance}
            </span>
          </div>
          <span class="font-proto text-text-muted text-[11px]">
            {i18n.t.ledgerFormula}
          </span>
        </div>

        <div class="font-proto space-y-2 text-[11px]">
          <div class="border-line flex items-start justify-between border-b py-1.5">
            <span class="text-text-muted">{i18n.t.ledgerAssetsLabel} (A)</span>
            <span class="text-text-strong tabular-nums font-proto">{formatIDR(healthStats.assets)}</span>
          </div>
          <div class="border-line flex items-start justify-between border-b py-1.5">
            <span class="text-text-muted">{i18n.t.liabilities} (L)</span>
            <span class="text-text-strong tabular-nums font-proto">{formatIDR(healthStats.liabilities)}</span>
          </div>
          <div class="border-line flex items-start justify-between border-b py-1.5">
            <span class="text-text-muted">{i18n.t.equity} (E)</span>
            <span class="text-text-strong tabular-nums font-proto">{formatIDR(healthStats.equity)}</span>
          </div>
          <div class="border-line flex items-start justify-between border-b py-1.5">
            <span class="text-text-muted">{i18n.t.net} (I - X)</span>
            <span class="text-text-strong tabular-nums font-proto">{formatIDR(healthStats.netIncome)}</span>
          </div>
          <div
            class="border-line bg-bg-card flex items-start justify-between border-b px-2 py-1.5"
          >
            <span class="text-text-muted">{i18n.t.ledgerLiabEquityLabel}</span>
            <span class="text-text-strong font-bold tabular-nums font-proto">
              {formatIDR(healthStats.liabilities + healthStats.equity + healthStats.netIncome)}
            </span>
          </div>
          <div
            class="flex items-center justify-between px-2 py-1.5 {healthStats.discrepancy === 0
              ? 'text-income'
              : 'text-expense font-bold'}"
          >
            <span>{i18n.t.difference}</span>
            <span class="tabular-nums font-proto">{formatIDR(healthStats.discrepancy)}</span>
          </div>
        </div>

        {#if healthStats.unbalancedTxCount > 0 || !healthStats.balanced}
          <div class="pt-2">
            <button
              type="button"
              onclick={() => {
                healthModalOpen = false;
                goto('/app/journal');
              }}
              class="btn-action border-expense/50 text-expense hover:bg-expense/10 w-full cursor-pointer justify-center py-2 text-[11px]"
            >
              {i18n.t.viewJournalFix}
            </button>
          </div>
        {/if}
      </div>
    </ModalShell>
  {/if}
</div>

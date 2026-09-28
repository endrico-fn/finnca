<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import type { ReportTab } from '../state/report.svelte';
  import { REPORT_GROUPS, REPORT_TAB_LABELS } from '../state/reportNav';

  let {
    currentTab,
    onSelectTab,
  }: {
    currentTab: ReportTab;
    onSelectTab: (tab: ReportTab) => void;
  } = $props();

  const ORDERED_TABS: ReportTab[] = [
    'tb',
    'pnl',
    'bs',
    'cashflow',
    'income-exp',
    'spending',
    'forecast',
    'debt',
    'trends',
    'fx',
    'networth',
  ];

  const SHORTCUT_MAP: Record<ReportTab, string> = {
    tb: '1',
    pnl: '2',
    bs: '3',
    cashflow: '4',
    'income-exp': '5',
    spending: '6',
    forecast: '7',
    debt: '8',
    trends: '9',
    fx: '0',
    networth: '-',
  };

  let railElement = $state<HTMLElement | null>(null);

  function focusTab(tabId: ReportTab) {
    if (!railElement) return;
    const btn = railElement.querySelector<HTMLButtonElement>(`[data-tab="${tabId}"]`);
    btn?.focus();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const target = e.target as HTMLElement | null;
    if (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLSelectElement ||
      target?.isContentEditable
    ) {
      return;
    }

    // Do not intercept if a modal or dialog is currently open
    if (document.querySelector('[role="dialog"], [aria-modal="true"]')) {
      return;
    }

    // Direct number jump shortcuts (1..9, 0, -)
    if (e.key >= '1' && e.key <= '9') {
      const idx = Number(e.key) - 1;
      if (idx < ORDERED_TABS.length) {
        e.preventDefault();
        onSelectTab(ORDERED_TABS[idx]);
      }
    } else if (e.key === '0') {
      e.preventDefault();
      onSelectTab('fx');
    } else if (e.key === '-') {
      e.preventDefault();
      onSelectTab('networth');
    } else if (railElement && (railElement === target || railElement.contains(target))) {
      // Arrow keys, j/k, Home, End only apply when user focuses within the rail navigation
      if (e.key === 'ArrowDown' || e.key === 'j') {
        e.preventDefault();
        const currentIdx = ORDERED_TABS.indexOf(currentTab);
        const nextIdx = (currentIdx + 1) % ORDERED_TABS.length;
        const nextTab = ORDERED_TABS[nextIdx];
        onSelectTab(nextTab);
        focusTab(nextTab);
      } else if (e.key === 'ArrowUp' || e.key === 'k') {
        e.preventDefault();
        const currentIdx = ORDERED_TABS.indexOf(currentTab);
        const prevIdx = (currentIdx - 1 + ORDERED_TABS.length) % ORDERED_TABS.length;
        const prevTab = ORDERED_TABS[prevIdx];
        onSelectTab(prevTab);
        focusTab(prevTab);
      } else if (e.key === 'Home') {
        e.preventDefault();
        const firstTab = ORDERED_TABS[0];
        onSelectTab(firstTab);
        focusTab(firstTab);
      } else if (e.key === 'End') {
        e.preventDefault();
        const lastTab = ORDERED_TABS[ORDERED_TABS.length - 1];
        onSelectTab(lastTab);
        focusTab(lastTab);
      }
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  });
</script>

<aside
  bind:this={railElement}
  class="border-line bg-bg-card/40 flex h-full w-56 shrink-0 flex-col justify-between border-r select-none"
  aria-label={i18n.t.railNavTitle}
>
  <div class="flex min-h-0 flex-1 flex-col overflow-y-auto">
    <div class="border-line/40 flex items-center justify-between border-b px-3 py-2">
      <span class="font-proto text-text-dim text-smaller font-bold tracking-widest uppercase">
        {i18n.t.railNavTitle}
      </span>
      <span class="font-proto text-text-muted text-smaller">
        {ORDERED_TABS.length}
      </span>
    </div>

    <div class="space-y-3 py-2">
      {#each REPORT_GROUPS as group (group.id)}
        <div>
          <div
            class="font-proto text-text-dim text-smaller font-bold tracking-wider uppercase px-3 pt-1 pb-1"
          >
            {i18n.t[group.labelKey]}
          </div>

          <div class="space-y-0.5">
            {#each group.tabs as tabId (tabId)}
              {@const isSelected = currentTab === tabId}
              {@const shortcut = SHORTCUT_MAP[tabId]}
              <button
                type="button"
                data-tab={tabId}
                onclick={() => onSelectTab(tabId)}
                class="group font-proto text-smaller flex w-full cursor-pointer items-center justify-between px-3 py-1.5 transition-colors {isSelected
                  ? 'border-teal bg-teal/10 text-teal border-l-2 font-bold'
                  : 'text-text-muted hover:text-text-white hover:bg-bg-btn/60 border-l-2 border-transparent'}"
              >
                <span class="truncate">{i18n.t[REPORT_TAB_LABELS[tabId]]}</span>
                <span
                  class="font-proto text-smaller border px-1 transition-colors {isSelected
                    ? 'border-teal/40 text-teal'
                    : 'border-line/60 text-text-dim group-hover:text-text-muted'}"
                >
                  {shortcut}
                </span>
              </button>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>

  <div class="border-line/40 border-t px-3 py-2">
    <div class="font-proto text-text-dim text-smaller flex items-center gap-1.5">
      <span class="border-line/60 border px-1 py-0.5 font-bold">1-9, 0</span>
      <span class="border-line/60 border px-1 py-0.5 font-bold">↑↓</span>
      <span class="truncate">{i18n.t.railNavShortcutHint}</span>
    </div>
  </div>
</aside>

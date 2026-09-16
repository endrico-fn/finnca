<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { accountsState } from '$lib/features/accounts/state/accounts.svelte';
  import { journalState, type Transaction } from '$lib/features/journal/state/journalDraft.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { searchPalette, type NavItem } from './commandPaletteSearch';

  let {
    open = $bindable(false),
    onQuickTxDraft = () => {},
    onLock = () => {},
  }: {
    open: boolean;
    onQuickTxDraft?: (tx: Transaction) => void;
    onLock?: () => void;
  } = $props();

  let paletteQuery = $state('');
  let selectedPaletteIdx = $state(0);

  const nav = $derived<NavItem[]>([
    { href: '/app', label: i18n.t.dashboard },
    { href: '/app/accounts', label: i18n.t.account },
    { href: '/app/journal', label: i18n.t.journal },
    { href: '/app/budget', label: i18n.t.budget },
    { href: '/app/reports', label: i18n.t.report },
    { href: '/app/reconcile', label: i18n.t.reconcile },
    { href: '/app/plan', label: i18n.t.plan },
    { href: '/app/setting', label: i18n.t.settings },
  ]);

  function handleNavigate(href: string, params?: Record<string, string>) {
    if (params && href === '/app/accounts/[code]') {
      goto(resolve('/app/accounts/[code]', { code: params.code }));
    } else {
      goto(resolve(href as '/app'));
    }
  }

  const paletteResults = $derived(
    searchPalette({
      query: paletteQuery,
      nav,
      accounts: accountsState.accounts,
      entries: journalState.entries,
      t: i18n.t,
      locale: i18n.locale,
      onQuickTxDraft,
      onLock,
      onNavigate: handleNavigate,
      onClose: () => (open = false),
    })
  );
</script>

{#if open}
  <div class="bg-overlay fixed inset-0 z-50 flex items-start justify-center p-4 pt-24 select-none">
    <div class="sharp-card border-teal/60 flex w-full max-w-xl flex-col gap-3 p-4">
      <div class="flex items-center gap-3 pb-3">
        <span class="text-income text-medium font-bold">{i18n.t.cmdPalettePrompt}</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          bind:value={paletteQuery}
          placeholder={i18n.t.cmdPalettePlaceholder}
          class="text-text-strong placeholder:text-text-muted text-medium w-full bg-transparent focus:outline-none"
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
        <span
          class="text-text-dim border-line text-smaller font-proto shrink-0 border px-1.5 py-0.5"
        >
          {i18n.t.cmdPaletteEscBadge}
        </span>
      </div>

      <div class="max-h-80 space-y-1 overflow-y-auto pr-1">
        {#if paletteResults.length === 0}
          <div class="text-text-dim text-small font-proto p-4 text-center">
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
                  <span class="bg-line text-text-base text-smaller font-proto px-1 font-bold">
                    {item.category}
                  </span>
                  <span
                    class="text-small font-proto font-medium {selectedPaletteIdx === idx
                      ? 'text-income'
                      : 'text-text-strong'}"
                  >
                    {item.label}
                  </span>
                </div>
                <p class="text-text-muted text-smaller mt-0.5 ml-1">
                  {item.sub}
                </p>
              </div>
              {#if selectedPaletteIdx === idx}
                <span class="text-teal font-proto text-smaller">{i18n.t.cmdPaletteEnterBadge}</span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      <div
        class="border-line text-text-muted text-smaller font-proto flex items-center justify-between border-t pt-2"
      >
        <span>{i18n.t.cmdPaletteFooter}</span>
        <span>{i18n.t.cmdPaletteTitle}</span>
      </div>
    </div>
  </div>
{/if}

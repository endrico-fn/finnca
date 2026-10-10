<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { accountsState } from '$lib/features/accounts/state/accounts.svelte';
  import { journalState } from '$lib/features/journal/state/journalDraft.svelte';
  import type { CreateJournalEntryInput } from '$lib/core/ipc/bindings';
  import { i18n } from '$lib/core/i18n.svelte';
  import { getAppNavItems } from '$lib/core/router/nav';
  import { searchPalette } from './commandPaletteSearch';

  let {
    open = $bindable(false),
    onQuickTxDraft = () => {},
    onLock = () => {},
    onTransfer = () => {},
  }: {
    open: boolean;
    onQuickTxDraft?: (draft: CreateJournalEntryInput) => void;
    onLock?: () => void;
    onTransfer?: () => void;
  } = $props();

  let paletteQuery = $state('');
  let selectedPaletteIdx = $state(0);
  let inputEl: HTMLInputElement | null = $state(null);
  let optionEls: HTMLElement[] = [];

  const nav = $derived(getAppNavItems(i18n.t));

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
      onQuickTxDraft,
      onLock,
      onTransfer,
      onNavigate: handleNavigate,
      onClose: () => (open = false),
    })
  );

  $effect(() => {
    // Reset selection index when query changes
    if (paletteQuery !== undefined) {
      selectedPaletteIdx = 0;
    }
  });

  $effect(() => {
    if (open) {
      setTimeout(() => inputEl?.focus(), 0);
    }
  });

  function scrollOptionIntoView(idx: number) {
    optionEls[idx]?.scrollIntoView({ block: 'nearest' });
  }

  function handleDialogKeydown(e: KeyboardEvent) {
    if (e.key === 'Tab') {
      e.preventDefault();
      inputEl?.focus();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      open = false;
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (paletteResults.length > 0) {
        selectedPaletteIdx = (selectedPaletteIdx + 1) % paletteResults.length;
        scrollOptionIntoView(selectedPaletteIdx);
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (paletteResults.length > 0) {
        selectedPaletteIdx =
          (selectedPaletteIdx - 1 + paletteResults.length) % paletteResults.length;
        scrollOptionIntoView(selectedPaletteIdx);
      }
    } else if (e.key === 'Enter' && paletteResults[selectedPaletteIdx]) {
      e.preventDefault();
      paletteResults[selectedPaletteIdx].action();
    }
  }
</script>

{#if open}
  <div
    class="bg-overlay backdrop-blur-subtle fixed inset-0 z-(--z-palette) flex items-start justify-center p-4 pt-24 select-none"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    aria-label={i18n.t.cmdPaletteTitle}
    onkeydown={handleDialogKeydown}
    onclick={(e) => {
      if (e.target === e.currentTarget) open = false;
    }}
  >
    <div class="sharp-card border-teal/60 flex w-full max-w-xl flex-col gap-3 p-4">
      <div class="flex items-center gap-3 pb-3">
        <span class="text-income text-medium font-bold">{i18n.t.cmdPalettePrompt}</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          bind:this={inputEl}
          bind:value={paletteQuery}
          role="combobox"
          aria-expanded={open}
          aria-haspopup="listbox"
          aria-controls="command-palette-results"
          aria-autocomplete="list"
          aria-activedescendant={paletteResults[selectedPaletteIdx]
            ? `palette-opt-${selectedPaletteIdx}`
            : undefined}
          aria-label={i18n.t.cmdPalettePlaceholder}
          placeholder={i18n.t.cmdPalettePlaceholder}
          autocomplete="off"
          spellcheck="false"
          class="text-text-strong placeholder:text-text-muted text-medium w-full bg-transparent focus:outline-none"
          autofocus
        />
        <span
          class="text-text-dim border-line text-smaller font-proto shrink-0 border px-1.5 py-0.5"
        >
          {i18n.t.cmdPaletteEscBadge}
        </span>
      </div>

      <div
        id="command-palette-results"
        role="listbox"
        aria-label={i18n.t.cmdPaletteTitle}
        class="max-h-80 space-y-1 overflow-y-auto pr-1"
      >
        {#if paletteResults.length === 0}
          <div class="text-text-dim text-small font-proto p-4 text-center">
            {i18n.t.cmdPaletteNoMatch.replace('{query}', paletteQuery)}
          </div>
        {:else}
          {#each paletteResults as item, idx (item.label + item.sub)}
            <div
              role="option"
              id={`palette-opt-${idx}`}
              aria-selected={selectedPaletteIdx === idx}
              tabindex="-1"
              bind:this={optionEls[idx]}
              onclick={item.action}
              onkeydown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  item.action();
                }
              }}
              onmouseenter={() => (selectedPaletteIdx = idx)}
              class="flex w-full cursor-pointer items-center justify-between border p-2 text-left transition-colors
{selectedPaletteIdx === idx
                ? 'bg-bg-row-active border-teal/40'
                : 'bg-bg-app hover:border-line border-transparent'}"
            >
              <div>
                <div class="flex items-center gap-2">
                  <span class="bg-bg-btn text-text-base text-smaller font-proto px-1 font-bold">
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
            </div>
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

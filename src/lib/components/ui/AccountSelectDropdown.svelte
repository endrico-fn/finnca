<script lang="ts">
  import { onMount } from 'svelte';
  import type { Account, AccountType } from '$lib/core/ipc/bindings';
  import {
    ACCOUNT_TYPE_BG,
    ACCOUNT_TYPE_TEXT,
    accountTypeLabel,
    getAccountBreadcrumb,
    getAccountPath,
  } from '$lib/core/format/account';
  import { i18n } from '$lib/core/i18n.svelte';
  import Icon from './Icon.svelte';

  export interface AccountOptionItem {
    account: Account;
    path: string;
    breadcrumbs: string[];
    isPlaceholder: boolean;
  }

  let {
    accounts = [],
    accountsById,
    value = $bindable(''),
    placeholder = '',
    filterType,
    excludePlaceholder = true,
    disabled = false,
    size = 'md',
    placement = 'auto',
    class: className = '',
    menuClass = '',
    onSelect,
  }: {
    accounts: Account[];
    accountsById?: Map<string, Account>;
    value: string;
    placeholder?: string;
    filterType?: AccountType | AccountType[];
    excludePlaceholder?: boolean;
    disabled?: boolean;
    size?: 'sm' | 'md';
    placement?: 'auto' | 'top' | 'bottom';
    class?: string;
    menuClass?: string;
    onSelect?: (acc: Account) => void;
  } = $props();

  let open = $state(false);
  let query = $state('');
  let containerEl: HTMLElement | null = $state(null);
  let searchInputEl: HTMLInputElement | null = $state(null);

  const resolvedById = $derived(accountsById ?? new Map(accounts.map((a) => [a.id, a])));

  const filterTypeSet = $derived.by(() => {
    if (!filterType) return null;
    if (Array.isArray(filterType)) return new Set(filterType);
    return new Set([filterType]);
  });

  const allItems = $derived<AccountOptionItem[]>(
    accounts.map((acc) => ({
      account: acc,
      path: getAccountPath(acc, resolvedById),
      breadcrumbs: getAccountBreadcrumb(acc, resolvedById),
      isPlaceholder: acc.placeholder,
    }))
  );

  const selectableItems = $derived(
    allItems.filter((item) => {
      if (filterTypeSet && !filterTypeSet.has(item.account.account_type)) {
        return false;
      }
      if (excludePlaceholder && item.isPlaceholder) {
        return false;
      }
      return true;
    })
  );

  const selectedItem = $derived(allItems.find((item) => item.account.id === value));

  const visibleItems = $derived.by(() => {
    const q = query.toLowerCase().trim();
    if (!q) return selectableItems;
    return selectableItems.filter(
      (item) =>
        item.account.code.toLowerCase().includes(q) ||
        item.account.name.toLowerCase().includes(q) ||
        item.path.toLowerCase().includes(q)
    );
  });

  let menuCoords = $state({
    top: 0,
    left: 0,
    width: 0,
    maxListHeight: 240,
    openUpward: false,
  });

  function updateCoords() {
    if (!containerEl || typeof window === 'undefined') return;
    const rect = containerEl.getBoundingClientRect();
    if (rect.bottom < 0 || rect.top > window.innerHeight) {
      open = false;
      return;
    }
    const spaceBelow = window.innerHeight - rect.bottom;
    const spaceAbove = rect.top;
    const openUpward =
      placement === 'top'
        ? true
        : placement === 'bottom'
          ? false
          : spaceBelow < 280 && spaceAbove > spaceBelow;
    const width = Math.min(Math.max(rect.width, 320), window.innerWidth - 16);
    const left = Math.max(8, Math.min(window.innerWidth - width - 8, rect.left));
    const availableHeight = openUpward ? spaceAbove - 60 : spaceBelow - 60;
    const maxListHeight = Math.max(120, Math.min(240, Math.floor(availableHeight)));

    menuCoords = {
      top: openUpward ? rect.top : rect.bottom,
      left,
      width,
      maxListHeight,
      openUpward,
    };
  }

  function toggle() {
    if (disabled) return;
    if (!open && containerEl) {
      updateCoords();
      open = true;
      setTimeout(() => searchInputEl?.focus(), 50);
    } else {
      open = false;
    }
  }

  function handleSelect(item: AccountOptionItem) {
    if (item.isPlaceholder && excludePlaceholder) return;
    value = item.account.id;
    open = false;
    query = '';
    onSelect?.(item.account);
  }

  function handleClickOutside(e: MouseEvent) {
    if (containerEl && !containerEl.contains(e.target as Node)) {
      open = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && open) {
      open = false;
    }
  }

  function handleScrollOrResize() {
    if (open) {
      updateCoords();
    }
  }

  onMount(() => {
    window.addEventListener('click', handleClickOutside);
    window.addEventListener('keydown', handleKeydown);
    window.addEventListener('scroll', handleScrollOrResize, true);
    window.addEventListener('resize', handleScrollOrResize);
    return () => {
      window.removeEventListener('click', handleClickOutside);
      window.removeEventListener('keydown', handleKeydown);
      window.removeEventListener('scroll', handleScrollOrResize, true);
      window.removeEventListener('resize', handleScrollOrResize);
    };
  });
</script>

<div class="relative inline-block w-full text-left {className}" bind:this={containerEl}>
  <!-- Trigger Button -->
  <button
    type="button"
    onclick={toggle}
    {disabled}
    aria-haspopup="listbox"
    aria-expanded={open}
    class="sharp-btn flex w-full items-center justify-between gap-2 text-left transition-colors
      {size === 'sm' ? 'text-smaller px-2 py-1' : 'text-small px-3 py-1.5'}
      {disabled ? 'cursor-not-allowed opacity-50' : 'hover:border-text-dim cursor-pointer'}
      {open ? 'border-teal bg-bg-card' : 'bg-bg-app border-line'}"
  >
    <div class="flex min-w-0 flex-1 items-center gap-2">
      {#if selectedItem}
        <span
          class="size-2 shrink-0 {ACCOUNT_TYPE_BG[selectedItem.account.account_type]}"
          title={accountTypeLabel(selectedItem.account.account_type)}
        ></span>
        <span class="font-proto text-text-muted text-smaller shrink-0">
          {selectedItem.account.code}
        </span>
        <div class="flex min-w-0 items-center gap-1 truncate">
          {#if selectedItem.breadcrumbs.length > 1}
            <span class="font-aux text-text-dim text-smaller hidden truncate sm:inline">
              {selectedItem.breadcrumbs.slice(0, -1).join(' > ')} &gt;
            </span>
          {/if}
          <span class="font-proto text-text-base truncate font-medium">
            {selectedItem.account.name}
          </span>
        </div>
      {:else}
        <span class="font-aux text-text-muted truncate italic">
          {placeholder || i18n.t.txSelectAccount}
        </span>
      {/if}
    </div>
    <span
      class="text-text-muted shrink-0 transition-transform {open ? 'text-teal rotate-180' : ''}"
    >
      <Icon name="chev-down" size={14} />
    </span>
  </button>

  <!-- Dropdown Popover -->
  {#if open}
    <div
      class="border-line bg-bg-card fixed z-(--z-popover) border-2 shadow-xl {menuClass}"
      style="left: {menuCoords.left}px; {menuCoords.openUpward
        ? `bottom: ${typeof window !== 'undefined' ? window.innerHeight - menuCoords.top + 4 : 0}px;`
        : `top: ${menuCoords.top + 4}px;`} width: {menuCoords.width}px;"
      role="listbox"
    >
      <!-- Search Box -->
      <div class="border-line/60 bg-bg-app border-b p-2">
        <div class="relative flex items-center">
          <span class="text-text-dim pointer-events-none absolute left-2.5">
            <Icon name="search" size={14} />
          </span>
          <input
            bind:this={searchInputEl}
            type="text"
            bind:value={query}
            placeholder={i18n.t.searchAllPlaceholder}
            autocomplete="off"
            spellcheck="false"
            class="sharp-input font-proto text-smaller w-full py-1 pr-7 pl-8"
          />
          {#if query}
            <button
              type="button"
              onclick={() => (query = '')}
              class="text-text-dim hover:text-text-base absolute right-2"
              aria-label={i18n.t.searchAllPlaceholder}
            >
              <Icon name="close" size={12} />
            </button>
          {/if}
        </div>
      </div>

      <!-- Account List -->
      <div
        class="divide-line/30 divide-y overflow-y-auto"
        style="max-height: {menuCoords.maxListHeight}px;"
      >
        {#if visibleItems.length === 0}
          <div class="font-aux text-text-dim text-smaller p-4 text-center">
            {i18n.t.noMatchingAccounts}
          </div>
        {:else}
          {#each visibleItems as item (item.account.id)}
            {@const isSelected = item.account.id === value}
            <button
              type="button"
              onclick={() => handleSelect(item)}
              class="hover:bg-bg-row-active flex w-full items-center justify-between gap-2 px-3 py-2 text-left transition-colors
                {isSelected ? 'bg-bg-row-active border-teal border-l-2' : ''}"
              role="option"
              aria-selected={isSelected}
            >
              <div class="flex min-w-0 flex-1 items-start gap-2">
                <span
                  class="mt-1 size-2 shrink-0 {ACCOUNT_TYPE_BG[item.account.account_type]}"
                  title={accountTypeLabel(item.account.account_type)}
                ></span>
                <div class="min-w-0 flex-1">
                  <!-- Breadcrumb Path -->
                  <div class="font-aux text-text-dim text-smaller flex items-center gap-1 truncate">
                    <span class="font-proto text-text-muted">[{item.account.code}]</span>
                    {#if item.breadcrumbs.length > 1}
                      <span class="truncate">{item.breadcrumbs.slice(0, -1).join(' > ')}</span>
                    {/if}
                  </div>
                  <!-- Leaf Account Name -->
                  <div class="font-proto text-text-base text-small truncate font-medium">
                    {item.account.name}
                  </div>
                </div>
              </div>

              <!-- Type badge & Checkmark -->
              <div class="flex shrink-0 items-center gap-2">
                <span
                  class="font-proto text-smaller {ACCOUNT_TYPE_TEXT[
                    item.account.account_type
                  ]} opacity-80"
                >
                  {accountTypeLabel(item.account.account_type)}
                </span>
                {#if isSelected}
                  <span class="text-teal">
                    <Icon name="check" size={14} />
                  </span>
                {/if}
              </div>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>

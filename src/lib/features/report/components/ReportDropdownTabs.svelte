<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import type { ReportGroup, ReportTab } from '../state/report.svelte';
  import {
    REPORT_GROUPS,
    REPORT_TAB_LABELS,
    REPORT_SHORTCUTS,
    ORDERED_REPORT_TABS,
  } from '../state/reportNav';

  let {
    currentTab,
    onSelectTab,
  }: {
    currentTab: ReportTab;
    onSelectTab: (tab: ReportTab) => void;
  } = $props();

  let openGroupId = $state<ReportGroup | null>(null);
  let navContainerEl = $state<HTMLElement | null>(null);

  function toggleDropdown(groupId: ReportGroup) {
    openGroupId = openGroupId === groupId ? null : groupId;
  }

  function handleSelect(tabId: ReportTab) {
    onSelectTab(tabId);
    openGroupId = null;
  }

  function handleClickOutside(e: MouseEvent) {
    if (openGroupId === null) return;
    const target = e.target as Element | null;
    if (!target) return;

    if (navContainerEl && !navContainerEl.contains(target)) {
      openGroupId = null;
      return;
    }

    const clickedButton = target.closest('button');
    if (!clickedButton) {
      openGroupId = null;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (openGroupId !== null) {
        e.preventDefault();
        const prevOpenId = openGroupId;
        openGroupId = null;
        const triggerBtn = document.getElementById(`report-btn-${prevOpenId}`);
        triggerBtn?.focus();
      }
      return;
    }

    if (openGroupId !== null) {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        e.preventDefault();
        const menuEl = document.getElementById(`report-dropdown-${openGroupId}`);
        if (menuEl) {
          const items = Array.from(
            menuEl.querySelectorAll<HTMLButtonElement>('button[role="menuitem"]')
          );
          if (items.length > 0) {
            const currentIdx = items.indexOf(document.activeElement as HTMLButtonElement);
            let nextIdx: number;
            if (e.key === 'ArrowDown') {
              nextIdx = currentIdx < items.length - 1 ? currentIdx + 1 : 0;
            } else {
              nextIdx = currentIdx > 0 ? currentIdx - 1 : items.length - 1;
            }
            items[nextIdx]?.focus();
          }
        }
        return;
      }

      if (e.key === 'Home') {
        e.preventDefault();
        const menuEl = document.getElementById(`report-dropdown-${openGroupId}`);
        menuEl?.querySelector<HTMLButtonElement>('button[role="menuitem"]')?.focus();
        return;
      }

      if (e.key === 'End') {
        e.preventDefault();
        const menuEl = document.getElementById(`report-dropdown-${openGroupId}`);
        const items = menuEl?.querySelectorAll<HTMLButtonElement>('button[role="menuitem"]');
        if (items && items.length > 0) {
          items[items.length - 1]?.focus();
        }
        return;
      }
    }

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
      if (idx < ORDERED_REPORT_TABS.length) {
        e.preventDefault();
        onSelectTab(ORDERED_REPORT_TABS[idx]);
        openGroupId = null;
      }
    } else if (e.key === '0') {
      e.preventDefault();
      onSelectTab('fx');
      openGroupId = null;
    } else if (e.key === '-') {
      e.preventDefault();
      onSelectTab('networth');
      openGroupId = null;
    }
  }

  onMount(() => {
    window.addEventListener('click', handleClickOutside);
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('click', handleClickOutside);
      window.removeEventListener('keydown', handleKeyDown);
    };
  });
</script>

<nav
  bind:this={navContainerEl}
  class="font-proto text-smaller inline-flex items-center gap-1"
  aria-label={i18n.t.report}
>
  {#each REPORT_GROUPS as group (group.id)}
    {@const isCategoryActive = group.tabs.includes(currentTab)}
    {@const isOpen = openGroupId === group.id}
    <div class="relative inline-flex items-center">
      <button
        type="button"
        id="report-btn-{group.id}"
        onclick={() => toggleDropdown(group.id)}
        onkeydown={(e) => {
          if (e.key === 'ArrowDown') {
            e.preventDefault();
            openGroupId = group.id;
            requestAnimationFrame(() => {
              const menuEl = document.getElementById(`report-dropdown-${group.id}`);
              menuEl?.querySelector<HTMLButtonElement>('button[role="menuitem"]')?.focus();
            });
          }
        }}
        aria-haspopup="menu"
        aria-expanded={isOpen}
        aria-controls="report-dropdown-{group.id}"
        class="font-proto text-smaller inline-flex h-6 cursor-pointer items-center justify-center gap-1.5 rounded-none border px-2 whitespace-nowrap transition-colors select-none {isCategoryActive
          ? 'bg-bg-row-active border-line text-text-strong font-semibold'
          : isOpen
            ? 'border-line/80 bg-bg-card text-text-white font-semibold'
            : 'text-text-muted hover:border-line/40 hover:bg-bg-card hover:text-text-base border-transparent bg-transparent font-semibold'}"
      >
        {#if isCategoryActive}
          <span class="bg-teal size-1.5 shrink-0"></span>
        {/if}

        <span class="tracking-wider uppercase">{i18n.t[group.labelKey]}</span>

        <span class="inline-flex transition-transform duration-150 {isOpen ? 'rotate-180' : ''}">
          <Icon name="chev-down" size={10} />
        </span>
      </button>

      {#if isOpen}
        <div
          id="report-dropdown-{group.id}"
          role="menu"
          aria-label={i18n.t[group.labelKey]}
          class="border-line bg-bg-card absolute top-full left-0 z-30 mt-1 max-w-[calc(100vw-2rem)] min-w-64 rounded-none border py-1 shadow-xl"
        >
          {#each group.tabs as tabId (tabId)}
            {@const isSelected = currentTab === tabId}
            {@const shortcut = REPORT_SHORTCUTS[tabId]}
            <button
              type="button"
              role="menuitem"
              tabindex={isOpen ? 0 : -1}
              onclick={() => handleSelect(tabId)}
              class="group font-proto text-smaller flex w-full cursor-pointer items-center justify-between rounded-none px-3 py-2 text-left transition-colors select-none focus-visible:outline-none {isSelected
                ? 'border-teal bg-teal/15 text-teal focus-visible:bg-teal/20 border-l-2 font-bold'
                : 'text-text-muted hover:bg-bg-btn hover:text-text-white focus-visible:bg-bg-btn focus-visible:text-text-white border-l-2 border-transparent'}"
            >
              <div class="flex min-w-0 items-center gap-2">
                {#if isSelected}
                  <span class="bg-teal size-1.5 shrink-0"></span>
                {:else}
                  <span class="size-1.5 shrink-0 opacity-0"></span>
                {/if}
                <span class="truncate uppercase">{i18n.t[REPORT_TAB_LABELS[tabId]]}</span>
              </div>

              {#if shortcut}
                <span
                  class="font-proto text-smaller ml-2 shrink-0 border px-1 py-0.5 transition-colors {isSelected
                    ? 'border-teal/40 text-teal'
                    : 'border-line/60 text-text-dim group-hover:text-text-muted'}"
                >
                  {shortcut}
                </span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</nav>

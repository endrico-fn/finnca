<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { todayString } from '$lib/core/format/date';
  import { DateRangeDropdown, DateDropdown } from '$lib/components/ui';
  import type { ReportTab, ReportGroup } from '../state/report.svelte';
  import { REPORT_GROUPS, REPORT_TAB_LABELS, reportGroupOf } from '../state/reportNav';

  let {
    currentTab,
    onSelectTab,
    from = $bindable(''),
    to = $bindable(''),
  }: {
    currentTab: ReportTab;
    onSelectTab: (tab: ReportTab) => void;
    from: string;
    to: string;
  } = $props();

  const activeGroup = $derived<ReportGroup>(reportGroupOf(currentTab));
  const currentGroupMeta = $derived(
    REPORT_GROUPS.find((g) => g.id === activeGroup) ?? REPORT_GROUPS[0]
  );

  const isRangeTab = $derived(
    currentTab === 'pnl' ||
      currentTab === 'cashflow' ||
      currentTab === 'spending' ||
      currentTab === 'fx' ||
      currentTab === 'income-exp' ||
      currentTab === 'trends' ||
      currentTab === 'networth'
  );
  const isAsOfTab = $derived(
    currentTab === 'bs' || currentTab === 'tb' || currentTab === 'debt' || currentTab === 'forecast'
  );
</script>

<div class="border-line mb-3 shrink-0 space-y-1.5 border-b pb-2.5 select-none">
  <div class="flex flex-wrap items-center justify-between gap-2.5">
    <div class="border-line/60 bg-bg-app inline-flex items-center gap-1 border p-0.5">
      {#each REPORT_GROUPS as g (g.id)}
        {@const isGroupActive = activeGroup === g.id}
        <button
          type="button"
          onclick={() => {
            if (!isGroupActive) {
              onSelectTab(g.defaultTab);
            }
          }}
          class="font-proto text-smaller flex h-7 cursor-pointer items-center gap-1.5 px-3 uppercase transition-colors {isGroupActive
            ? 'border-line/80 bg-bg-card text-text-strong font-bold border'
            : 'text-text-muted hover:text-text-strong hover:bg-bg-btn'}"
        >
          {#if isGroupActive}
            <span class="bg-teal size-1.5 shrink-0"></span>
          {/if}
          <span>{i18n.t[g.labelKey]}</span>
        </button>
      {/each}
    </div>

    {#if isRangeTab}
      <div class="flex shrink-0 items-center">
        <DateRangeDropdown bind:from bind:to />
      </div>
    {:else if isAsOfTab}
      <div class="flex h-7 shrink-0 items-center gap-2">
        <label
          for="as-of-date"
          class="font-proto text-text-dim text-smaller tracking-wider uppercase"
        >
          {i18n.t.asOfTodayLabel}:
        </label>
        <DateDropdown bind:value={to} />
        {#if to && to > todayString()}
          <span class="text-teal font-proto text-smaller animate-pulse font-bold">
            {i18n.t.forecastBadge}
          </span>
        {/if}
      </div>
    {/if}
  </div>

  <div class="flex items-center gap-1.5 overflow-x-auto pt-0.5">
    {#each currentGroupMeta.tabs as tabId (tabId)}
      {@const isSelected = currentTab === tabId}
      <button
        type="button"
        onclick={() => onSelectTab(tabId)}
        class="font-proto text-smaller flex h-7 cursor-pointer items-center gap-1.5 px-2.5 uppercase transition-colors whitespace-nowrap {isSelected
          ? 'bg-teal/15 text-teal border-teal/50 font-bold border'
          : 'border-line/40 bg-bg-card/50 text-text-muted hover:text-text-strong hover:bg-bg-btn hover:border-line border'}"
      >
        <span>{i18n.t[REPORT_TAB_LABELS[tabId]]}</span>
      </button>
    {/each}
  </div>
</div>

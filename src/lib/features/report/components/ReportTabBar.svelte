<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { todayString } from '$lib/core/format/date';
  import { Tabs, DateRangeDropdown, DateDropdown } from '$lib/components/ui';

  export type ReportTab = 'tb' | 'trends' | 'debt' | 'cashflow' | 'fx';

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
</script>

<div class="border-line mb-2 flex shrink-0 items-center justify-between gap-2 border-b pb-2">
  <div class="flex min-w-0 items-center gap-1">
    <Tabs
      tabs={[
        { id: 'tb', label: i18n.t.trialBalanceTitle },
        { id: 'trends', label: i18n.t.trendsTitle },
        { id: 'debt', label: i18n.t.debtReportTitle },
        { id: 'cashflow', label: i18n.t.cashflowTab },
        { id: 'fx', label: i18n.t.fxRevalTab },
      ]}
      active={currentTab}
      onSelect={(id) => onSelectTab(id as ReportTab)}
    />
  </div>

  {#if currentTab === 'cashflow' || currentTab === 'fx'}
    <div class="flex shrink-0 items-center">
      <DateRangeDropdown bind:from bind:to />
    </div>
  {:else if currentTab !== 'trends'}
    <div class="flex h-6 shrink-0 items-center gap-2">
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

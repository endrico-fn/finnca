<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { todayString } from '$lib/core/format/date';
  import { Tabs, DateRangeDropdown, DateDropdown } from '$lib/components/ui';
  import type { ReportTab, ReportGroup } from '../state/report.svelte';

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

  const groups: { id: ReportGroup; label: string; defaultTab: ReportTab }[] = $derived([
    { id: 'statements', label: i18n.t.reportGroupStatements, defaultTab: 'bs' },
    { id: 'budget-debt', label: i18n.t.reportGroupBudgetDebt, defaultTab: 'budget-actual' },
    { id: 'analysis', label: i18n.t.reportGroupAnalysis, defaultTab: 'trends' },
  ]);

  const activeGroup = $derived<ReportGroup>(
    currentTab === 'bs' || currentTab === 'pnl' || currentTab === 'cashflow' || currentTab === 'tb'
      ? 'statements'
      : currentTab === 'budget-actual' || currentTab === 'debt'
        ? 'budget-debt'
        : 'analysis'
  );

  const currentGroupTabs = $derived.by(() => {
    switch (activeGroup) {
      case 'statements':
        return [
          { id: 'bs', label: i18n.t.balanceSheetTitle },
          { id: 'pnl', label: i18n.t.pnlTitle },
          { id: 'cashflow', label: i18n.t.cashflowTab },
          { id: 'tb', label: i18n.t.trialBalanceTitle },
        ];
      case 'budget-debt':
        return [
          { id: 'budget-actual', label: i18n.t.budgetVsActualTitle },
          { id: 'debt', label: i18n.t.debtReportTitle },
        ];
      case 'analysis':
        return [
          { id: 'trends', label: i18n.t.trendsTitle },
          { id: 'fx', label: i18n.t.fxRevalTab },
        ];
    }
  });

  const isRangeTab = $derived(
    currentTab === 'pnl' || currentTab === 'cashflow' || currentTab === 'fx'
  );
  const isAsOfTab = $derived(
    currentTab === 'bs' || currentTab === 'tb' || currentTab === 'debt'
  );

  function onSelectGroup(groupId: ReportGroup) {
    const grp = groups.find((g) => g.id === groupId);
    if (grp && activeGroup !== groupId) {
      onSelectTab(grp.defaultTab);
    }
  }
</script>

<div class="border-line mb-3 flex flex-col gap-2 border-b pb-2">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div class="flex items-center gap-2">
      {#each groups as g (g.id)}
        {@const isGroupActive = activeGroup === g.id}
        <button
          type="button"
          onclick={() => onSelectGroup(g.id)}
          class="font-proto text-smaller px-2.5 py-1 tracking-wider uppercase transition-colors {isGroupActive
            ? 'border-b-2 border-teal text-teal font-bold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {g.label}
        </button>
      {/each}
    </div>

    {#if isRangeTab}
      <div class="flex shrink-0 items-center">
        <DateRangeDropdown bind:from bind:to />
      </div>
    {:else if isAsOfTab}
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

  <div class="border-line/40 flex flex-wrap items-center gap-1 border-t pt-1.5">
    <Tabs
      tabs={currentGroupTabs}
      active={currentTab}
      onSelect={(id) => onSelectTab(id as ReportTab)}
    />
  </div>
</div>


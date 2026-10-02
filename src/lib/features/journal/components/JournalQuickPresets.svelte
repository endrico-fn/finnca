<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { FilterMenu, FilterSection, FilterOption } from '$lib/components/ui';
  import { journalState } from '../state/journalDraft.svelte';
  import { buildRecentPresets, type PresetInput } from '../state/journalFormUtils';
  import { fromMinor, formatMinorGrouping } from '$lib/core/format/currency';

  let {
    onSelect,
    align = 'left',
    class: className = '',
  }: {
    onSelect: (preset: PresetInput) => void;
    align?: 'left' | 'right';
    class?: string;
  } = $props();

  const staticPresets: PresetInput[] = [
    {
      desc: i18n.t.txPresetDining,
      expenseKeywords: ['food', 'makan', 'dining', 'resto', 'cafe'],
      expenseCodes: ['5110', '5100'],
    },
    {
      desc: i18n.t.txPresetGroceries,
      expenseKeywords: ['grocer', 'belanja', 'supermarket', 'pasar'],
      expenseCodes: ['5120', '5200'],
    },
    {
      desc: i18n.t.txPresetTransport,
      expenseKeywords: ['transport', 'bensin', 'fuel', 'tol', 'gas'],
      expenseCodes: ['5210', '5300'],
    },
    {
      desc: i18n.t.txPresetBills,
      expenseKeywords: ['util', 'tagihan', 'bill', 'listrik', 'water', 'internet'],
      expenseCodes: ['5310', '5400'],
    },
    {
      desc: i18n.t.txPresetOpening,
      expenseKeywords: [],
      equityCodes: ['3110'],
    },
  ];

  const recents = $derived(buildRecentPresets(journalState.entries));

  function selectRecent(r: (typeof recents)[number]) {
    onSelect({
      desc: r.desc,
      expenseKeywords: [],
      fromAccountId: r.fromAccountId,
      toAccountId: r.toAccountId,
      amountMajor: String(fromMinor(r.currency, r.amountMinor)),
      currency: r.currency,
    });
  }
</script>

<div class="flex items-center {className}">
  <FilterMenu label={i18n.t.quickPresets} {align} panelClass="min-w-64">
    <FilterSection title={i18n.t.presetStaticTitle} layout="grid">
      {#each staticPresets as p (p.desc)}
        <FilterOption label={p.desc} selected={false} check={false} onclick={() => onSelect(p)} />
      {/each}
    </FilterSection>
    {#if recents.length > 0}
      <FilterSection title={i18n.t.presetRecentTitle} layout="list">
        {#each recents as r (r.desc.toLowerCase())}
          <FilterOption
            label={r.desc}
            count={r.count}
            title="{formatMinorGrouping(r.amountMinor, r.currency)} • {r.lastDate}"
            check={false}
            onclick={() => selectRecent(r)}
          />
        {/each}
      </FilterSection>
    {/if}
  </FilterMenu>
</div>

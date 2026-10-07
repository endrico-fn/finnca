<script lang="ts">
  import { ModalShell, Button, FilterSection, FilterOption } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';

  let {
    open = $bindable(false),
    reportTitle,
    supportsPDF = true,
    onExport,
  }: {
    open: boolean;
    reportTitle: string;
    supportsPDF?: boolean;
    onExport: (format: 'pdf' | 'csv', from: string, to: string) => Promise<void>;
  } = $props();

  let format = $state<'pdf' | 'csv'>('pdf');
  let periodMode = $state<'this_month' | 'last_month' | 'this_year' | 'all' | 'custom'>(
    'this_month'
  );
  let customFrom = $state('');
  let customTo = $state('');
  let busy = $state(false);

  $effect(() => {
    if (!supportsPDF && format === 'pdf') format = 'csv';
  });

  const resolvedRange = $derived.by(() => {
    const now = new Date();
    const y = now.getFullYear();
    const m = now.getMonth();

    if (periodMode === 'this_month') {
      const mStr = String(m + 1).padStart(2, '0');
      const last = new Date(y, m + 1, 0).getDate();
      return { from: `${y}-${mStr}-01`, to: `${y}-${mStr}-${String(last).padStart(2, '0')}` };
    }
    if (periodMode === 'last_month') {
      const prevY = m === 0 ? y - 1 : y;
      const prevM = m === 0 ? 12 : m;
      const prevMStr = String(prevM).padStart(2, '0');
      const last = new Date(prevY, prevM, 0).getDate();
      return {
        from: `${prevY}-${prevMStr}-01`,
        to: `${prevY}-${prevMStr}-${String(last).padStart(2, '0')}`,
      };
    }
    if (periodMode === 'this_year') {
      return { from: `${y}-01-01`, to: `${y}-12-31` };
    }
    if (periodMode === 'all') {
      return { from: '', to: '' };
    }
    return { from: customFrom, to: customTo };
  });

  const canExport = $derived(
    periodMode !== 'custom' || (customFrom !== '' && customTo !== '' && customFrom <= customTo)
  );

  async function handleExport() {
    if (!canExport) return;
    busy = true;
    try {
      await onExport(format, resolvedRange.from, resolvedRange.to);
      open = false;
    } finally {
      busy = false;
    }
  }

  const periods: { id: typeof periodMode; label: string }[] = [
    { id: 'this_month', label: i18n.t.thisMonth },
    { id: 'last_month', label: i18n.t.lastMonth },
    { id: 'this_year', label: i18n.t.thisYear },
    { id: 'all', label: i18n.t.allTime },
    { id: 'custom', label: i18n.t.exportCustomRange },
  ];
</script>

<ModalShell bind:open title="{i18n.t.exportReportTitle} — {reportTitle}" size="xs">
  <div class="space-y-4 pt-2 pb-1">
    {#if supportsPDF}
      <div>
        <p class="text-text-dim text-smaller mb-2 tracking-widest uppercase">
          {i18n.t.exportFormat}
        </p>
        <div class="flex gap-2">
          <Button
            variant={format === 'pdf' ? 'primary' : 'ghost'}
            size="sm"
            onclick={() => (format = 'pdf')}
            class="flex-1"
          >
            PDF
          </Button>
          <Button
            variant={format === 'csv' ? 'primary' : 'ghost'}
            size="sm"
            onclick={() => (format = 'csv')}
            class="flex-1"
          >
            CSV
          </Button>
        </div>
      </div>
    {/if}

    <div>
      <FilterSection title={i18n.t.exportPeriod}>
        {#each periods as opt (opt.id)}
          <FilterOption
            label={opt.label}
            selected={periodMode === opt.id}
            onclick={() => (periodMode = opt.id)}
          />
        {/each}
      </FilterSection>

      {#if periodMode === 'custom'}
        <div class="border-line mt-3 space-y-2 border-t pt-3">
          <div class="flex items-center gap-2">
            <span class="text-text-dim text-smaller w-6 shrink-0 tracking-wider uppercase"
              >{i18n.t.pickFrom}</span
            >
            <input
              type="date"
              bind:value={customFrom}
              class="sharp-input font-proto text-small flex-1 px-2 py-1 tabular-nums"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="text-text-dim text-smaller w-6 shrink-0 tracking-wider uppercase"
              >{i18n.t.pickTo}</span
            >
            <input
              type="date"
              bind:value={customTo}
              min={customFrom}
              class="sharp-input font-proto text-small flex-1 px-2 py-1 tabular-nums"
            />
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div class="border-line mt-3 flex shrink-0 gap-2 border-t pt-3">
    <Button variant="ghost" onclick={() => (open = false)} disabled={busy} class="flex-1">
      {i18n.t.cancelBtn}
    </Button>
    <Button variant="primary" onclick={handleExport} disabled={busy || !canExport} class="flex-1">
      {busy ? i18n.t.planProcessing : `${i18n.t.exportReportBtn} →`}
    </Button>
  </div>
</ModalShell>

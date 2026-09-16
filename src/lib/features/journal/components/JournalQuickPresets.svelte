<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { Button } from '$lib/components/ui';

  let {
    onSelect,
  }: {
    onSelect: (preset: { desc: string; expenseKeyword: string; assetKeyword?: string }) => void;
  } = $props();
</script>

<div class="font-proto text-smaller mt-2 flex items-center gap-1.5 overflow-x-auto pb-1">
  <span class="text-text-dim mr-1 shrink-0 tracking-wider uppercase">
    {i18n.t.quickPresets}:
  </span>
  <Button
    variant="ghost"
    size="sm"
    onclick={() => onSelect({ desc: 'Makan Siang / Minum', expenseKeyword: 'makan' })}
    class="shrink-0"
  >
    {i18n.t.txPresetDining}
  </Button>
  <Button
    variant="ghost"
    size="sm"
    onclick={() => onSelect({ desc: 'Belanja Bulanan & Supermarket', expenseKeyword: 'belanja' })}
    class="shrink-0"
  >
    {i18n.t.txPresetGroceries}
  </Button>
  <Button
    variant="ghost"
    size="sm"
    onclick={() => onSelect({ desc: 'Transport / Bensin / Tol', expenseKeyword: 'transport' })}
    class="shrink-0"
  >
    {i18n.t.txPresetTransport}
  </Button>
  <Button
    variant="ghost"
    size="sm"
    onclick={() =>
      onSelect({ desc: 'Tagihan Listrik / Air / Internet', expenseKeyword: 'tagihan' })}
    class="shrink-0"
  >
    {i18n.t.txPresetBills}
  </Button>
</div>

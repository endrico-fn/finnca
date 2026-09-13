<script lang="ts">
  import { i18n } from '$lib/i18n.svelte';
  import Button from './Button.svelte';
  import Icon from './Icon.svelte';

  let {
    currentPage = $bindable(),
    totalPages,
    totalItems = undefined,
    itemLabel = '',
    class: extraClass = '',
    layout = 'grouped',
  } = $props<{
    currentPage: number;
    totalPages: number;
    totalItems?: number;
    itemLabel?: string;
    class?: string;
    layout?: 'grouped' | 'spread';
  }>();

  const resolvedItemLabel = $derived(itemLabel || i18n.t.paginationDefaultUnit);

  function prev() {
    currentPage = Math.max(1, currentPage - 1);
  }

  function next() {
    currentPage = Math.min(totalPages, currentPage + 1);
  }
</script>

<div
  class={layout === 'spread'
    ? `flex w-full items-center justify-between ${extraClass}`
    : `flex items-center gap-2 ${extraClass}`}
>
  <Button variant="pager" onclick={prev} disabled={currentPage <= 1} ariaLabel={i18n.t.prev}>
    <Icon name="chev-left" size={14} />
    {i18n.t.prev}
  </Button>

  <span class="font-proto text-text-dim text-smaller px-1 tabular-nums">
    {currentPage} / {totalPages}{#if totalItems !== undefined}
      &bull; {totalItems} {resolvedItemLabel}{/if}
  </span>

  <Button
    variant="pager"
    onclick={next}
    disabled={currentPage >= totalPages}
    ariaLabel={i18n.t.next}
  >
    {i18n.t.next}
    <Icon name="chev-right" size={14} />
  </Button>
</div>

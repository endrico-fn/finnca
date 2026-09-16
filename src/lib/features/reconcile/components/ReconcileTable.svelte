<script lang="ts">
  import { reconcileState } from '../state/reconcile.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Button, Icon, EmptyState } from '$lib/components/ui';

  let {
    currency = 'IDR',
  }: {
    currency?: string;
  } = $props();
</script>

<div class="sharp-card flex flex-1 flex-col overflow-y-auto">
  {#if !reconcileState.selectedAccountId}
    <div class="flex h-full items-center justify-center p-8">
      <EmptyState
        title={i18n.t.reconcileEmptyTitle}
        hint={i18n.t.reconcileEmptyHint}
        icon="check"
      />
    </div>
  {:else}
    <div
      class="border-line flex shrink-0 items-center justify-between gap-2 border-b px-3 py-2.5"
    >
      <div class="font-proto text-smaller flex items-center gap-2">
        <span class="text-text-muted">{i18n.t.reconcileStatus}:</span>
        <span class="text-text-strong font-bold">{reconcileState.clearedCount}</span>
        <span class="text-text-muted">
          / {reconcileState.unclearedPostings.length} {i18n.t.clearedStatus}
        </span>
      </div>
      <div class="flex items-center gap-1.5">
        <Button
          variant="ghost"
          size="sm"
          onclick={() => reconcileState.selectAll()}
          class="font-proto text-smaller h-6 px-2"
        >
          {i18n.t.reconcileSelectAll}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onclick={() => reconcileState.clearAll()}
          class="font-proto text-smaller h-6 px-2"
        >
          {i18n.t.reconcileDeselectAll}
        </Button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto">
      <div
        class="border-line bg-line/20 font-proto text-text-muted text-smaller sticky top-0 z-10 grid grid-cols-12 gap-2 border-b px-3 py-1.5 tracking-widest uppercase"
      >
        <div class="col-span-1 text-center">{i18n.t.reconcileColClr}</div>
        <div class="col-span-2">{i18n.t.reconcileColDate}</div>
        <div class="col-span-6">{i18n.t.reconcileColDesc}</div>
        <div class="col-span-3 text-right">{i18n.t.reconcileColAmount}</div>
      </div>

      {#if reconcileState.unclearedPostings.length === 0}
        <div class="text-text-muted font-proto text-smaller p-6 text-center">
          {i18n.t.reconcileNoUncleared}
        </div>
      {/if}

      {#each reconcileState.unclearedPostings as posting (posting.posting_id)}
        {@const isCleared = reconcileState.clearedMap.get(posting.posting_id) ?? false}
        <button
          type="button"
          onclick={() => reconcileState.toggleCleared(posting.posting_id)}
          class="border-line/40 font-proto hover:bg-bg-btn text-smaller grid w-full grid-cols-12 items-center gap-2 border-b px-3 py-1.5 text-left transition-colors"
        >
          <div class="col-span-1 flex justify-center">
            <div
              class="flex h-3.5 w-3.5 items-center justify-center border transition-colors {isCleared
                ? 'bg-teal border-teal text-bg-app'
                : 'border-line hover:border-text-muted bg-transparent'}"
            >
              {#if isCleared}
                <Icon name="check" size={9} />
              {/if}
            </div>
          </div>
          <div class="text-text-dim font-proto col-span-2 tabular-nums">
            {posting.date}
          </div>
          <div class="text-text-strong col-span-6 truncate">
            {posting.description}
            {#if posting.memo}
              <span class="text-text-muted text-smaller">({posting.memo})</span>
            {/if}
          </div>
          <div
            class="font-proto col-span-3 text-right font-bold tabular-nums {posting.amount > 0
              ? 'text-income'
              : 'text-expense'}"
          >
            {formatMinorToDisplay(posting.amount, currency)}
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

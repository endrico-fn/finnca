<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import { ACCOUNT_TYPE_COLOR } from '../state/accounts.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Badge } from '$lib/components/ui';

  export interface AccountRowItem extends Account {
    fullPath: string;
    balance: number;
  }

  let {
    accounts = [],
    selectedId = null,
    onSelect,
    onDblClick,
    onToggleHide,
  }: {
    accounts?: AccountRowItem[];
    selectedId?: string | null;
    onSelect?: (acc: AccountRowItem) => void;
    onDblClick?: (acc: AccountRowItem) => void;
    onToggleHide?: (acc: AccountRowItem, e: MouseEvent) => void;
  } = $props();
</script>

<table class="w-full border-collapse">
  <thead class="bg-bg-card sticky top-0 z-10">
    <tr class="border-line text-text-base border-b">
      <th class="label-xs w-16 py-1.5 pl-3 text-left font-normal whitespace-nowrap">
        {i18n.t.colCode}
      </th>
      <th class="label-xs px-3 py-1.5 text-left font-normal">
        {i18n.t.colHierarchyPath}
      </th>
      <th class="label-xs w-28 px-3 py-1.5 text-right font-normal whitespace-nowrap">
        {i18n.t.colBalance}
      </th>
      <th class="label-xs w-20 py-1.5 pr-3 text-right font-normal"></th>
    </tr>
  </thead>
  <tbody class="divide-line/40 divide-y">
    {#each accounts as acc (acc.id)}
      {@const isSelected = selectedId === acc.id}
      <tr
        class="cursor-pointer transition-colors {isSelected
          ? 'bg-bg-row-active'
          : 'hover:bg-bg-row-active'} {acc.hidden ? 'opacity-40' : ''}"
        onclick={() => onSelect?.(acc)}
        ondblclick={() => onDblClick?.(acc)}
        title={acc.placeholder ? i18n.t.singleSelectDoubleEdit : i18n.t.singleSelectDoubleLedger}
      >
        <td class="text-text-muted font-proto text-smaller py-1 pl-3 whitespace-nowrap">
          {acc.code}
        </td>
        <td class="text-text-strong w-full max-w-0 truncate px-3 py-1">
          <div class="flex items-center gap-2 truncate">
            <span
              class="size-1.5 shrink-0"
              style="background:{ACCOUNT_TYPE_COLOR[acc.account_type]}"
            ></span>
            <span class="font-proto text-smaller truncate">{acc.fullPath}</span>
            {#if acc.placeholder}
              <Badge size="s" tone="neutral" class="shrink-0">{i18n.t.badgePh}</Badge>
            {/if}
            {#if acc.hidden}
              <Badge size="s" tone="neutral" class="shrink-0">{i18n.t.badgeHidden}</Badge>
            {/if}
          </div>
        </td>
        <td
          class="font-proto text-smaller px-3 py-1 text-right whitespace-nowrap tabular-nums {acc.balance <
          0
            ? 'text-expense'
            : 'text-text-base'}"
        >
          {formatMinorToDisplay(Math.abs(acc.balance), acc.currency)}
        </td>
        <td class="py-1 pr-3 text-right whitespace-nowrap">
          <div class="flex items-center justify-end">
            <button
              type="button"
              onclick={(e) => onToggleHide?.(acc, e)}
              title={acc.hidden ? i18n.t.showAccount : i18n.t.hideAccount}
              aria-label={acc.hidden ? i18n.t.showAccount : i18n.t.hideAccount}
              class="text-text-muted hover:text-text-base text-smaller px-1 py-0.5 transition-colors"
            >
              {acc.hidden ? '◉' : '◎'}
            </button>
          </div>
        </td>
      </tr>
    {/each}
  </tbody>
</table>

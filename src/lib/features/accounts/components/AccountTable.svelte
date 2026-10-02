<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import { ACCOUNT_TYPE_BG } from '../state/accounts.svelte';
  import { isDebitNormal } from '../state/accountLedgerUtils';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Badge, Icon, AccountHoverCard } from '$lib/components/ui';

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

<table class="sharp-table w-full border-x-0 border-t-0" spellcheck="false">
  <thead class="bg-bg-card sticky top-0 z-10">
    <tr class="border-line border-b">
      <th class="w-24 py-2 pl-3 whitespace-nowrap">{i18n.t.colCode}</th>
      <th class="px-3 py-2">{i18n.t.colHierarchyPath}</th>
      <th class="numeric w-36 px-3 py-2 text-right whitespace-nowrap">{i18n.t.colBalance}</th>
      <th class="center w-12 py-2 pr-3"></th>
    </tr>
  </thead>
  <tbody>
    {#each accounts as acc (acc.id)}
      {@const isSelected = selectedId === acc.id}
      {@const segments = acc.fullPath.split(' > ')}
      {@const isDebit = isDebitNormal(acc.account_type)}
      {@const normalBalance = isDebit ? acc.balance : -acc.balance}
      <tr
        id="acc-row-{acc.id}"
        class="hover:bg-bg-row-active cursor-pointer transition-colors {isSelected
          ? 'selected bg-bg-row-active border-teal border-l-2'
          : ''} {acc.hidden ? 'opacity-40' : ''}"
        aria-selected={isSelected}
        onclick={() => onSelect?.(acc)}
        ondblclick={() => onDblClick?.(acc)}
        title={acc.placeholder ? i18n.t.singleSelectDoubleEdit : i18n.t.singleSelectDoubleLedger}
      >
        <td
          class="text-text-muted font-proto text-smaller w-24 py-2 pl-3 whitespace-nowrap tabular-nums"
        >
          {acc.code}
        </td>
        <td class="max-w-0 px-3 py-2">
          <AccountHoverCard
            account={acc}
            directBalance={acc.balance}
            rollupBalance={acc.balance}
            currency={acc.currency}
            class="w-full"
          >
            <div class="flex min-w-0 items-center gap-2">
              <span class="size-1.5 shrink-0 {ACCOUNT_TYPE_BG[acc.account_type]}"></span>
              <div class="flex min-w-0 items-center gap-1 truncate">
                {#if segments.length > 1}
                  <span
                    class="text-text-dim font-aux text-smaller min-w-0 shrink truncate normal-case"
                  >
                    {segments.slice(0, -1).join(' > ')}
                    <span class="text-text-muted/50 font-proto text-smaller mx-0.5 select-none"
                      >&gt;</span
                    >
                  </span>
                {/if}
                <span class="text-text-strong font-aux text-small truncate font-medium normal-case">
                  {segments[segments.length - 1]}
                </span>
              </div>
              {#if acc.placeholder}
                <Badge size="s" tone="neutral" class="shrink-0">{i18n.t.badgePh}</Badge>
              {/if}
              {#if acc.hidden}
                <Badge size="s" tone="neutral" class="shrink-0">{i18n.t.badgeHidden}</Badge>
              {/if}
            </div>
          </AccountHoverCard>
        </td>
        <td
          class="numeric font-proto text-smaller w-36 px-3 py-2 text-right whitespace-nowrap tabular-nums {normalBalance <
          0
            ? 'text-expense font-bold'
            : 'text-text-strong font-medium'}"
        >
          {formatMinorToDisplay(normalBalance, acc.currency)}
        </td>
        <td class="center w-12 py-2 pr-3 whitespace-nowrap">
          <button
            type="button"
            onclick={(e) => onToggleHide?.(acc, e)}
            title={acc.hidden ? i18n.t.showAccount : i18n.t.hideAccount}
            aria-label={acc.hidden ? i18n.t.showAccount : i18n.t.hideAccount}
            class="text-text-muted hover:text-text-base text-smaller inline-flex items-center justify-center p-1 transition-colors"
          >
            <Icon name={acc.hidden ? 'eye-off' : 'eye'} size={13} />
          </button>
        </td>
      </tr>
    {/each}
  </tbody>
</table>

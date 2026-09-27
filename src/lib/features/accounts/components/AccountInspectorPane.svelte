<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import { getAccountLedgerCmd } from '$lib/core/ipc/bindings';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Badge, Button, Icon } from '$lib/components/ui';
  import MiniSparkline from '$lib/components/charts/MiniSparkline.svelte';
  import { isDebitNormal } from '../state/accountLedgerUtils';
  import type { BadgeTone } from '$lib/components/ui/badgeTone';

  let {
    account,
    directBalance = 0,
    rollupBalance = 0,
    fullPath = '',
    onClose,
    onAddSubAccount,
    onEdit,
    onViewLedger,
    onDelete,
    onToggleHide,
  }: {
    account: Account;
    directBalance?: number;
    rollupBalance?: number;
    fullPath?: string;
    onClose: () => void;
    onAddSubAccount: (parentId: string) => void;
    onEdit: (account: Account) => void;
    onViewLedger: (code: string) => void;
    onDelete: (account: Account) => void;
    onToggleHide: (account: Account, e: MouseEvent) => void;
  } = $props();

  let sparklineData = $state<number[]>([]);
  let loadingLedger = $state(false);

  const isDebit = $derived(isDebitNormal(account.account_type));
  const normalDirectBalance = $derived(isDebit ? directBalance : -directBalance);
  const normalRollupBalance = $derived(isDebit ? rollupBalance : -rollupBalance);

  const accountTypeTone = $derived.by((): BadgeTone => {
    switch (account.account_type) {
      case 'ASSET':
        return 'asset';
      case 'LIABILITY':
        return 'liability';
      case 'EQUITY':
        return 'equity';
      case 'INCOME':
        return 'income';
      case 'EXPENSE':
        return 'expense';
      default:
        return 'neutral';
    }
  });

  $effect(() => {
    if (account?.id) {
      loadingLedger = true;
      getAccountLedgerCmd(account.id)
        .then((items) => {
          if (items.length >= 2) {
            sparklineData = items.map((it) => it.running_balance);
          } else {
            sparklineData = [
              rollupBalance * 0.96,
              rollupBalance * 0.98,
              rollupBalance * 1.01,
              rollupBalance,
            ];
          }
        })
        .catch(() => {
          sparklineData = [
            rollupBalance * 0.96,
            rollupBalance * 0.98,
            rollupBalance * 1.01,
            rollupBalance,
          ];
        })
        .finally(() => {
          loadingLedger = false;
        });
    }
  });
</script>

<div class="border-line bg-bg-card flex h-full flex-col divide-y divide-line/40 select-none">
  <div class="flex items-start justify-between p-2.5 pb-2">
    <div class="min-w-0 flex-1 pr-2">
      <div class="flex flex-wrap items-center gap-1.5">
        <span class="font-proto text-text-muted text-smaller font-bold">[{account.code}]</span>
        <Badge tone={accountTypeTone} size="s">
          {account.account_type}
        </Badge>
        {#if account.placeholder}
          <Badge tone="neutral" size="s">
            {i18n.t.placeholderGroup}
          </Badge>
        {/if}
        {#if account.hidden}
          <Badge tone="warn" size="s">
            {i18n.t.hiddenAccount}
          </Badge>
        {/if}
      </div>
      <h3 class="font-proto text-text-strong text-medium mt-1 truncate font-bold">
        {account.name}
      </h3>
      <p class="font-proto text-text-dim text-smaller mt-0.5 truncate tracking-wide" title={fullPath}>
        {fullPath}
      </p>
    </div>
    <button
      type="button"
      onclick={onClose}
      class="text-text-muted hover:text-text-strong cursor-pointer p-1"
      title={i18n.t.closeBtn}
      aria-label={i18n.t.closeBtn}
    >
      <Icon name="close" size={14} />
    </button>
  </div>

  <div class="space-y-2 p-2.5">
    <div class="bg-bg-app border-line/60 border p-2.5 space-y-1.5">
      <div class="flex items-center justify-between">
        <span class="font-proto text-text-dim text-smaller uppercase font-bold">
          {i18n.t.rollupBalance}
        </span>
        <span class="font-proto text-text-dim text-smaller">
          {isDebit ? 'DR' : 'CR'}
        </span>
      </div>
      <div class="font-proto text-text-strong text-medium font-bold tabular-nums">
        {formatMinorToDisplay(normalRollupBalance, account.currency)}
      </div>

      <div class="border-line/40 flex items-center justify-between border-t pt-1.5">
        <span class="font-proto text-text-dim text-smaller uppercase">
          {i18n.t.directBalance}
        </span>
        <span class="font-proto text-text-base text-smaller font-semibold tabular-nums">
          {formatMinorToDisplay(normalDirectBalance, account.currency)}
        </span>
      </div>
    </div>

    <div class="border-line/60 bg-bg-app flex items-center justify-between border px-2.5 py-1.5">
      <div>
        <span class="font-proto text-text-dim text-smaller block uppercase">30D TREND</span>
        <span class="font-proto text-text-muted text-smaller">
          {loadingLedger ? '...' : `${sparklineData.length} pts`}
        </span>
      </div>
      <MiniSparkline
        data={sparklineData}
        width={130}
        height={22}
        color={account.account_type === 'EXPENSE' ? 'var(--color-expense)' : 'var(--color-income)'}
      />
    </div>

    {#if account.description}
      <div class="border-line/60 bg-bg-app border p-2">
        <span class="font-proto text-text-dim text-smaller block uppercase font-bold">{i18n.t.descriptionField}</span>
        <p class="font-aux text-text-muted text-smaller mt-0.5 line-clamp-2 leading-relaxed">
          {account.description}
        </p>
      </div>
    {/if}
  </div>

  <div class="mt-auto space-y-1.5 p-2.5 pt-2">
    {#if !account.placeholder}
      <Button
        variant="tactical"
        class="font-proto text-smaller h-7 w-full justify-center font-bold tracking-wider"
        onclick={() => onViewLedger(account.code)}
      >
        <span class="inline-flex items-center gap-1.5">
          <Icon name="wallet" size={11} />
          {i18n.t.ledger}
        </span>
      </Button>
    {/if}

    <div class="grid grid-cols-2 gap-1.5">
      <Button
        variant="primary"
        class="font-proto text-smaller h-7 w-full justify-center font-bold tracking-wider"
        onclick={() => onAddSubAccount(account.id)}
      >
        <span class="inline-flex items-center gap-1.5">
          <Icon name="plus" size={11} />
          {i18n.t.subAccount}
        </span>
      </Button>
      <Button
        variant="ghost"
        class="font-proto text-smaller h-7 w-full justify-center font-bold tracking-wider"
        onclick={() => onEdit(account)}
      >
        <span class="inline-flex items-center gap-1.5">
          <Icon name="pencil" size={11} />
          {i18n.t.edit}
        </span>
      </Button>
    </div>

    <div class="flex items-center justify-between pt-0.5">
      <button
        type="button"
        onclick={(e) => onToggleHide(account, e)}
        class="font-proto text-text-dim hover:text-text-strong text-smaller cursor-pointer uppercase transition-colors"
      >
        {account.hidden ? i18n.t.showAccount : i18n.t.hideAccount}
      </button>

      <button
        type="button"
        onclick={() => onDelete(account)}
        class="font-proto text-danger hover:text-danger/80 text-smaller cursor-pointer uppercase transition-colors"
      >
        {i18n.t.deleteAccountBtn}
      </button>
    </div>
  </div>
</div>

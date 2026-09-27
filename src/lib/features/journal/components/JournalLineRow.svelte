<script lang="ts">
  import type { PostingInput, Account } from '$lib/core/ipc/bindings';
  import { fromMinor } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Icon, Button, AccountSelectDropdown } from '$lib/components/ui';

  let {
    split = $bindable(),
    currency = 'IDR',
    accounts,
    onAmountChange,
    onRemove,
  }: {
    split: PostingInput;
    currency?: string;
    accounts: Account[];
    onAmountChange: (split: PostingInput, field: 'debit' | 'credit', val: string) => void;
    onRemove: (id?: string | null) => void;
  } = $props();

  const debitVal = $derived(split.amount > 0 ? String(fromMinor(currency, split.amount)) : '');
  const creditVal = $derived(split.amount < 0 ? String(fromMinor(currency, -split.amount)) : '');
</script>

<tr class="hover:bg-bg-row-active">
  <td class="min-w-60 px-2 py-1.5">
    <AccountSelectDropdown
      bind:value={split.account_id}
      {accounts}
      placeholder={i18n.t.txSelectAccount}
      size="sm"
      class="w-full"
    />
  </td>
  <td class="px-2 py-1.5">
    <input
      value={debitVal}
      oninput={(e) => onAmountChange(split, 'debit', (e.target as HTMLInputElement).value)}
      placeholder={i18n.t.commonZeroPlaceholder}
      inputmode="decimal"
      autocomplete="off"
      class="sharp-input text-income font-proto w-full text-right"
    />
  </td>
  <td class="px-2 py-1.5">
    <input
      value={creditVal}
      oninput={(e) => onAmountChange(split, 'credit', (e.target as HTMLInputElement).value)}
      placeholder={i18n.t.commonZeroPlaceholder}
      inputmode="decimal"
      autocomplete="off"
      class="sharp-input text-text-base font-proto w-full text-right"
    />
  </td>
  <td class="px-2 py-1.5">
    <input
      bind:value={split.memo}
      placeholder={i18n.t.txSplitMemoPlaceholder}
      class="sharp-input font-aux text-small w-full"
    />
  </td>
  <td class="px-1 py-1.5 text-center">
    <Button
      variant="pager"
      size="icon"
      onclick={() => onRemove(split.id)}
      title={i18n.t.txDeleteSplit}
      ariaLabel={i18n.t.txDeleteSplit}
    >
      <Icon name="close" size={12} />
    </Button>
  </td>
</tr>

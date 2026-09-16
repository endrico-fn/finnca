<script lang="ts">
  import type { Split, Currency } from '../state/journalDraft.svelte';
  import type { Account } from '$lib/core/ipc/bindings';
  import { fromMinor } from '$lib/core/format/currency';
  import { accountTypeLabel } from '$lib/core/format/account';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Icon, Button, SelectDropdown } from '$lib/components/ui';

  let {
    split = $bindable(),
    currency,
    accounts,
    onAmountChange,
    onRemove,
  }: {
    split: Split;
    currency: Currency;
    accounts: Account[];
    onAmountChange: (split: Split, field: 'debit' | 'credit', val: string) => void;
    onRemove: (id: string) => void;
  } = $props();

  const debitVal = $derived(split.amount > 0 ? String(fromMinor(currency, split.amount)) : '');
  const creditVal = $derived(split.amount < 0 ? String(fromMinor(currency, -split.amount)) : '');

  const accountOptions = $derived(
    accounts.map((a) => ({
      value: a.id,
      label: `${a.code} — ${a.name}`,
      sublabel: `[${accountTypeLabel(a.account_type)}]`,
    }))
  );
</script>

<tr class="hover:bg-bg-row-active">
  <td class="px-2 py-1.5">
    <SelectDropdown
      bind:value={split.accountId}
      searchable
      placeholder={i18n.t.txSelectAccount}
      options={accountOptions}
      class="w-full"
      menuClass="w-max"
    />
  </td>
  <td class="px-2 py-1.5">
    <input
      value={debitVal}
      oninput={(e) => onAmountChange(split, 'debit', (e.target as HTMLInputElement).value)}
      placeholder={i18n.t.commonZeroPlaceholder}
      class="sharp-input text-income font-proto w-full text-right"
    />
  </td>
  <td class="px-2 py-1.5">
    <input
      value={creditVal}
      oninput={(e) => onAmountChange(split, 'credit', (e.target as HTMLInputElement).value)}
      placeholder={i18n.t.commonZeroPlaceholder}
      class="sharp-input text-text-base font-proto w-full text-right"
    />
  </td>
  <td class="px-2 py-1.5">
    <input
      bind:value={split.memo}
      placeholder={i18n.t.txSplitMemoPlaceholder}
      class="sharp-input w-full"
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

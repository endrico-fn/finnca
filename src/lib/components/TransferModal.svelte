<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { toMinor, todayString, parseStringAmountToMinor } from '$lib/accounting/finance';
  import { ModalShell, Button } from '$lib/components/ui';
  import { i18n } from '$lib/i18n.svelte';

  let {
    open = $bindable(false),
    initialFrom = '',
    initialTo = '',
    onSuccess,
  }: {
    open: boolean;
    initialFrom?: string;
    initialTo?: string;
    onSuccess?: () => void;
  } = $props();

  // svelte-ignore state_referenced_locally
  let transferFrom = $state(initialFrom);
  // svelte-ignore state_referenced_locally
  let transferTo = $state(initialTo);
  let transferAmount = $state('');
  let transferDate = $state(todayString());
  let transferNote = $state('');
  let transferError = $state('');
  let transferBusy = $state(false);

  $effect(() => {
    if (open) {
      transferFrom = initialFrom;
      transferTo = initialTo;
      transferError = '';
      if (!transferDate) transferDate = todayString();
    }
  });

  const transferAssetAccounts = $derived(
    ledger.accounts.filter((a) => !a.placeholder && (a.type === 'ASSET' || a.type === 'LIABILITY'))
  );

  async function executeTransfer() {
    transferError = '';
    if (!transferFrom || !transferTo) {
      transferError = i18n.t.selectSourceTarget;
      return;
    }
    if (transferFrom === transferTo) {
      transferError = i18n.t.sourceTargetSame;
      return;
    }
    const fromAcc = ledger.accountsById.get(transferFrom);
    const toAcc = ledger.accountsById.get(transferTo);
    if (!fromAcc || !toAcc) return;

    const minor = parseStringAmountToMinor(transferAmount, fromAcc.currency);
    if (minor <= 0) {
      transferError = i18n.t.transferValidAmount;
      return;
    }
    if (fromAcc.currency !== toAcc.currency) {
      transferError = i18n.t.transferCurrencyMismatch
        .replace('{from}', fromAcc.name)
        .replace('{fromCur}', fromAcc.currency)
        .replace('{to}', toAcc.name)
        .replace('{toCur}', toAcc.currency);
      return;
    }

    transferBusy = true;
    try {
      await ledger.upsertTransaction({
        id: crypto.randomUUID(),
        date: transferDate,
        description:
          transferNote.trim() ||
          i18n.t.transferDesc.replace('{from}', fromAcc.name).replace('{to}', toAcc.name),
        currency: fromAcc.currency,
        splits: [
          {
            id: crypto.randomUUID(),
            accountId: transferFrom,
            amount: -minor,
            reconcile: 'n',
          },
          {
            id: crypto.randomUUID(),
            accountId: transferTo,
            amount: minor,
            reconcile: 'n',
          },
        ],
      });
      open = false;
      transferAmount = '';
      transferNote = '';
      transferError = '';
      onSuccess?.();
    } catch (e) {
      transferError = String(e).replace('Error: ', '');
    } finally {
      transferBusy = false;
    }
  }
</script>

<ModalShell bind:open title={i18n.t.quickTransferTitle}>
  <div class="space-y-2">
    <div>
      <div class="label-xs text-text-base mb-1 block">
        {i18n.t.transferSource}
      </div>
      <select bind:value={transferFrom} class="sharp-input w-full px-2.5 py-1.5 text-[12px]">
        <option value="">{i18n.t.transferSelectSource}</option>
        {#each transferAssetAccounts as a (a.id || a)}
          <option value={a.id}>{a.code} - {a.name} ({a.currency})</option>
        {/each}
      </select>
    </div>

    <div>
      <div class="label-xs text-text-base mb-1 block">
        {i18n.t.transferTarget}
      </div>
      <select bind:value={transferTo} class="sharp-input w-full px-2.5 py-1.5 text-[12px]">
        <option value="">{i18n.t.transferSelectTarget}</option>
        {#each transferAssetAccounts as a (a.id || a)}
          <option value={a.id}>{a.code} - {a.name} ({a.currency})</option>
        {/each}
      </select>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.amount}
        </div>
        <input
          type="text"
          bind:value={transferAmount}
          placeholder="500000"
          class="sharp-input w-full px-2.5 py-1.5 text-[12px]"
        />
      </div>
      <div>
        <div class="label-xs text-text-base mb-1 block">{i18n.t.date}</div>
        <input
          type="date"
          bind:value={transferDate}
          class="sharp-input w-full px-2 py-2 text-[11px]"
        />
      </div>
    </div>

    <div>
      <div class="label-xs text-text-base mb-1 block">
        {i18n.t.transferNoteLabel}
      </div>
      <input
        type="text"
        bind:value={transferNote}
        placeholder={i18n.t.transferNotePlaceholder}
        class="sharp-input w-full px-2.5 py-1.5 text-[12px]"
      />
    </div>

    {#if transferError}
      <p class="badge-err px-2.5 py-1 text-[11px]">{transferError}</p>
    {/if}
  </div>

  <div class="border-line mt-3 flex justify-end gap-2 border-t pt-3">
    <Button variant="ghost" onclick={() => (open = false)} disabled={transferBusy}>
      {i18n.t.cancelBtn}
    </Button>
    <Button variant="primary" onclick={executeTransfer} disabled={transferBusy}>
      {transferBusy ? i18n.t.transferringBtn : i18n.t.transferBtn}
    </Button>
  </div>
</ModalShell>

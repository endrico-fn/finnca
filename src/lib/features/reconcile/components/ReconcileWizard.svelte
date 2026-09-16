<script lang="ts">
  import { onMount } from 'svelte';
  import { listAccountsCmd, readStatementFileCmd, type Account } from '$lib/core/ipc/bindings';
  import { reconcileState } from '../state/reconcile.svelte';
  import { toMinor } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { PageLayout, Button } from '$lib/components/ui';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { parseCsvStatement } from '../state/statementParser';
  import ReconcileSidebar from './ReconcileSidebar.svelte';
  import ReconcileTable from './ReconcileTable.svelte';

  let accounts = $state<Account[]>([]);

  onMount(async () => {
    try {
      const items = await listAccountsCmd();
      accounts = items.map((i) => i.account);
    } catch {
      accounts = [];
    }
  });

  const account = $derived(accounts.find((a) => a.id === reconcileState.selectedAccountId));
  const isDebit = $derived(
    account?.account_type === 'ASSET' || account?.account_type === 'EXPENSE'
  );

  const startingBalance = $derived(reconcileState.reconciledBalance);
  const clearedBalance = $derived(reconcileState.clearedBalance);

  const effectiveStartingBalance = $derived(isDebit ? startingBalance : -startingBalance);
  const effectiveClearedBalance = $derived(isDebit ? clearedBalance : -clearedBalance);

  const targetBalance = $derived.by(() => {
    const raw = reconcileState.targetBalanceStr.replace(/[^0-9.-]/g, '');
    if (!raw.trim()) return 0;
    const num = parseFloat(raw);
    if (isNaN(num)) return 0;
    return toMinor(account?.currency || 'IDR', num);
  });

  $effect(() => {
    reconcileState.targetBalanceMinor = targetBalance;
  });

  const difference = $derived(targetBalance - effectiveClearedBalance);

  const accountOptions = $derived(
    accounts
      .filter(
        (a) => !a.placeholder && (a.account_type === 'ASSET' || a.account_type === 'LIABILITY')
      )
      .map((acc) => ({ value: acc.id, label: `${acc.code} - ${acc.name}` }))
  );

  async function handleAccountChange(id: string) {
    await reconcileState.selectAccount(id);
  }

  async function finishReconciliation() {
    if (difference !== 0) return;
    try {
      await reconcileState.finish();
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.reconcileTitle,
        message: i18n.t.reconcileReadyMsg,
      });
    } catch (e) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.reconcileTitle,
        message: String(e),
      });
    }
  }

  async function handleCsvUpload() {
    if (!reconcileState.selectedAccountId) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.reconcileTitle,
        message: i18n.t.reconcileSelectAccountWarning,
      });
      return;
    }

    const selectedPath = await open({
      multiple: false,
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    });

    if (!selectedPath || typeof selectedPath !== 'string') return;

    try {
      const fileContent = await readStatementFileCmd(selectedPath);
      const statements = parseCsvStatement(fileContent, account?.currency || 'IDR', isDebit);
      if (statements.length === 0) return;

      const result = await reconcileState.matchStatement(statements, true);
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.reconcileTitle,
        message: `${result.matched_count} ${i18n.t.clearedStatus}`,
      });
    } catch (e) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.reconcileTitle,
        message: String(e),
      });
    }
  }
</script>

<PageLayout title={i18n.t.reconcile}>
  {#snippet actions()}
    <div class="flex items-center gap-1.5">
      <Button
        variant={reconcileState.selectedAccountId &&
        difference === 0 &&
        reconcileState.targetBalanceStr !== ''
          ? 'tactical'
          : 'primary'}
        title={i18n.t.finishReconciliationBtn}
        ariaLabel={i18n.t.finishReconciliationBtn}
        disabled={!reconcileState.selectedAccountId ||
          difference !== 0 ||
          reconcileState.targetBalanceStr === ''}
        onclick={finishReconciliation}
      >
        {i18n.t.finishReconciliationBtn}
      </Button>
    </div>
  {/snippet}

  <div class="flex flex-1 gap-2 overflow-hidden">
    <ReconcileSidebar
      {account}
      {accountOptions}
      onSelectAccount={handleAccountChange}
      {effectiveStartingBalance}
      {effectiveClearedBalance}
      {difference}
      onUploadCsv={handleCsvUpload}
    />
    <ReconcileTable currency={account?.currency || 'IDR'} />
  </div>
</PageLayout>

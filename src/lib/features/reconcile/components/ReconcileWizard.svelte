<script lang="ts">
  import { onMount } from 'svelte';
  import { listAccountsCmd, readStatementFileCmd, type Account } from '$lib/core/ipc/bindings';
  import { reconcileState } from '../state/reconcile.svelte';
  import { parseStringAmountToMinor } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { PageLayout, Button } from '$lib/components/ui';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { pickOpenFile } from '$lib/core/dialog';
  import { parseCsvStatement } from '../state/statementParser';
  import { modalState } from '$lib/core/state/modal.svelte';
  import type { MatchStatementOutput, StatementRow } from '../state/reconcile.svelte';
  import ReconcileSidebar from './ReconcileSidebar.svelte';
  import ReconcileTable from './ReconcileTable.svelte';
  import ReconcileMatchModal from './ReconcileMatchModal.svelte';
  import ReconcileRulesModal from './ReconcileRulesModal.svelte';

  let accounts = $state<Account[]>([]);
  let showMatchModal = $state(false);
  let showRulesModal = $state(false);
  let matchResult = $state<MatchStatementOutput | null>(null);

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
    return parseStringAmountToMinor(reconcileState.targetBalanceStr, account?.currency || 'IDR');
  });

  $effect(() => {
    reconcileState.targetBalanceMinor = targetBalance;
  });

  const difference = $derived(targetBalance - effectiveClearedBalance);

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

    const selectedPath = await pickOpenFile([{ name: 'CSV', extensions: ['csv'] }]);

    if (!selectedPath) return;

    try {
      const fileContent = await readStatementFileCmd(selectedPath);
      const statements = parseCsvStatement(fileContent, account?.currency || 'IDR', isDebit);
      if (statements.length === 0) return;

      const result = await reconcileState.matchStatement(statements, false);
      matchResult = result;
      showMatchModal = true;
    } catch (e) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.reconcileTitle,
        message: String(e),
      });
    }
  }

  async function handleApplyMatches(postingIds: string[]) {
    try {
      await reconcileState.applyMatchedPostings(postingIds);
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.reconcileTitle,
        message: `${postingIds.length} ${i18n.t.clearedStatus}`,
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

  function resolveAdjustmentAccount(adjType: 'FEE' | 'INTEREST'): string {
    if (adjType === 'INTEREST') {
      const inc = accounts.find(
        (a) => !a.placeholder && a.account_type === 'INCOME' && /bunga|interest|giro/i.test(a.name)
      );
      return (
        inc?.id || accounts.find((a) => !a.placeholder && a.account_type === 'INCOME')?.id || ''
      );
    } else {
      const exp = accounts.find(
        (a) =>
          !a.placeholder &&
          a.account_type === 'EXPENSE' &&
          /adm|admin|biaya bank|bank fee/i.test(a.name)
      );
      return (
        exp?.id || accounts.find((a) => !a.placeholder && a.account_type === 'EXPENSE')?.id || ''
      );
    }
  }

  function handleQuickBankFee(row: StatementRow, adjType: 'FEE' | 'INTEREST') {
    const targetAccId = resolveAdjustmentAccount(adjType);
    handleQuickAdd(row, targetAccId);
  }

  function handleQuickAdd(
    row: StatementRow,
    suggestedAccountId = '',
    overrideDescription?: string
  ) {
    if (!reconcileState.selectedAccountId) return;
    const isIncome = row.amount > 0;
    const finalDesc = overrideDescription || row.description || '';
    modalState.openQuickTx({
      id: crypto.randomUUID(),
      date: row.date,
      description: finalDesc,
      notes: '[From Bank Statement]',
      currency: account?.currency || 'IDR',
      fx_rate: null,
      postings: isIncome
        ? [
            {
              id: crypto.randomUUID(),
              account_id: reconcileState.selectedAccountId,
              amount: row.amount,
              memo: row.description || null,
              reconcile: 'c',
            },
            {
              id: crypto.randomUUID(),
              account_id: suggestedAccountId,
              amount: -row.amount,
              memo: null,
              reconcile: 'n',
            },
          ]
        : [
            {
              id: crypto.randomUUID(),
              account_id: suggestedAccountId,
              amount: -row.amount,
              memo: null,
              reconcile: 'n',
            },
            {
              id: crypto.randomUUID(),
              account_id: reconcileState.selectedAccountId,
              amount: row.amount,
              memo: row.description || null,
              reconcile: 'c',
            },
          ],
    });
  }
</script>

<PageLayout title={i18n.t.reconcile}>
  {#snippet actions()}
    <div class="flex items-center gap-1.5">
      <Button
        variant="outline"
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
        title={i18n.t.reconcileRulesManageBtn}
        ariaLabel={i18n.t.reconcileRulesManageBtn}
        onclick={() => (showRulesModal = true)}
      >
        ⚙ {i18n.t.reconcileRulesManageBtn}
      </Button>
      <Button
        variant={reconcileState.selectedAccountId &&
        difference === 0 &&
        reconcileState.targetBalanceStr !== ''
          ? 'tactical'
          : 'primary'}
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
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
      {accounts}
      onSelectAccount={handleAccountChange}
      {effectiveStartingBalance}
      {effectiveClearedBalance}
      {difference}
      onUploadCsv={handleCsvUpload}
    />
    <ReconcileTable currency={account?.currency || 'IDR'} />
  </div>
</PageLayout>

<ReconcileMatchModal
  bind:open={showMatchModal}
  {matchResult}
  {accounts}
  currency={account?.currency || 'IDR'}
  onApply={handleApplyMatches}
  onClose={() => (showMatchModal = false)}
  onQuickAdd={handleQuickAdd}
  onQuickBankFee={handleQuickBankFee}
/>

<ReconcileRulesModal bind:open={showRulesModal} {accounts} />

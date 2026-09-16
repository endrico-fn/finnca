<script lang="ts">
  import { accountsState, getAccountPath } from '$lib/features/accounts/state/accounts.svelte';
  import {
    filterAccountRows,
    computeAccountTypeCounts,
  } from '$lib/features/accounts/state/accountFilters';
  import type { Account, AccountType } from '$lib/core/ipc/bindings';
  import { i18n } from '$lib/core/i18n.svelte';
  import { extractErrorMessage } from '$lib/core/ipc/errors';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import ConfirmDialog from '$lib/components/feedback/ConfirmDialog.svelte';
  import AccountTable, {
    type AccountRowItem,
  } from '$lib/features/accounts/components/AccountTable.svelte';
  import AccountFilterBar from '$lib/features/accounts/components/AccountFilterBar.svelte';
  import AccountModal from '$lib/features/accounts/components/AccountModal.svelte';
  import AccountTypeLegend from '$lib/features/accounts/components/AccountTypeLegend.svelte';
  import TransferModal from '$lib/features/journal/components/TransferModal.svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { onMount } from 'svelte';
  import { Splash, ErrorState, PageLayout, Card, Button } from '$lib/components/ui';

  let accountModalOpen = $state(false);
  let editingAccount = $state<Account | null>(null);
  let parentIdForNew = $state<string | null>(null);

  let searchQuery = $state('');
  let selectedTypeFilter = $state<AccountType | 'ALL'>('ALL');
  let showHidden = $state(false);
  let placeholderOnly = $state(false);
  let confirmDeleteOpen = $state(false);
  let deleteTarget = $state<Account | null>(null);
  let deleteError = $state('');
  let transferOpen = $state(false);
  let selectedId = $state<string | null>(null);

  const selectedAccount = $derived(
    selectedId ? (accountsState.accountsById.get(selectedId) ?? null) : null
  );

  const accountsWithPath = $derived(
    accountsState.items.map((item): AccountRowItem => ({
      ...item.account,
      fullPath: getAccountPath(item.account, accountsState.accountsById),
      balance: item.recursive_balance,
    }))
  );

  const filteredAccounts = $derived(
    filterAccountRows(
      accountsWithPath,
      searchQuery,
      selectedTypeFilter,
      showHidden,
      placeholderOnly
    )
  );

  const countsData = $derived(
    computeAccountTypeCounts(
      accountsWithPath,
      searchQuery,
      showHidden,
      placeholderOnly,
      selectedTypeFilter
    )
  );

  function resetAccountFilter() {
    selectedTypeFilter = 'ALL';
    placeholderOnly = false;
    showHidden = false;
  }

  function openAdd(parentId: string | null = null) {
    editingAccount = null;
    parentIdForNew = parentId;
    accountModalOpen = true;
  }

  function openEdit(acc: Account) {
    editingAccount = acc;
    parentIdForNew = null;
    accountModalOpen = true;
  }

  function requestDelete(acc: Account) {
    deleteTarget = acc;
    confirmDeleteOpen = true;
  }

  async function executeDelete() {
    if (!deleteTarget) return;
    deleteError = '';
    try {
      await accountsState.delete(deleteTarget.id);
      if (selectedId === deleteTarget.id) {
        selectedId = null;
      }
    } catch (e) {
      const msg = extractErrorMessage(e);
      deleteError = msg;
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.deleteFailedTitle,
        message: msg,
      });
    }
  }

  async function toggleHide(acc: Account, e: MouseEvent) {
    e.stopPropagation();
    deleteError = '';
    try {
      await accountsState.update(acc.id, { hidden: !acc.hidden });
    } catch (err) {
      const msg = extractErrorMessage(err);
      deleteError = msg;
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.deleteFailedTitle,
        message: msg,
      });
    }
  }

  function handleRowClick(acc: AccountRowItem) {
    selectedId = acc.id;
  }

  function handleRowDblClick(acc: AccountRowItem) {
    selectedId = acc.id;
    if (!acc.placeholder) {
      goto(resolve('/app/accounts/[code]', { code: acc.code }));
    } else {
      openEdit(acc);
    }
  }

  onMount(() => {
    if (accountsState.items.length === 0) {
      accountsState.load();
    }
  });
</script>

<PageLayout title={i18n.t.account}>
  {#snippet actions()}
    <div class="flex shrink-0 items-center gap-1.5">
      <Button
        variant="primary"
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
        title={i18n.t.rootAccount}
        ariaLabel={i18n.t.rootAccount}
        onclick={() => openAdd(null)}
      >
        {i18n.t.rootAccount}
      </Button>
      <Button
        variant="ghost"
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
        title={selectedAccount
          ? `${i18n.t.subAccount}: ${selectedAccount.name}`
          : i18n.t.selectAccountFirst}
        ariaLabel={i18n.t.subAccount}
        disabled={!selectedAccount}
        onclick={() => openAdd(selectedAccount?.id ?? null)}
      >
        {i18n.t.subAccount}
      </Button>
      <Button
        variant="ghost"
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
        title={selectedAccount
          ? `${i18n.t.edit} ${selectedAccount.code} — ${selectedAccount.name}`
          : i18n.t.selectAccountFirst}
        ariaLabel={i18n.t.editAccount}
        disabled={!selectedAccount}
        onclick={() => selectedAccount && openEdit(selectedAccount)}
      >
        {i18n.t.editAccount}
      </Button>
      <Button
        variant="ghost"
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
        title={i18n.t.transfersTitle}
        ariaLabel={i18n.t.transfersTitle}
        onclick={() => (transferOpen = true)}
      >
        {i18n.t.transfersTitle}
      </Button>
    </div>
  {/snippet}

  {#if accountsState.loading && accountsState.items.length === 0}
    <Splash variant="inline" label={i18n.t.loadingFinnca} />
  {:else if accountsState.error}
    <ErrorState message={accountsState.error} onRetry={() => accountsState.load()} />
  {:else}
    <Card
      title="{i18n.t.chartOfAccounts} ({filteredAccounts.length})"
      class="min-h-0 flex-1"
      padding={false}
    >
      {#snippet header()}
        <span
          class="font-proto text-text-muted text-smaller hidden leading-none tracking-widest uppercase md:inline"
        >
          {i18n.t.pathHierarchy}
        </span>
      {/snippet}

      <AccountFilterBar
        bind:searchQuery
        bind:selectedType={selectedTypeFilter}
        bind:placeholderOnly
        bind:showHidden
        typeCounts={countsData.typeCounts}
        phCount={countsData.phCount}
        filteredCount={filteredAccounts.length}
        totalCount={accountsWithPath.length}
        onReset={resetAccountFilter}
      />

      {#if deleteError}
        <div class="px-3 pt-2">
          <div class="badge-err font-proto text-small px-3 py-2">{deleteError}</div>
        </div>
      {/if}

      <div class="font-proto text-small min-h-0 flex-1 overflow-y-auto py-0">
        {#if filteredAccounts.length === 0}
          {#if accountsState.items.length === 0}
            <div class="flex h-full flex-col items-center justify-center space-y-3 p-8 text-center">
              <p class="text-text-muted">{i18n.t.noAccountsYet}</p>
              <Button variant="primary" onclick={() => accountsState.seedRoots()}>
                <span class="font-proto text-small inline-flex items-center gap-1.5 font-bold">
                  ◈ {i18n.t.initRootAccounts}
                </span>
              </Button>
              <p class="text-text-dim text-smaller font-proto">{i18n.t.initRootAccountsDesc}</p>
            </div>
          {:else}
            <div class="text-text-muted flex h-full items-center justify-center p-8 text-center">
              <p>{i18n.t.noAccountsMatch}</p>
            </div>
          {/if}
        {:else}
          <AccountTable
            accounts={filteredAccounts}
            {selectedId}
            onSelect={handleRowClick}
            onDblClick={handleRowDblClick}
            onToggleHide={toggleHide}
          />
        {/if}
      </div>

      <AccountTypeLegend />
    </Card>
  {/if}

  <AccountModal
    bind:open={accountModalOpen}
    {editingAccount}
    parentId={parentIdForNew}
    onSaved={() => accountsState.load()}
    onDeleteRequest={requestDelete}
  />

  <ConfirmDialog
    bind:open={confirmDeleteOpen}
    title={i18n.t.deleteAccountTitle}
    message={i18n.t.confirmDeleteAccountMsg
      .replace('{code}', deleteTarget?.code ?? '')
      .replace('{name}', deleteTarget?.name ?? '')}
    confirmLabel={i18n.t.confirmBtn}
    cancelLabel={i18n.t.cancelModalBtn}
    onConfirm={executeDelete}
  />

  <TransferModal bind:open={transferOpen} onSuccess={() => accountsState.load()} />
</PageLayout>

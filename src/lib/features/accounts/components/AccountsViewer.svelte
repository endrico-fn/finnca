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
  import { modalState } from '$lib/core/state/modal.svelte';
  import AccountTable, {
    type AccountRowItem,
  } from '$lib/features/accounts/components/AccountTable.svelte';
  import AccountFilterBar from '$lib/features/accounts/components/AccountFilterBar.svelte';
  import AccountModal from '$lib/features/accounts/components/AccountModal.svelte';
  import AccountTypeLegend from '$lib/features/accounts/components/AccountTypeLegend.svelte';
  import AccountInspectorPane from '$lib/features/accounts/components/AccountInspectorPane.svelte';
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
  let deleteError = $state('');
  let selectedId = $state<string | null>(null);

  const selectedAccount = $derived(
    selectedId ? (accountsState.accountsById.get(selectedId) ?? null) : null
  );
  const selectedItem = $derived(
    selectedId ? (accountsState.items.find((it) => it.account.id === selectedId) ?? null) : null
  );
  const selectedFullPath = $derived(
    selectedAccount ? getAccountPath(selectedAccount, accountsState.accountsById) : ''
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
    modalState.confirm({
      title: i18n.t.deleteAccountTitle,
      message: i18n.t.confirmDeleteAccountMsg
        .replace('{code}', acc.code)
        .replace('{name}', acc.name),
      confirmLabel: i18n.t.confirmBtn,
      cancelLabel: i18n.t.cancelModalBtn,
      danger: true,
      onConfirm: async () => {
        deleteError = '';
        try {
          await accountsState.delete(acc.id);
          if (selectedId === acc.id) {
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
      },
    });
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

  function handleKeydown(e: KeyboardEvent) {
    if (
      e.target instanceof HTMLInputElement ||
      e.target instanceof HTMLTextAreaElement ||
      e.target instanceof HTMLSelectElement
    ) {
      return;
    }

    if (e.key === 'j' || e.key === 'ArrowDown') {
      if (filteredAccounts.length === 0) return;
      e.preventDefault();
      const currentIdx = filteredAccounts.findIndex((a) => a.id === selectedId);
      const nextIdx = currentIdx < 0 ? 0 : Math.min(filteredAccounts.length - 1, currentIdx + 1);
      const nextAcc = filteredAccounts[nextIdx];
      selectedId = nextAcc.id;
      document.getElementById(`acc-row-${nextAcc.id}`)?.scrollIntoView({ block: 'nearest' });
    } else if (e.key === 'k' || e.key === 'ArrowUp') {
      if (filteredAccounts.length === 0) return;
      e.preventDefault();
      const currentIdx = filteredAccounts.findIndex((a) => a.id === selectedId);
      const prevIdx = currentIdx < 0 ? filteredAccounts.length - 1 : Math.max(0, currentIdx - 1);
      const prevAcc = filteredAccounts[prevIdx];
      selectedId = prevAcc.id;
      document.getElementById(`acc-row-${prevAcc.id}`)?.scrollIntoView({ block: 'nearest' });
    } else if (e.key === 'Enter') {
      if (selectedAccount) {
        e.preventDefault();
        const currentItem = filteredAccounts.find((a) => a.id === selectedId);
        if (currentItem) {
          handleRowDblClick(currentItem);
        }
      }
    } else if (e.key === 'e' || e.key === 'E') {
      if (selectedAccount) {
        e.preventDefault();
        openEdit(selectedAccount);
      }
    } else if (e.key === 'a' || e.key === 'A') {
      if (selectedAccount) {
        e.preventDefault();
        openAdd(selectedAccount.id);
      }
    } else if (e.key === 'Escape') {
      if (selectedId) {
        selectedId = null;
      }
    }
  }

  onMount(() => {
    if (accountsState.items.length === 0) {
      accountsState.load();
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

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
    </div>
  {/snippet}

  {#if accountsState.loading && accountsState.items.length === 0}
    <Splash variant="inline" label={i18n.t.loadingFinnca} />
  {:else if accountsState.error}
    <ErrorState message={accountsState.error} onRetry={() => accountsState.load()} />
  {:else}
    <Card
      title="{i18n.t.chartOfAccounts} ({filteredAccounts.length})"
      class="border-line bg-bg-card flex min-h-0 flex-1 flex-col overflow-hidden border"
      padding={false}
      borderHeader
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

      <div class="flex min-h-0 flex-1 flex-col overflow-hidden xl:flex-row">
        <div class="font-proto text-small min-h-0 flex-1 overflow-y-auto py-0">
          {#if filteredAccounts.length === 0}
            {#if accountsState.items.length === 0}
              <div
                class="flex h-full flex-col items-center justify-center space-y-3 p-8 text-center"
              >
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

        {#if selectedAccount && selectedItem}
          <div
            class="border-line bg-bg-card flex min-h-0 w-full shrink-0 flex-col overflow-y-auto border-t xl:w-80 xl:border-t-0 xl:border-l"
          >
            <AccountInspectorPane
              account={selectedAccount}
              directBalance={selectedItem.direct_balance}
              rollupBalance={selectedItem.recursive_balance}
              fullPath={selectedFullPath}
              onClose={() => (selectedId = null)}
              onAddSubAccount={(parentId) => openAdd(parentId)}
              onEdit={(acc) => openEdit(acc)}
              onViewLedger={(code) => goto(resolve('/app/accounts/[code]', { code }))}
              onDelete={(acc) => requestDelete(acc)}
              onToggleHide={(acc, e) => toggleHide(acc, e)}
            />
          </div>
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
</PageLayout>

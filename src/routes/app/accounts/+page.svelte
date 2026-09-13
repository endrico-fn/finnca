<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import {
    ACCOUNT_TYPES,
    accountTypeLabel,
    ACCOUNT_TYPE_COLOR,
    type Account,
    type AccountType,
    type Currency,
  } from '$lib/accounting/types';
  import {
    uid,
    accountBalanceMinor,
    buildChildrenMap,
    parseStringAmountToMinor,
    formatIDR,
    formatUSD,
    todayString,
  } from '$lib/accounting/finance';
  import { i18n } from '$lib/i18n.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import AccountForm from '$lib/components/AccountForm.svelte';
  import TransferModal from '$lib/components/TransferModal.svelte';
  import { getAccountPath } from '$lib/accounting/finance';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { onMount } from 'svelte';
  import {
    Splash,
    ErrorState,
    PageLayout,
    Card,
    Button,
    Icon,
    ModalShell,
    SearchBar,
    FilterMenu,
    FilterSection,
    FilterOption,
  } from '$lib/components/ui';

  // ── Overlay ADD/EDIT ACCOUNT (single, center) ──
  let overlayOpen = $state(false);
  let isNew = $state(false);
  let editingId = $state<string | null>(null);
  let form = $state<Partial<Account>>({});
  let error = $state('');
  let saving = $state(false);
  let openingBalance = $state('');
  let openingBalanceDate = $state(todayString());

  let searchQuery = $state('');
  let selectedTypeFilter = $state<AccountType | 'ALL'>('ALL');
  let showHidden = $state(false);
  let placeholderOnly = $state(false);
  let filterOpen = $state(false);
  let confirmDeleteOpen = $state(false);
  let deleteTarget = $state<Account | null>(null);
  let transferOpen = $state(false);
  let selectedId = $state<string | null>(null);
  const selectedAccount = $derived(
    selectedId ? (ledger.accountsById.get(selectedId) ?? null) : null
  );

  const childrenMap = $derived(ledger.data ? buildChildrenMap(ledger.data.accounts) : new Map());

  const accountsWithPath = $derived(
    ledger.accounts.map((acc) => ({
      ...acc,
      fullPath: getAccountPath(acc, ledger.accountsById),
      balance: ledger.data ? accountBalanceMinor(acc.id, ledger.data, childrenMap) : 0,
    }))
  );

  const filteredAccounts = $derived(
    accountsWithPath
      .filter((acc) => {
        if (!showHidden && acc.hidden) return false;
        if (placeholderOnly && !acc.placeholder) return false;
        if (selectedTypeFilter !== 'ALL' && acc.type !== selectedTypeFilter) return false;
        if (searchQuery.trim()) {
          const q = searchQuery.toLowerCase().trim();
          return (
            acc.code.toLowerCase().includes(q) ||
            acc.fullPath.toLowerCase().includes(q) ||
            (acc.note ?? '').toLowerCase().includes(q)
          );
        }
        return true;
      })
      .sort((a, b) => a.code.localeCompare(b.code, undefined, { numeric: true }))
  );

  const baseHiddenSearch = $derived(
    accountsWithPath.filter((acc) => {
      if (!showHidden && acc.hidden) return false;
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase().trim();
        return (
          acc.code.toLowerCase().includes(q) ||
          acc.fullPath.toLowerCase().includes(q) ||
          (acc.note ?? '').toLowerCase().includes(q)
        );
      }
      return true;
    })
  );

  const baseForCounts = $derived(
    baseHiddenSearch.filter((acc) => !placeholderOnly || acc.placeholder)
  );

  const typeCounts = $derived.by(() => {
    const counts: Record<AccountType | 'ALL', number> = {
      ALL: baseForCounts.length,
      ASSET: 0,
      LIABILITY: 0,
      EQUITY: 0,
      INCOME: 0,
      EXPENSE: 0,
    };
    for (const acc of baseForCounts) counts[acc.type] += 1;
    return counts;
  });

  const phCount = $derived(
    baseHiddenSearch.filter(
      (acc) => (selectedTypeFilter === 'ALL' || acc.type === selectedTypeFilter) && acc.placeholder
    ).length
  );

  const isFilterActive = $derived(selectedTypeFilter !== 'ALL' || showHidden || placeholderOnly);

  const activeFilterCount = $derived(
    (selectedTypeFilter !== 'ALL' ? 1 : 0) + (showHidden ? 1 : 0) + (placeholderOnly ? 1 : 0)
  );

  const filterButtonLabel = $derived(
    selectedTypeFilter === 'ALL'
      ? i18n.t.filterBtn
      : accountTypeLabel(selectedTypeFilter).toUpperCase()
  );

  function resetAccountFilter() {
    selectedTypeFilter = 'ALL';
    placeholderOnly = false;
    showHidden = false;
  }

  function openAdd(parentId: string | null = null) {
    const parent = parentId ? ledger.accountsById.get(parentId) : null;
    form = {
      id: uid(),
      code: '',
      name: '',
      type: (parent?.type as AccountType) ?? 'ASSET',
      parentId,
      currency: (parent?.currency as Currency) ?? 'IDR',
      placeholder: false,
      hidden: false,
      note: '',
      description: '',
      color: parent ? parent.color : undefined,
    };
    isNew = true;
    editingId = null;
    openingBalance = '';
    openingBalanceDate = todayString();
    error = '';
    overlayOpen = true;
  }

  function openEdit(acc: Account) {
    form = { ...acc };
    isNew = false;
    editingId = acc.id;
    openingBalance = '';
    error = '';
    overlayOpen = true;
  }

  function closeOverlay() {
    overlayOpen = false;
    error = '';
    openingBalance = '';
  }

  async function save() {
    error = '';
    saving = true;
    try {
      if (!form.code?.trim() || !form.name?.trim()) throw new Error(i18n.t.accountCodeNameRequired);
      if (!form.type) throw new Error(i18n.t.accountTypeRequired);
      const dup = ledger.accounts.find((a) => a.code === form.code!.trim() && a.id !== form.id);
      if (dup)
        throw new Error(
          i18n.t.accountCodeDuplicate
            .replace('{code}', form.code!.trim())
            .replace('{name}', dup.name)
        );
      const acc: Account = {
        id: form.id as string,
        code: form.code!.trim(),
        name: form.name!.trim(),
        type: form.type as AccountType,
        parentId: form.parentId ?? null,
        currency: form.currency as Currency,
        placeholder: !!form.placeholder,
        hidden: !!form.hidden,
        note: form.note?.trim() || undefined,
        description: (form.description as string)?.trim() || undefined,
        color: form.color,
        createdAt:
          (ledger.accounts.find((a) => a.id === editingId) as Account)?.createdAt ??
          new Date().toISOString(),
      };
      // auto-convert leaf parent to placeholder when creating sub-account
      if (isNew && acc.parentId) {
        const parent = ledger.accountsById.get(acc.parentId);
        if (parent && !parent.placeholder) {
          await ledger.upsertAccount({ ...parent, placeholder: true });
        }
      }
      if (!acc.placeholder && ledger.accounts.some((a) => a.parentId === acc.id))
        throw new Error(i18n.t.cannotDisablePlaceholder);
      await ledger.upsertAccount(acc);

      if (isNew && openingBalance.trim()) {
        const minor = parseStringAmountToMinor(openingBalance, acc.currency);
        if (minor > 0) {
          const obEquity = ledger.accounts.find(
            (a) =>
              !a.placeholder &&
              (a.code === '3010' ||
                a.name.toLowerCase().includes('opening balance') ||
                a.name.toLowerCase().includes('modal awal') ||
                a.name.toLowerCase().includes('saldo awal') ||
                a.type === 'EQUITY')
          );
          if (obEquity) {
            if (obEquity.currency !== acc.currency) {
              error = i18n.t.currencyMismatchOpenBalance
                .replace('{acc}', acc.currency)
                .replace('{eq}', obEquity.currency)
                .replace('{name}', obEquity.name)
                .replace('{cur}', acc.currency);
              saving = false;
              return;
            }
            const isDebitNorm = acc.type === 'ASSET' || acc.type === 'EXPENSE';
            await ledger.upsertTransaction({
              id: uid(),
              date: openingBalanceDate || todayString(),
              description: i18n.t.openingBalanceDesc.replace('{name}', acc.name),
              currency: acc.currency,
              notes: i18n.t.openingBalanceNotes,
              splits: [
                {
                  id: uid(),
                  accountId: acc.id,
                  amount: isDebitNorm ? minor : -minor,
                  reconcile: 'y',
                },
                {
                  id: uid(),
                  accountId: obEquity.id,
                  amount: isDebitNorm ? -minor : minor,
                  reconcile: 'y',
                },
              ],
            });
          }
        }
      }
      closeOverlay();
    } catch (e) {
      error = String(e).replace('Error: ', '');
    } finally {
      saving = false;
    }
  }

  function requestDelete(acc: Account) {
    deleteTarget = acc;
    confirmDeleteOpen = true;
  }

  async function executeDelete() {
    if (!deleteTarget) return;
    try {
      await ledger.deleteAccount(deleteTarget.id);
    } catch (e) {
      error = String(e).replace('Error: ', '');
    }
  }

  async function toggleHide(acc: Account, e: MouseEvent) {
    e.stopPropagation();
    await ledger.upsertAccount({ ...acc, hidden: !acc.hidden });
  }

  function handleRowClick(acc: (typeof filteredAccounts)[0]) {
    selectedId = acc.id;
  }
  function handleRowDblClick(acc: (typeof filteredAccounts)[0]) {
    selectedId = acc.id;
    if (!acc.placeholder) goto(resolve('/app/accounts/[code]', { code: acc.code }));
    else openEdit(acc);
  }

  onMount(() => {
    if (!ledger.data) ledger.load();
  });
</script>

<PageLayout title={i18n.t.account}>
  {#snippet actions()}
    <Button variant="ghost" onclick={() => (transferOpen = true)}>
      <span class="text-income inline-flex items-center gap-1.5">
        <Icon name="transfer" size={12} />
        {i18n.t.transferTitle}
      </span>
    </Button>
    <Button
      variant="primary"
      onclick={() => openAdd(selectedAccount?.placeholder ? selectedAccount.id : null)}
    >
      {selectedAccount?.placeholder ? i18n.t.subAccount : i18n.t.rootAccount}
    </Button>
    <Button
      variant="ghost"
      onclick={() => selectedAccount && openEdit(selectedAccount)}
      disabled={!selectedAccount}
    >
      <span
        title={selectedAccount
          ? `${i18n.t.edit} ${selectedAccount.code} — ${selectedAccount.name}`
          : i18n.t.selectAccountFirst}
      >
        {i18n.t.editAccount}
      </span>
    </Button>
  {/snippet}

  {#if ledger.loading}
    <Splash variant="inline" label={i18n.t.loadingFinnca} />
  {:else if ledger.error}
    <ErrorState message={ledger.error} onRetry={() => ledger.load()} />
  {:else}
    <Card
      title="{i18n.t.chartOfAccounts} ({filteredAccounts.length})"
      class="min-h-0 flex-1"
      padding={false}
    >
      {#snippet header()}
        <span
          class="font-proto text-text-muted text-smaller leading-none tracking-widest uppercase"
        >
          {i18n.t.pathHierarchy}
        </span>
      {/snippet}

      <div class="border-line flex shrink-0 flex-wrap items-center gap-2 border-b px-3 py-2.5">
        <SearchBar
          bind:value={searchQuery}
          placeholder={i18n.t.searchAccountPlaceholder}
          class="max-w-none min-w-40 flex-1"
        />

        <FilterMenu
          bind:open={filterOpen}
          label={filterButtonLabel}
          active={isFilterActive}
          count={activeFilterCount}
          onReset={resetAccountFilter}
          resetLabel={i18n.t.reset}
          resetDisabled={!isFilterActive}
        >
          <FilterSection title={i18n.t.filterTypeTitle}>
            <FilterOption
              label={i18n.t.filterAllLabel}
              count={typeCounts.ALL}
              selected={selectedTypeFilter === 'ALL'}
              check={false}
              onclick={() => {
                selectedTypeFilter = 'ALL';
                filterOpen = false;
              }}
            />
            {#each ACCOUNT_TYPES as typeKey (typeKey)}
              <FilterOption
                label={accountTypeLabel(typeKey).toUpperCase()}
                dot={ACCOUNT_TYPE_COLOR[typeKey]}
                count={typeCounts[typeKey]}
                selected={selectedTypeFilter === typeKey}
                onclick={() => {
                  selectedTypeFilter = typeKey;
                  filterOpen = false;
                }}
              />
            {/each}
          </FilterSection>

          <div class="border-line/60 border-t">
            <FilterSection title={i18n.t.filterAttrTitle} layout="list">
              <FilterOption
                label={i18n.t.filterPhOnly}
                count={phCount}
                selected={placeholderOnly}
                onclick={() => (placeholderOnly = !placeholderOnly)}
              />
              <FilterOption
                label={showHidden ? i18n.t.toggleHiddenShow : i18n.t.toggleHiddenHide}
                tone="warning"
                selected={showHidden}
                title={i18n.t.toggleHiddenTitle}
                onclick={() => (showHidden = !showHidden)}
              />
            </FilterSection>
          </div>
        </FilterMenu>
        <span class="text-text-dim font-proto text-smaller shrink-0 px-1">
          {filteredAccounts.length}/{accountsWithPath.length}
        </span>
        {#if searchQuery || isFilterActive}
          <Button
            variant="ghost"
            size="sm"
            onclick={() => {
              searchQuery = '';
              resetAccountFilter();
            }}
          >
            {i18n.t.reset}
          </Button>
        {/if}
      </div>

      <div class="font-proto text-small min-h-0 flex-1 overflow-y-auto py-0">
        {#if filteredAccounts.length === 0}
          <div class="text-text-muted flex h-full items-center justify-center p-8 text-center">
            <p>{i18n.t.noAccountsMatch}</p>
          </div>
        {:else}
          <table class="w-full border-collapse">
            <thead class="bg-bg-card sticky top-0 z-10">
              <tr class="border-line text-text-base border-b">
                <th class="label-xs w-16 py-2 pl-3 text-left font-normal">{i18n.t.colCode}</th>
                <th class="label-xs px-3 py-2 text-left font-normal">{i18n.t.colHierarchyPath}</th>
                <th class="label-xs w-36 px-3 py-2 text-right font-normal">{i18n.t.colBalance}</th>
                <th class="label-xs w-20 py-2 pr-3 text-right font-normal"></th>
              </tr>
            </thead>
            <tbody class="divide-line/40 divide-y">
              {#each filteredAccounts as acc (acc.id)}
                {@const isSelected = selectedId === acc.id}
                <tr
                  class="cursor-pointer transition-colors {isSelected
                    ? 'bg-bg-row-active'
                    : 'hover:bg-bg-row-active'} {acc.hidden ? 'opacity-40' : ''}"
                  onclick={() => handleRowClick(acc)}
                  ondblclick={() => handleRowDblClick(acc)}
                  title={acc.placeholder
                    ? i18n.t.singleSelectDoubleEdit
                    : i18n.t.singleSelectDoubleLedger}
                >
                  <td class="text-text-muted font-proto text-small py-2.5 pl-3 whitespace-nowrap">
                    {acc.code}
                  </td>
                  <td class="text-text-strong w-full max-w-0 truncate px-3 py-2.5">
                    <div class="flex items-center gap-2 truncate">
                      <span
                        class="size-1.5 shrink-0"
                        style="background:{ACCOUNT_TYPE_COLOR[acc.type]}"
                      ></span>
                      <span class="font-proto text-small truncate">{acc.fullPath}</span>
                      {#if acc.placeholder}
                        <span
                          class="bg-bg-app border-line text-text-muted font-proto text-smaller shrink-0 border px-1"
                          >{i18n.t.badgePh}</span
                        >
                      {/if}
                      {#if acc.hidden}
                        <span class="bg-line text-text-muted font-proto text-smaller shrink-0 px-1"
                          >{i18n.t.badgeHidden}</span
                        >
                      {/if}
                    </div>
                  </td>
                  <td
                    class="font-proto text-small px-3 py-2.5 text-right whitespace-nowrap {acc.balance <
                    0
                      ? 'text-expense'
                      : 'text-text-base'}"
                  >
                    {acc.currency === 'USD'
                      ? formatUSD(Math.abs(acc.balance))
                      : formatIDR(Math.abs(acc.balance))}
                  </td>
                  <td class="py-2.5 pr-3 text-right whitespace-nowrap">
                    <div class="flex items-center justify-end">
                      <button
                        type="button"
                        onclick={(e) => toggleHide(acc, e)}
                        title={acc.hidden ? i18n.t.showAccount : i18n.t.hideAccount}
                        aria-label={acc.hidden ? i18n.t.showAccount : i18n.t.hideAccount}
                        class="text-text-muted hover:text-text-base text-smaller px-1.5 py-1 transition-colors"
                      >
                        {acc.hidden ? '◉' : '◎'}
                      </button>
                    </div>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>

      <div class="border-line bg-bg-card flex shrink-0 flex-wrap gap-1.5 border-t px-3 py-2.5">
        {#each ACCOUNT_TYPES as k (k)}
          <span
            class="bg-bg-app border-line text-text-muted font-proto text-smaller inline-flex items-center gap-1.5 border px-2 py-1 leading-none"
          >
            <span class="size-1.5 shrink-0" style="background:{ACCOUNT_TYPE_COLOR[k]}"></span>
            {accountTypeLabel(k)}
          </span>
        {/each}
      </div>
    </Card>
  {/if}

  <ModalShell
    bind:open={overlayOpen}
    title={isNew ? i18n.t.newAccountTitle : i18n.t.editAccountTitle}
    maxWidth="max-w-xl"
    onClose={closeOverlay}
  >
    {#if !isNew}
      <div class="flex justify-end pb-2">
        <Button
          variant="danger"
          onclick={() => {
            if (form.id) {
              const acc = ledger.accounts.find((a) => a.id === form.id);
              if (acc) requestDelete(acc);
            }
          }}
        >
          {i18n.t.deleteAccountBtn}
        </Button>
      </div>
    {/if}
    <div class="min-h-0 flex-1 overflow-y-auto pr-2">
      <AccountForm bind:form bind:openingBalance bind:openingBalanceDate {isNew} {error} />
    </div>

    <div class="border-line flex shrink-0 gap-2 border-t pt-3">
      <Button variant="ghost" onclick={closeOverlay}>
        <span class="flex-1 text-center">{i18n.t.cancelBtn}</span>
      </Button>
      <Button variant="primary" onclick={save} disabled={saving}>
        <span class="flex-1 text-center">
          {saving ? i18n.t.savingBtn : isNew ? i18n.t.createAccountBtn : i18n.t.saveChanges}
        </span>
      </Button>
    </div>
  </ModalShell>

  <ConfirmModal
    bind:open={confirmDeleteOpen}
    title={i18n.t.deleteAccountTitle}
    message={i18n.t.confirmDeleteAccountMsg
      .replace('{code}', deleteTarget?.code ?? '')
      .replace('{name}', deleteTarget?.name ?? '')}
    confirmLabel={i18n.t.confirmBtn}
    cancelLabel={i18n.t.cancelModalBtn}
    onConfirm={executeDelete}
  />

  <TransferModal bind:open={transferOpen} />
</PageLayout>

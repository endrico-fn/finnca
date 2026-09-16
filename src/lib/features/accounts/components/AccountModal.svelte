<script lang="ts">
  import { accountsState } from '$lib/features/accounts/state/accounts.svelte';
  import {
    ensureOpeningEquity,
    isDebitNormalType,
    openingSplitAmounts,
  } from '$lib/features/accounts/state/openingBalance';
  import { postJournalEntryCmd, type Account, type AccountType } from '$lib/core/ipc/bindings';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { todayString } from '$lib/core/format/date';
  import { parseStringAmountToMinor } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import AccountForm from './AccountForm.svelte';
  import { ModalShell, Button } from '$lib/components/ui';

  let {
    open = $bindable(false),
    editingAccount = null,
    parentId = null,
    onSaved,
    onDeleteRequest,
  }: {
    open: boolean;
    editingAccount?: Account | null;
    parentId?: string | null;
    onSaved?: () => void;
    onDeleteRequest?: (acc: Account) => void;
  } = $props();

  let form = $state<Partial<Account>>({});
  let error = $state('');
  let saving = $state(false);
  let openingBalance = $state('');
  let openingBalanceDate = $state(todayString());

  const isNew = $derived(!editingAccount);

  let formTab = $state<'general' | 'more' | 'balance'>('general');

  $effect(() => {
    if (open) {
      error = '';
      openingBalance = '';
      openingBalanceDate = todayString();
      formTab = 'general';
      if (editingAccount) {
        form = { ...editingAccount };
      } else {
        const parent = parentId ? accountsState.accountsById.get(parentId) : null;
        form = {
          code: '',
          name: '',
          account_type: (parent?.account_type as AccountType) ?? 'ASSET',
          parent_id: parentId,
          currency: parent?.currency ?? 'IDR',
          placeholder: false,
          hidden: false,
          description: '',
          color: parent ? parent.color : null,
        };
      }
    }
  });

  function handleClose() {
    open = false;
    error = '';
    openingBalance = '';
    formTab = 'general';
  }

  async function save() {
    error = '';
    saving = true;
    try {
      if (!form.code?.trim() || !form.name?.trim()) throw new Error(i18n.t.accountCodeNameRequired);
      if (!form.account_type) throw new Error(i18n.t.accountTypeRequired);

      const editingId = editingAccount?.id ?? null;
      const dup = accountsState.accounts.find(
        (a) => a.code === form.code!.trim() && a.id !== editingId
      );
      if (dup) {
        throw new Error(
          i18n.t.accountCodeDuplicate
            .replace('{code}', form.code!.trim())
            .replace('{name}', dup.name)
        );
      }

      if (isNew && form.parent_id) {
        const parent = accountsState.accountsById.get(form.parent_id);
        if (parent && !parent.placeholder) {
          throw new Error(i18n.t.parentMustBePlaceholder.replace('{code}', parent.code));
        }
      }

      if (isNew && !form.parent_id && !form.placeholder) {
        throw new Error(i18n.t.rootMustBePlaceholder);
      }

      if (!form.placeholder && editingId) {
        if (accountsState.accounts.some((a) => a.parent_id === editingId)) {
          throw new Error(i18n.t.cannotDisablePlaceholder);
        }
      }

      if (isNew) {
        const wantOpening = openingBalance.trim().length > 0;
        if (wantOpening && !!form.placeholder) {
          formTab = 'balance';
          throw new Error(i18n.t.placeholderNotPostable);
        }
        let openingMinor = 0;
        if (wantOpening) {
          openingMinor = parseStringAmountToMinor(openingBalance, form.currency ?? 'IDR');
          if (openingMinor <= 0) {
            formTab = 'balance';
            throw new Error(i18n.t.openingBalanceRequired);
          }
        }

        const created = await accountsState.create({
          code: form.code!.trim(),
          name: form.name!.trim(),
          account_type: form.account_type as AccountType,
          parent_id: form.parent_id ?? null,
          currency: form.currency ?? 'IDR',
          placeholder: !!form.placeholder,
          hidden: !!form.hidden,
          description: form.description?.trim() || null,
          color: form.color || null,
        });

        if (wantOpening) {
          const obEquity = await ensureOpeningEquity(
            created.currency,
            created.id,
            i18n.t.openingBalanceAutoName.replace('{cur}', created.currency)
          );
          const [toAcc, toEq] = openingSplitAmounts(
            isDebitNormalType(created.account_type),
            openingMinor
          );
          await postJournalEntryCmd({
            date: openingBalanceDate || todayString(),
            description: i18n.t.openingBalanceDesc.replace('{name}', created.name),
            notes: i18n.t.openingBalanceNotes,
            currency: created.currency,
            fx_rate: null,
            postings: [
              {
                id: undefined,
                account_id: created.id,
                amount: toAcc,
                memo: null,
                action: null,
                reconcile: 'y',
              },
              {
                id: undefined,
                account_id: obEquity.id,
                amount: toEq,
                memo: null,
                action: null,
                reconcile: 'y',
              },
            ],
          });
          eventBus.emit('transaction:posted', { id: '' });
          await accountsState.load();
        }
      } else if (editingId) {
        await accountsState.update(editingId, {
          code: form.code!.trim(),
          name: form.name!.trim(),
          account_type: form.account_type as AccountType,
          parent_id: form.parent_id ?? null,
          currency: form.currency ?? 'IDR',
          placeholder: !!form.placeholder,
          hidden: !!form.hidden,
          description: form.description?.trim() || null,
          color: form.color || null,
        });

        if (openingBalance.trim()) {
          if (form.placeholder) {
            formTab = 'balance';
            throw new Error(i18n.t.placeholderNotPostable);
          }
          const targetMinor = parseStringAmountToMinor(openingBalance, form.currency ?? 'IDR');
          if (targetMinor <= 0) {
            formTab = 'balance';
            throw new Error(i18n.t.openingBalanceRequired);
          }
          const currentBal = accountsState.balancesById.get(editingId)?.direct ?? 0;
          const isDebitNorm = isDebitNormalType(form.account_type);
          const signedTarget = isDebitNorm ? targetMinor : -targetMinor;
          const diff = signedTarget - currentBal;
          if (diff !== 0) {
            const obEquity = await ensureOpeningEquity(
              form.currency ?? 'IDR',
              editingId,
              i18n.t.openingBalanceAutoName.replace('{cur}', form.currency ?? 'IDR')
            );
            await postJournalEntryCmd({
              date: openingBalanceDate || todayString(),
              description: `${i18n.t.openingBalanceDesc.replace('{name}', form.name || '')} (Adjustment)`,
              notes: i18n.t.openingBalanceNotes,
              currency: form.currency ?? 'IDR',
              fx_rate: null,
              postings: [
                {
                  id: undefined,
                  account_id: editingId,
                  amount: diff,
                  memo: null,
                  action: null,
                  reconcile: 'y',
                },
                {
                  id: undefined,
                  account_id: obEquity.id,
                  amount: -diff,
                  memo: null,
                  action: null,
                  reconcile: 'y',
                },
              ],
            });
            eventBus.emit('transaction:posted', { id: '' });
            await accountsState.load();
          }
        }
      }

      onSaved?.();
      handleClose();
    } catch (e) {
      error = String(e).replace('Error: ', '');
    } finally {
      saving = false;
    }
  }
</script>

<ModalShell
  bind:open
  title={isNew ? i18n.t.newAccountTitle : i18n.t.editAccountTitle}
  maxWidth="max-w-xl"
  onClose={handleClose}
>
  {#if !isNew && editingAccount}
    <div class="flex justify-end pb-2">
      <Button
        variant="danger"
        onclick={() => {
          if (editingAccount) {
            onDeleteRequest?.(editingAccount);
            handleClose();
          }
        }}
      >
        {i18n.t.deleteAccountBtn}
      </Button>
    </div>
  {/if}

  <div class="min-h-0 flex-1 overflow-y-auto pr-2">
    <AccountForm
      bind:form
      bind:openingBalance
      bind:openingBalanceDate
      bind:activeTab={formTab}
      {isNew}
      {error}
    />
  </div>

  <div class="border-line flex shrink-0 gap-2 border-t pt-3">
    <Button variant="ghost" onclick={handleClose}>
      <span class="flex-1 text-center">{i18n.t.cancelBtn}</span>
    </Button>
    <Button variant="primary" onclick={save} disabled={saving}>
      <span class="flex-1 text-center">
        {saving ? i18n.t.savingBtn : isNew ? i18n.t.createAccountBtn : i18n.t.saveChanges}
      </span>
    </Button>
  </div>
</ModalShell>

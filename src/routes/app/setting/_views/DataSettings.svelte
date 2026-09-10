<script lang="ts">
  import { i18n } from '$lib/i18n.svelte';
  import { store } from '$lib/stores/app-store.svelte';
  import { ledger } from '$lib/accounting/store.svelte';
  import { todayString, fromMinor } from '$lib/accounting/finance';
  import * as api from '$lib/api';
  import { ModalShell, Card } from '$lib/components/ui';
  import {
    forgetVault,
    clearVaultRegistry,
    getKnownVaults,
    getActiveVaultId,
    setActiveVault,
  } from '$lib/stores/vault-registry.svelte';

  let notifToggles = $state({
    due: true,
    fx: true,
    spike: true,
    integrity: true,
  });

  export function loadLocalPrefs() {
    if (typeof localStorage === 'undefined') return;
    try {
      const n = JSON.parse(localStorage.getItem('finnca_notif_toggles') || '{}');
      notifToggles = { ...notifToggles, ...n };
    } catch {}
  }

  function saveNotifToggles() {
    if (typeof localStorage === 'undefined') return;
    localStorage.setItem('finnca_notif_toggles', JSON.stringify(notifToggles));
  }

  function toggle(key: keyof typeof notifToggles) {
    notifToggles[key] = !notifToggles[key];
    saveNotifToggles();
  }

  let deleteModalOpen = $state(false);
  let deletePassword = $state('');
  let deleteError = $state('');
  let deleteBusy = $state(false);

  async function handleDeleteVault() {
    if (!deletePassword) {
      deleteError = i18n.t.deleteMasterPasswordError;
      return;
    }
    deleteBusy = true;
    deleteError = '';
    try {
      const currentVaultName = store.appState?.vault_name;
      const currentActiveId = getActiveVaultId();
      const state = await api.deleteVaultAndAccount(deletePassword);
      store.appState = state;

      if (currentActiveId) {
        await forgetVault(currentActiveId);
      }
      if (currentVaultName) {
        await forgetVault(currentVaultName);
      }

      const remaining = getKnownVaults();
      if (remaining.length > 0) {
        setActiveVault(remaining[0].id);
        window.location.href = `/login?vault=${encodeURIComponent(remaining[0].id)}`;
      } else {
        clearVaultRegistry();
        window.location.href = '/setup';
      }
    } catch (e: any) {
      deleteError = e.message;
    } finally {
      deleteBusy = false;
    }
  }

  // Backup & Export Functions
  function exportVaultBackupJson() {
    if (!ledger.data) return;
    const jsonStr = JSON.stringify(ledger.data, null, 2);
    const blob = new Blob([jsonStr], { type: 'application/json;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `finnca-vault-backup-${todayString()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function exportTransactionsCsv() {
    if (!ledger.data) return;
    const rows = [['Date', 'Description', 'Currency', 'Account Code', 'Account Name', 'Amount']];
    for (const tx of ledger.transactions) {
      for (const split of tx.splits) {
        const acc = ledger.accountsById.get(split.accountId);
        rows.push([
          tx.date,
          `"${tx.description.replace(/"/g, '""')}"`,
          tx.currency,
          acc?.code ?? '',
          `"${acc?.name?.replace(/"/g, '""') ?? ''}"`,
          fromMinor(tx.currency, split.amount).toString(),
        ]);
      }
    }
    const csvContent = rows.map((r) => r.join(',')).join('\n');
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `finnca-transactions-${todayString()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }
  async function handleEncryptedBackup() {
    try {
      const dest = await api.pickDirectory();
      if (!dest) return;
      await api.createVaultBackup(dest);
      alert('Encrypted Backup successfully created at ' + dest);
    } catch (e: any) {
      alert('Backup failed: ' + e.message);
    }
  }
</script>

<div class="grid grid-cols-1 gap-2 select-none lg:grid-cols-2">
  <!-- LEFT COLUMN: BACKUP & EXPORT + DANGER ZONE -->
  <div class="flex flex-col gap-2">
    <!-- 1. BACKUP & EXPORT CARD -->
    <Card title={i18n.t.backupExportTitle} badge="STORAGE & ARCHIVE" class="justify-between">
      <div>
        <p class="text-text-dim mb-3 font-mono text-[11px] leading-relaxed">
          {i18n.t.backupExportDesc}
        </p>

        <div class="flex flex-col gap-2.5">
          <button
            type="button"
            onclick={handleEncryptedBackup}
            class="sharp-btn btn-primary font-proto flex items-center justify-center gap-2 p-2.5 text-[10px]"
          >
            <span>{i18n.t.backupEncryptedVaultBtn}</span>
          </button>

          <div class="grid grid-cols-2 gap-2.5">
            <button
              type="button"
              onclick={exportVaultBackupJson}
              class="sharp-btn btn-ghost font-proto border-line hover:border-text-dim flex items-center justify-center gap-2 p-2.5 text-[9px]"
              title={i18n.t.exportPlaintextJsonTitle}
            >
              <span>{i18n.t.exportPlaintextJsonBtn}</span>
            </button>

            <button
              type="button"
              onclick={exportTransactionsCsv}
              class="sharp-btn btn-ghost font-proto border-line hover:border-text-dim flex items-center justify-center gap-2 p-2.5 text-[10px]"
            >
              <span>{i18n.t.exportCsvBtn}</span>
            </button>
          </div>
        </div>
      </div>

      <p class="text-text-dim border-line/30 mt-auto border-t pt-2 font-mono text-[9px]">
        {i18n.t.backupEncryptedNote}
      </p>
    </Card>

    <!-- 2. DANGER ZONE CARD -->
    <Card
      title={i18n.t.dangerZoneTitle}
      badge="DESTRUCTIVE"
      badgeTone="err"
      class="border-expense/40 bg-expense/5 justify-between"
    >
      <div>
        <p class="text-text-dim mb-3 font-mono text-[10px] leading-relaxed">
          {i18n.t.deleteVaultWarning}
        </p>
      </div>

      <button
        type="button"
        onclick={() => (deleteModalOpen = true)}
        class="sharp-btn btn-danger font-proto self-start px-4 py-2 text-[10px]"
      >
        {i18n.t.deleteVaultBtn}
      </button>
    </Card>
  </div>

  <!-- RIGHT COLUMN: NOTIFICATIONS PREFERENCES CARD -->
  <Card title={i18n.t.notificationsTitle} badge="ALERTS" class="justify-between">
    <div>
      <div class="flex flex-col gap-2">
        <label
          class="border-line/40 bg-bg-app hover:border-text-dim group flex cursor-pointer items-center justify-between border p-2.5 transition-colors"
        >
          <div class="flex items-center gap-2.5">
            <input
              type="checkbox"
              checked={notifToggles.due}
              onchange={() => toggle('due')}
              class="accent-teal size-3.5"
            />
            <span
              class="font-proto text-text-strong group-hover:text-text-white text-[11px] transition-colors"
              >{i18n.t.notifDueDatesLabel}</span
            >
          </div>
          <span
            class="font-proto text-[9px] {notifToggles.due
              ? 'text-teal font-bold'
              : 'text-text-muted'}"
          >
            {notifToggles.due ? i18n.t.enabledLabel : i18n.t.disabledLabel}
          </span>
        </label>

        <label
          class="border-line/40 bg-bg-app hover:border-text-dim group flex cursor-pointer items-center justify-between border p-2.5 transition-colors"
        >
          <div class="flex items-center gap-2.5">
            <input
              type="checkbox"
              checked={notifToggles.fx}
              onchange={() => toggle('fx')}
              class="accent-teal size-3.5"
            />
            <span
              class="font-proto text-text-strong group-hover:text-text-white text-[11px] transition-colors"
              >{i18n.t.notifFxAlertsLabel}</span
            >
          </div>
          <span
            class="font-proto text-[9px] {notifToggles.fx
              ? 'text-teal font-bold'
              : 'text-text-muted'}"
          >
            {notifToggles.fx ? i18n.t.enabledLabel : i18n.t.disabledLabel}
          </span>
        </label>

        <label
          class="border-line/40 bg-bg-app hover:border-text-dim group flex cursor-pointer items-center justify-between border p-2.5 transition-colors"
        >
          <div class="flex items-center gap-2.5">
            <input
              type="checkbox"
              checked={notifToggles.spike}
              onchange={() => toggle('spike')}
              class="accent-teal size-3.5"
            />
            <span
              class="font-proto text-text-strong group-hover:text-text-white text-[11px] transition-colors"
              >{i18n.t.notifExpenseSpikeLabel}</span
            >
          </div>
          <span
            class="font-proto text-[9px] {notifToggles.spike
              ? 'text-teal font-bold'
              : 'text-text-muted'}"
          >
            {notifToggles.spike ? i18n.t.enabledLabel : i18n.t.disabledLabel}
          </span>
        </label>

        <label
          class="border-line/40 bg-bg-app hover:border-text-dim group flex cursor-pointer items-center justify-between border p-2.5 transition-colors"
        >
          <div class="flex items-center gap-2.5">
            <input
              type="checkbox"
              checked={notifToggles.integrity}
              onchange={() => toggle('integrity')}
              class="accent-teal size-3.5"
            />
            <span
              class="font-proto text-text-strong group-hover:text-text-white text-[11px] transition-colors"
              >{i18n.t.notifLedgerIntegrityLabel}</span
            >
          </div>
          <span
            class="font-proto text-[9px] {notifToggles.integrity
              ? 'text-teal font-bold'
              : 'text-text-muted'}"
          >
            {notifToggles.integrity ? i18n.t.enabledLabel : i18n.t.disabledLabel}
          </span>
        </label>
      </div>
    </div>

    <p class="text-text-muted font-proto border-line/40 mt-auto border-t pt-2 text-[9px]">
      {i18n.t.notifStoredLocally}
    </p>
  </Card>
</div>

{#if deleteModalOpen}
  <ModalShell
    bind:open={deleteModalOpen}
    title={i18n.t.confirmVaultDeletionTitle}
    onClose={() => {
      deleteModalOpen = false;
      deletePassword = '';
      deleteError = '';
    }}
  >
    <p class="text-text-base py-3 font-mono text-[12px] leading-relaxed">
      {i18n.t.enterPasswordToConfirm}
    </p>
    <div class="mt-2 mb-6">
      <input
        type="password"
        bind:value={deletePassword}
        placeholder={i18n.t.enterMasterPassword}
        class="sharp-input w-full px-3 py-2 text-center font-mono"
      />
      {#if deleteError}<p class="badge-err font-proto mt-2 px-2.5 py-1 text-center text-[11px]">
          {deleteError}
        </p>{/if}
    </div>
    <div class="border-line flex justify-end gap-2 border-t pt-3">
      <button
        type="button"
        class="sharp-btn btn-ghost font-proto px-4 py-2 text-[11px]"
        onclick={() => {
          deleteModalOpen = false;
          deletePassword = '';
          deleteError = '';
        }}
        disabled={deleteBusy}
      >
        {i18n.t.cancelBtn}
      </button>
      <button
        type="button"
        class="sharp-btn btn-danger font-proto px-4 py-2 text-[11px]"
        onclick={handleDeleteVault}
        disabled={deleteBusy}
      >
        {deleteBusy ? i18n.t.processingBtn : i18n.t.deleteVaultBtn}
      </button>
    </div>
  </ModalShell>
{/if}

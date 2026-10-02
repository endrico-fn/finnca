<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { session } from '$lib/core/state/session.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { Button, Badge } from '$lib/components/ui';
  import { diagnoseVaultHealthCmd, type VaultHealthReport } from '$lib/core/ipc/bindings';
  import {
    exportVaultBackupJson,
    exportCsvTransactions,
    exportEncryptedVault,
    exportBeancountLedger,
  } from '../state/exportBackupUtils';
  import DeleteVaultModal from './DeleteVaultModal.svelte';

  let deleteModalOpen = $state(false);
  let diagnosing = $state(false);
  let healthReport = $state<VaultHealthReport | null>(null);

  async function handleRunDiagnostics() {
    diagnosing = true;
    try {
      healthReport = await diagnoseVaultHealthCmd();
    } catch (e: unknown) {
      healthReport = {
        is_healthy: false,
        sqlite_integrity: String(e),
        foreign_keys_ok: false,
        unbalanced_entries_count: 0,
        orphan_postings_count: 0,
        placeholder_postings_count: 0,
        issues: [e instanceof Error ? e.message : String(e)],
      };
    } finally {
      diagnosing = false;
    }
  }

  async function handleExportJson() {
    try {
      const dest = await exportVaultBackupJson(session.currentVault);
      if (dest) {
        notificationState.addNotification({
          type: 'INFO',
          priority: 'low',
          title: i18n.t.backupExportTitle,
          message: i18n.t.backupCreatedMsg.replace('{dest}', dest),
        });
      }
    } catch (e: unknown) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.backupExportTitle,
        message: e instanceof Error ? e.message : String(e),
      });
    }
  }

  async function handleExportCsv() {
    try {
      const dest = await exportCsvTransactions();
      if (dest) {
        notificationState.addNotification({
          type: 'INFO',
          priority: 'low',
          title: i18n.t.exportCsvBtn,
          message: i18n.t.exportSuccess ?? 'CSV file exported successfully',
        });
      }
    } catch (e: unknown) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.exportCsvBtn,
        message: e instanceof Error ? e.message : String(e),
      });
    }
  }

  async function handleExportBeancount() {
    try {
      const dest = await exportBeancountLedger();
      if (dest) {
        notificationState.addNotification({
          type: 'INFO',
          priority: 'low',
          title: i18n.t.exportBeancountBtn,
          message: i18n.t.exportBeancountSuccess.replace('{dest}', dest),
        });
      }
    } catch (e: unknown) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.exportBeancountBtn,
        message: e instanceof Error ? e.message : String(e),
      });
    }
  }

  async function handleEncryptedBackup() {
    try {
      const dest = await exportEncryptedVault();
      if (!dest) return;
      notificationState.addNotification({
        type: 'INFO',
        priority: 'low',
        title: i18n.t.backupEncryptedVaultBtn,
        message: i18n.t.backupCreatedMsg.replace('{dest}', dest),
      });
    } catch (e: unknown) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.backupEncryptedVaultBtn,
        message: i18n.t.backupFailedMsg.replace(
          '{msg}',
          e instanceof Error ? e.message : String(e)
        ),
      });
    }
  }
</script>

<section class="border-line bg-bg-app flex flex-col border p-4 sm:p-6 select-none max-w-4xl">
  <div class="flex flex-col border-b border-line pb-6 mb-6 last:border-0 last:mb-0 last:pb-0">
    <div class="flex items-start justify-between mb-4">
      <div class="flex flex-col gap-1">
        <h3 class="font-proto text-text-strong text-small tracking-widest uppercase">{i18n.t.backupExportTitle}</h3>
        <p class="text-text-dim text-smaller font-aux">
          {i18n.t.backupEncryptedNote}
        </p>
      </div>
      <Badge size="m" tone="neutral">{i18n.t.badgeStorageArchive}</Badge>
    </div>
    <div>
      <p class="text-text-dim text-small font-aux mb-3 leading-relaxed">
        {i18n.t.backupExportDesc}
      </p>

      <div class="flex flex-col gap-2">
        <Button variant="primary" onclick={handleEncryptedBackup}>
          <span>{i18n.t.backupEncryptedVaultBtn}</span>
        </Button>

        <div class="grid grid-cols-2 gap-2">
          <Button
            variant="ghost"
            onclick={handleExportJson}
            title={i18n.t.exportPlaintextJsonTitle}
          >
            <span>{i18n.t.exportPlaintextJsonBtn}</span>
          </Button>

          <Button variant="ghost" onclick={handleExportCsv}>
            <span>{i18n.t.exportCsvBtn}</span>
          </Button>
        </div>

        <Button
          variant="tactical"
          onclick={handleExportBeancount}
          title={i18n.t.exportBeancountTitle}
        >
          <span>{i18n.t.exportBeancountBtn}</span>
        </Button>
      </div>
    </div>
  </div>

  <div class="flex flex-col pb-6">
    <div class="flex items-start justify-between mb-4">
      <div class="flex flex-col gap-1">
        <h3 class="font-proto text-text-strong text-small tracking-widest uppercase">{i18n.t.vaultHealthDiagnosticsTitle}</h3>
      </div>
      <Badge size="m" tone="neutral">SQLITE & LEDGER</Badge>
    </div>
    <div>
      <p class="text-text-dim text-small font-aux mb-3 leading-relaxed">
        {i18n.t.vaultHealthDiagnosticsDesc}
      </p>

      <Button variant="tactical" disabled={diagnosing} onclick={handleRunDiagnostics}>
        {diagnosing ? '...' : i18n.t.runDiagnosticsBtn}
      </Button>

      {#if healthReport}
        <div
          class="font-proto text-smaller mt-3 border p-2.5 {healthReport.is_healthy
            ? 'border-income/40 bg-income/5'
            : 'border-expense/40 bg-expense/5'}"
        >
          <div class="mb-1.5 flex items-center justify-between">
            <span class="font-bold {healthReport.is_healthy ? 'text-income' : 'text-expense'}">
              {healthReport.is_healthy
                ? i18n.t.vaultHealthHealthy
                : i18n.t.vaultHealthIssuesFound}
            </span>
            <Badge size="s" tone={healthReport.is_healthy ? 'ok' : 'err'}>
              SQLite: {healthReport.sqlite_integrity}
            </Badge>
          </div>
          {#if healthReport.issues.length > 0}
            <ul class="text-expense text-smaller mt-2 list-inside list-disc space-y-1">
              {#each healthReport.issues as issue, idx (idx)}
                <li class="truncate">{issue}</li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <div class="flex flex-col pt-6 mt-6 border-t border-expense/40">
    <div class="flex items-start justify-between mb-4">
      <div class="flex flex-col gap-1">
        <h3 class="font-proto text-expense text-small tracking-widest uppercase">{i18n.t.dangerZoneTitle}</h3>
      </div>
      <Badge size="m" tone="err">{i18n.t.badgeDestructive}</Badge>
    </div>
    <div>
      <p class="text-text-dim text-smaller font-aux mb-3 leading-relaxed">
        {i18n.t.deleteVaultWarning}
      </p>

      <Button variant="danger" onclick={() => (deleteModalOpen = true)}>
        {i18n.t.deleteVaultBtn}
      </Button>
    </div>
  </div>
</section>

<DeleteVaultModal bind:open={deleteModalOpen} onClose={() => (deleteModalOpen = false)} />

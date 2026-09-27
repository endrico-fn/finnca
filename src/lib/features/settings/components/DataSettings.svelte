<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { session } from '$lib/core/state/session.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { Card, Button, Badge } from '$lib/components/ui';
  import { diagnoseVaultHealthCmd, type VaultHealthReport } from '$lib/core/ipc/bindings';
  import {
    exportVaultBackupJson,
    exportCsvTransactions,
    exportEncryptedVault,
    exportBeancountLedger,
  } from '../state/exportBackupUtils';
  import NotificationSettings from './NotificationSettings.svelte';
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

<div class="grid grid-cols-1 gap-2 select-none lg:grid-cols-2">
  <div class="flex flex-col gap-2">
    <Card
      title={i18n.t.backupExportTitle}
      badge={i18n.t.badgeStorageArchive}
      class="justify-between"
    >
      <div>
        <p class="text-text-dim text-small font-aux mb-3 leading-relaxed">
          {i18n.t.backupExportDesc}
        </p>

        <div class="flex flex-col gap-2.5">
          <Button variant="primary" onclick={handleEncryptedBackup}>
            <span>{i18n.t.backupEncryptedVaultBtn}</span>
          </Button>

          <div class="grid grid-cols-2 gap-2.5">
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

      <p class="text-text-dim border-line/30 text-smaller font-aux mt-auto border-t pt-2">
        {i18n.t.backupEncryptedNote}
      </p>
    </Card>

    <Card
      title={i18n.t.dangerZoneTitle}
      badge={i18n.t.badgeDestructive}
      badgeTone="err"
      class="border-expense/40 bg-expense/5 justify-between"
    >
      <div>
        <p class="text-text-dim text-smaller font-aux mb-3 leading-relaxed">
          {i18n.t.deleteVaultWarning}
        </p>
      </div>

      <Button variant="danger" onclick={() => (deleteModalOpen = true)}>
        {i18n.t.deleteVaultBtn}
      </Button>
    </Card>
  </div>

  <div class="flex flex-col gap-2">
    <NotificationSettings />

    <Card
      title={i18n.t.vaultHealthDiagnosticsTitle}
      badge="SQLITE & LEDGER"
      class="justify-between"
    >
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
    </Card>
  </div>
</div>

<DeleteVaultModal bind:open={deleteModalOpen} onClose={() => (deleteModalOpen = false)} />

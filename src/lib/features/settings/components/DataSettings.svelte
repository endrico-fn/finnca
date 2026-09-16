<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { session } from '$lib/core/state/session.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { Card, Button } from '$lib/components/ui';
  import {
    exportVaultBackupJson,
    exportCsvTransactions,
    exportEncryptedVault,
  } from '../state/exportBackupUtils';
  import NotificationSettings from './NotificationSettings.svelte';
  import DeleteVaultModal from './DeleteVaultModal.svelte';

  let deleteModalOpen = $state(false);

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
        message: i18n.t.backupFailedMsg.replace('{msg}', e instanceof Error ? e.message : String(e)),
      });
    }
  }
</script>

<div class="grid grid-cols-1 gap-2 select-none lg:grid-cols-2">
  <!-- LEFT COLUMN: BACKUP & EXPORT + DANGER ZONE -->
  <div class="flex flex-col gap-2">
    <!-- 1. BACKUP & EXPORT CARD -->
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
        </div>
      </div>

      <p class="text-text-dim border-line/30 text-smaller font-aux mt-auto border-t pt-2">
        {i18n.t.backupEncryptedNote}
      </p>
    </Card>

    <!-- 2. DANGER ZONE CARD -->
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

  <!-- RIGHT COLUMN: NOTIFICATIONS PREFERENCES CARD -->
  <NotificationSettings />
</div>

<DeleteVaultModal bind:open={deleteModalOpen} onClose={() => (deleteModalOpen = false)} />

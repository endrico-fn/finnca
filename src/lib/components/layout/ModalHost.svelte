<script lang="ts">
  import { modalState } from '$lib/core/state/modal.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { journalState } from '$lib/features/journal/state/journalDraft.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import CommandPalette from '$lib/components/layout/CommandPalette.svelte';
  import HealthPulseModal, {
    type HealthStats,
  } from '$lib/components/layout/HealthPulseModal.svelte';
  import ConfirmDialog from '$lib/components/feedback/ConfirmDialog.svelte';
  import EntryInspector from '$lib/features/journal/components/EntryInspector.svelte';
  import FloatingInspectorWindow from '$lib/components/layout/FloatingInspectorWindow.svelte';

  let {
    healthStats = null,
    onLock,
  }: {
    healthStats?: HealthStats | null;
    onLock: () => void | Promise<void>;
  } = $props();
</script>

<CommandPalette
  bind:open={modalState.commandPaletteOpen}
  onQuickTxDraft={(draft) => modalState.openQuickTx(draft)}
  onTransfer={() => modalState.openTransfer()}
  {onLock}
/>

{#if healthStats}
  <HealthPulseModal
    bind:open={modalState.healthPulseOpen}
    {healthStats}
    onFixInJournal={() => {
      modalState.closeHealthPulse();
      goto(resolve('/app/journal'));
    }}
  />
{/if}

{#if modalState.inspectorOpen}
  <FloatingInspectorWindow
    open={true}
    title={modalState.inspectorEntry ? i18n.t.editEntryTitle : modalState.inspectorMode === 'transfer' ? i18n.t.quickTransferTitle : i18n.t.newEntryTitle}
    onClose={() => modalState.closeInspector()}
  >
    <EntryInspector
      entry={modalState.inspectorEntry}
      initialDraft={modalState.inspectorDraft}
      initialMode={modalState.inspectorMode}
      initialFrom={modalState.inspectorInitialFrom}
      initialTo={modalState.inspectorInitialTo}
      onSave={async (savedTx) => {
        await journalState.saveEntry(savedTx);
        notificationState.addNotification({
          type: 'LEDGER_INTEGRITY',
          priority: 'low',
          title: i18n.t.quickTxSavedNotifTitle,
          message: i18n.t.quickTxSavedNotifMsg,
        });
        modalState.closeInspector();
      }}
      onCancel={() => modalState.closeInspector()}
      onDelete={async (id) => {
        await journalState.deleteEntry(id);
        modalState.closeInspector();
      }}
    />
  </FloatingInspectorWindow>
{/if}

{#if modalState.confirmConfig}
  <ConfirmDialog
    open={true}
    title={modalState.confirmConfig.title}
    message={modalState.confirmConfig.message}
    confirmLabel={modalState.confirmConfig.confirmLabel}
    cancelLabel={modalState.confirmConfig.cancelLabel}
    danger={modalState.confirmConfig.danger}
    onConfirm={async () => {
      const fn = modalState.confirmConfig?.onConfirm;
      modalState.confirmConfig = null;
      if (fn) await fn();
    }}
    onCancel={() => modalState.closeConfirm()}
  />
{/if}

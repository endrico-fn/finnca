<script lang="ts">
  import { modalState } from '$lib/core/state/modal.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { journalState } from '$lib/features/journal/state/journalDraft.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { ModalShell } from '$lib/components/ui';
  import CommandPalette from '$lib/components/layout/CommandPalette.svelte';
  import HealthPulseModal, {
    type HealthStats,
  } from '$lib/components/layout/HealthPulseModal.svelte';
  import ConfirmDialog from '$lib/components/feedback/ConfirmDialog.svelte';
  import JournalEntryForm from '$lib/features/journal/components/JournalEntryForm.svelte';
  import TransferModal from '$lib/features/journal/components/TransferModal.svelte';

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

{#if modalState.quickTxOpen}
  <ModalShell
    bind:open={modalState.quickTxOpen}
    title={modalState.quickTxIsNew ? i18n.t.newTransactionTitle : i18n.t.quickTxModalTitle}
    maxWidth="max-w-5xl"
    onClose={() => modalState.closeQuickTx()}
  >
    <svelte:boundary>
      {#if modalState.quickTxDraft}
        <JournalEntryForm
          tx={modalState.quickTxIsNew ? null : modalState.quickTxDraft}
          initialDraft={modalState.quickTxIsNew ? modalState.quickTxDraft : undefined}
          onSave={async (savedTx) => {
            await journalState.saveTransaction(savedTx);
            notificationState.addNotification({
              type: 'LEDGER_INTEGRITY',
              priority: 'low',
              title: i18n.t.quickTxSavedNotifTitle,
              message: i18n.t.quickTxSavedNotifMsg,
            });
            modalState.closeQuickTx();
          }}
          onCancel={() => modalState.closeQuickTx()}
        />
      {/if}
      {#snippet failed(err, reset)}
        <div class="badge-err font-proto text-small mt-2 px-3 py-2">
          {i18n.t.quickTxLoadFailed}
        </div>
        <pre class="font-proto text-smaller text-text-muted mt-2 overflow-x-auto px-1 py-2 wrap-break-word whitespace-pre-wrap">{err instanceof Error ? (err.stack ?? err.message) : String(err)}</pre>
        <div class="mt-2 flex justify-end">
          <button
            type="button"
            onclick={reset}
            class="sharp-btn btn-ghost font-proto text-small h-8 cursor-pointer px-3.5"
          >
            {i18n.t.commonRetry}
          </button>
        </div>
      {/snippet}
    </svelte:boundary>
  </ModalShell>
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

{#if modalState.transferModalOpen}
  <TransferModal
    bind:open={modalState.transferModalOpen}
  />
{/if}

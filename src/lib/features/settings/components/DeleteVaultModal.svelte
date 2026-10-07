<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { session } from '$lib/core/state/session.svelte';
  import {
    deleteVaultAndAccount,
    forgetKnownVaultCmd,
    getKnownVaultsCmd,
    setActiveVault,
  } from '$lib/core/ipc/bindings';
  import { ModalShell, Button } from '$lib/components/ui';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  let {
    open = $bindable(false),
    onClose,
  }: {
    open: boolean;
    onClose: () => void;
  } = $props();

  let deletePassword = $state('');
  let deleteError = $state('');
  let deleteBusy = $state(false);

  function closeDeleteModal() {
    deletePassword = '';
    deleteError = '';
    onClose();
  }

  async function executePermanentVaultDeletion() {
    if (!deletePassword) {
      deleteError = i18n.t.deleteMasterPasswordError;
      return;
    }
    deleteBusy = true;
    deleteError = '';
    try {
      const currentVaultName = session.currentVault;
      const currentVaultPath = session.vaultPath;
      const state = await deleteVaultAndAccount(deletePassword);
      session.raw = state;

      if (currentVaultPath) {
        await forgetKnownVaultCmd(currentVaultPath).catch(() => {});
      }
      if (currentVaultName) {
        await forgetKnownVaultCmd(currentVaultName).catch(() => {});
      }

      const remaining = await getKnownVaultsCmd().catch(() => []);
      if (remaining.length > 0) {
        await setActiveVault(remaining[0].path).catch(() => {});
      }

      eventBus.emit('vault:deleted', undefined);
      eventBus.emit('vault:registry_changed', undefined);

      open = false;
      deletePassword = '';
      await goto(resolve('/'));
    } catch (e: unknown) {
      deleteError = e instanceof Error ? e.message : String(e);
    } finally {
      deleteBusy = false;
    }
  }
</script>

{#if open}
  <ModalShell
    bind:open
    title={i18n.t.confirmVaultDeletionTitle}
    tone="err"
    onClose={closeDeleteModal}
  >
    <p class="text-text-base text-small font-aux py-3 leading-relaxed">
      {i18n.t.enterPasswordToConfirm}
    </p>
    <div class="mt-2 mb-6">
      <input
        type="password"
        bind:value={deletePassword}
        placeholder={i18n.t.enterMasterPassword}
        class="sharp-input w-full px-3 py-2 text-center"
      />
      {#if deleteError}
        <p class="badge-err font-proto text-small mt-2 px-2.5 py-1 text-center">
          {deleteError}
        </p>
      {/if}
    </div>
    <div class="border-line mt-3 flex shrink-0 justify-end gap-2 border-t pt-3">
      <Button variant="ghost" onclick={closeDeleteModal} disabled={deleteBusy}>
        {i18n.t.cancelBtn}
      </Button>
      <Button variant="danger" onclick={executePermanentVaultDeletion} disabled={deleteBusy}>
        {deleteBusy ? i18n.t.processingBtn : i18n.t.deleteVaultBtn}
      </Button>
    </div>
  </ModalShell>
{/if}

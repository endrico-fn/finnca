<script lang="ts">
  import { session } from '$lib/core/state/session.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { renameVault, openVaultFolder } from '$lib/core/ipc/bindings';
  import { Card, Button } from '$lib/components/ui';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { APP_NAME } from '$lib/core/types';

  let vaultName = $state(session.currentVault || '');
  let vaultNameError = $state('');
  let vaultNameSaving = $state(false);
  let vaultNameSuccess = $state('');

  const isDirty = $derived(vaultName.trim() !== '' && vaultName.trim() !== (session.currentVault || ''));
  const canSave = $derived(!vaultNameSaving && isDirty);

  async function handleSaveVaultName() {
    if (!vaultName.trim()) {
      vaultNameError = i18n.t.vaultNameEmpty;
      return;
    }
    vaultNameSaving = true;
    vaultNameError = '';
    vaultNameSuccess = '';
    try {
      const state = await renameVault(vaultName.trim());
      session.raw = state;
      eventBus.emit('vault:registry_changed', undefined);
      vaultNameSuccess = i18n.t.vaultRenamedOk;
      setTimeout(() => (vaultNameSuccess = ''), 3000);
    } catch (e: unknown) {
      vaultNameError = e instanceof Error ? e.message : String(e);
    } finally {
      vaultNameSaving = false;
    }
  }

  async function handleOpenVaultFolder() {
    try {
      await openVaultFolder();
    } catch (e: unknown) {
      console.error(e);
    }
  }
</script>

<Card title={i18n.t.vaultIdentity} badge={i18n.t.badgeVault} class="justify-between">
  <div class="flex flex-col gap-2.5">
    <div>
      <label for="vault-name-input" class="label-xs text-text-muted mb-1 block">
        {i18n.t.vaultName}
      </label>
      <input
        id="vault-name-input"
        type="text"
        class="sharp-input text-small w-full px-2.5 py-1.5"
        bind:value={vaultName}
        onkeydown={(e) => e.key === 'Enter' && canSave && handleSaveVaultName()}
      />
    </div>

    <div>
      <p class="label-xs text-text-muted mb-1 block">
        {i18n.t.vaultPathLabel}
      </p>
      <div class="flex items-center gap-2">
        <div
          class="sharp-input text-text-muted bg-bg-app border-line text-smaller font-aux flex-1 truncate border px-2.5 py-1.5"
        >
          {session.currentVault
            ? `${APP_NAME.toLowerCase()}-${session.currentVault.toLowerCase().replace(/\s+/g, '-')}`
            : `${APP_NAME.toLowerCase()}-vault`}
        </div>
        <Button variant="ghost" onclick={handleOpenVaultFolder}>
          {i18n.t.openFolderBtn}
        </Button>
      </div>
    </div>

    <div class="border-line/40 mt-1 flex items-center justify-between gap-2 border-t pt-2.5">
      <div class="min-h-5 flex items-center">
        {#if vaultNameError}
          <p class="text-expense font-proto text-smaller">{vaultNameError}</p>
        {:else if vaultNameSuccess}
          <p class="text-income font-proto text-smaller">{vaultNameSuccess}</p>
        {/if}
      </div>
      <Button
        variant="primary"
        class="font-proto text-small h-8 px-3 font-bold"
        disabled={!canSave}
        onclick={handleSaveVaultName}
      >
        {i18n.t.saveChanges}
      </Button>
    </div>
  </div>

  <p class="text-text-dim border-line/30 text-smaller font-aux mt-auto border-t pt-2">
    {i18n.t.vaultStorageNote}
  </p>
</Card>

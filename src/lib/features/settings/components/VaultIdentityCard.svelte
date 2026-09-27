<script lang="ts">
  import { session } from '$lib/core/state/session.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { openVaultFolder } from '$lib/core/ipc/bindings';
  import { Card, Icon } from '$lib/components/ui';
  import { APP_SLUG } from '$lib/core/types';
  import { generalSettingsState } from '../state/settings.svelte';

  const vaultLocationDisplay = $derived(
    session.vaultPath ||
      (session.currentVault
        ? `${APP_SLUG}-${session.currentVault.toLowerCase().replace(/\s+/g, '-')}`
        : `${APP_SLUG}-vault`)
  );

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
        class="sharp-input text-small h-8 w-full px-2.5"
        bind:value={generalSettingsState.vaultName}
        onkeydown={(e) =>
          e.key === 'Enter' && generalSettingsState.canSave && generalSettingsState.save()}
      />
    </div>

    <div>
      <p class="label-xs text-text-muted mb-1 block">
        {i18n.t.vaultPathLabel}
      </p>
      <div class="border-line bg-bg-app flex h-8 w-full items-center justify-between border px-2.5">
        <div class="flex min-w-0 items-center gap-2">
          <span class="text-teal inline-flex shrink-0">
            <Icon name="folder" size={13} />
          </span>
          <span class="text-text-muted font-aux text-smaller truncate" title={vaultLocationDisplay}>
            {vaultLocationDisplay}
          </span>
        </div>
        <button
          type="button"
          onclick={handleOpenVaultFolder}
          class="sharp-btn btn-ghost font-proto text-text-muted hover:text-text-strong text-smaller ml-2 inline-flex shrink-0 cursor-pointer items-center gap-1 px-2 py-0.5 uppercase transition-colors"
          title={i18n.t.openFolderBtn}
        >
          <span>{i18n.t.openFolderBtn}</span>
        </button>
      </div>
    </div>
  </div>

  <p class="text-text-dim border-line/30 text-smaller font-aux mt-auto border-t pt-2">
    {i18n.t.vaultStorageNote}
  </p>
</Card>

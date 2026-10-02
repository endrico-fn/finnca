<script lang="ts">
  import { session } from '$lib/core/state/session.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { openVaultFolder } from '$lib/core/ipc/bindings';
  import { Icon, Badge } from '$lib/components/ui';
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

<div class="border-line mb-6 flex flex-col border-b pb-6 last:mb-0 last:border-0 last:pb-0">
  <div class="mb-4 flex items-start justify-between">
    <div class="flex flex-col gap-1">
      <h3 class="font-proto text-text-strong text-small tracking-widest uppercase">
        {i18n.t.vaultIdentity}
      </h3>
      <p class="text-text-dim text-smaller font-aux">
        {i18n.t.vaultStorageNote}
      </p>
    </div>
    <Badge size="m" tone="neutral">{i18n.t.badgeVault}</Badge>
  </div>
  <div class="flex flex-col gap-3 sm:flex-row sm:gap-4">
    <div class="flex-1">
      <label
        for="vault-name-input"
        class="font-proto text-text-muted text-smaller mb-1 block tracking-widest uppercase"
      >
        {i18n.t.vaultName}
      </label>
      <input
        id="vault-name-input"
        type="text"
        class="sharp-input font-proto text-small h-8 w-full px-2.5"
        bind:value={generalSettingsState.vaultName}
        onkeydown={(e) =>
          e.key === 'Enter' && generalSettingsState.canSave && generalSettingsState.save()}
      />
    </div>

    <div class="flex-1">
      <p class="font-proto text-text-muted text-smaller mb-1 block tracking-widest uppercase">
        {i18n.t.vaultPathLabel}
      </p>
      <div class="border-line bg-bg-app flex h-8 w-full items-center justify-between border px-2.5">
        <div class="flex min-w-0 items-center gap-2">
          <span class="text-teal inline-flex shrink-0">
            <Icon name="folder" size={13} />
          </span>
          <span
            class="font-proto text-text-strong text-small truncate tracking-wider uppercase"
            title={vaultLocationDisplay}
          >
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
</div>

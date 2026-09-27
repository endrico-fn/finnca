<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { Icon } from '$lib/components/ui';
  import type { KnownVault } from '$lib/features/vault/state/vaultList.svelte';

  let {
    knownVaults,
    selectedVaultId = $bindable<string | null>(null),
    activeVaultPath,
    onSelectVault,
    onNavigateSetup,
  }: {
    knownVaults: KnownVault[];
    selectedVaultId?: string | null;
    activeVaultPath?: string;
    onSelectVault: (path: string) => void;
    onNavigateSetup: () => void;
  } = $props();
</script>

{#if knownVaults.length > 0}
  <div>
    <div class="mb-2 flex items-center justify-between">
      <span class="text-text-dim font-proto text-smaller block tracking-wider uppercase">
        {i18n.t.savedVaultsLabel}
      </span>
      <button
        type="button"
        onclick={onNavigateSetup}
        class="text-teal font-proto text-smaller inline-flex cursor-pointer items-center gap-1 hover:underline"
      >
        <Icon name="plus" size={9} />
        {i18n.t.addVaultBtn}
      </button>
    </div>

    <div class="max-h-48 space-y-1.5 overflow-x-hidden overflow-y-auto">
      {#each knownVaults as v, idx (v.id || v)}
        {@const isSel = v.path === (selectedVaultId ?? activeVaultPath)}
        <button
          type="button"
          onclick={() => {
            selectedVaultId = v.path;
            onSelectVault(v.path);
          }}
          class="font-proto flex w-full cursor-pointer items-stretch border text-left transition-colors {isSel
            ? 'border-teal/60 bg-bg-row-active text-text-strong'
            : 'border-line/60 bg-bg-card hover:border-line hover:bg-bg-app text-text-muted hover:text-text-base'}"
        >
          <div
            class="font-proto text-largest grid aspect-square w-12 shrink-0 place-items-center border-r transition-colors {isSel
              ? 'border-teal/50 bg-teal/15 text-teal'
              : 'border-line/60 bg-bg-app text-text-muted'}"
          >
            <span class="leading-none tracking-tight tabular-nums">
              {String(idx + 1).padStart(2, '0')}
            </span>
          </div>

          <div class="flex min-w-0 flex-1 flex-col justify-center px-2.5 py-1">
            <span
              class="text-small block truncate font-medium {isSel
                ? 'text-teal font-bold'
                : 'text-text-strong'}"
            >
              {v.name}
            </span>
            <span class="text-smaller text-text-dim block truncate font-normal">
              {v.username ? `@${v.username}` : i18n.t.vaultUnknownUser}
            </span>
          </div>

          {#if isSel}
            <div class="flex items-center pr-2">
              <span class="text-income inline-flex">
                <Icon name="check" size={11} />
              </span>
            </div>
          {/if}
        </button>
      {/each}
    </div>
  </div>
{:else}
  <div>
    <span class="text-text-dim text-smaller block tracking-wider uppercase">{i18n.t.vault}</span>
    <span class="text-income font-medium">{i18n.t.vaultDefaultName}</span>
  </div>
  <div class="border-line border-t pt-2">
    <button
      type="button"
      onclick={onNavigateSetup}
      class="text-teal font-proto text-smaller inline-flex cursor-pointer items-center gap-1 hover:underline"
    >
      <Icon name="plus" size={10} />
      {i18n.t.addVaultBtn}
    </button>
  </div>
{/if}

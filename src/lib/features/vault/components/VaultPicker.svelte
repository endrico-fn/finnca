<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { session } from '$lib/core/state/session.svelte';
  import { Icon, Button } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import {
    getKnownVaults,
    getActiveVaultId,
    setActiveVault,
    rememberVault,
    syncVaultsFromBackend,
    type KnownVault,
  } from '$lib/features/vault/state/vaultList.svelte';

  let open = $state(false);
  let containerEl: HTMLElement | null = $state(null);

  const activeId = $derived(getActiveVaultId());
  const vaults = $derived(getKnownVaults());
  const activeVault = $derived(
    vaults.find((v) => v.path === activeId) ??
      vaults.find((v) => v.name === session.raw?.vault_name) ??
      vaults[0] ??
      null
  );

  function isCurrentActive(v: KnownVault): boolean {
    if (activeId && v.path === activeId) return true;
    if (activeVault && v.path === activeVault.path) return true;
    if (
      session.raw?.vault_name &&
      v.name === session.raw.vault_name &&
      (!v.username || v.username === session.raw.username)
    ) {
      return true;
    }
    return false;
  }

  function handleClickOutside(event: MouseEvent) {
    if (containerEl && !containerEl.contains(event.target as Node)) {
      open = false;
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape' && open) {
      open = false;
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);

    syncVaultsFromBackend().then(() => {
      if (session.raw?.vault_path && session.raw.vault_name) {
        const currentExists = vaults.some((v) => v.path === session.raw?.vault_path);
        if (!currentExists) {
          rememberVault({
            id: session.raw.vault_path,
            name: session.raw.vault_name,
            path: session.raw.vault_path,
            username: session.raw.username ?? i18n.t.vaultUnknownUser,
          });
        }
      }
    });

    return () => {
      document.removeEventListener('click', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  });

  async function switchVault(target: KnownVault) {
    if (isCurrentActive(target)) {
      open = false;
      return;
    }
    open = false;
    setActiveVault(target.path);
    await session.lock();
    goto(resolve(`/login?vault=${encodeURIComponent(target.path)}` as '/login'));
  }

  function toggleDropdown() {
    open = !open;
    if (open) {
      syncVaultsFromBackend();
    }
  }

  function handleAddVault() {
    open = false;
    goto(resolve('/setup?from=app'));
  }
</script>

<div bind:this={containerEl} class="relative w-full" role="group">
  <button
    type="button"
    onclick={toggleDropdown}
    class="group flex w-full cursor-pointer items-center gap-2.5 border p-1.5 text-left transition-all select-none {open
      ? 'bg-bg-card border-line'
      : 'hover:bg-bg-card/70 hover:border-line/40 border-transparent'}"
    aria-haspopup="listbox"
    aria-expanded={open}
  >
    <!-- User & Vault Details -->
    <div class="flex min-w-0 flex-1 flex-col gap-0.5">
      <div
        class="text-text-strong font-aux group-hover:text-teal text-small truncate leading-tight font-medium transition-colors"
      >
        {session.raw?.username ?? activeVault?.username ?? i18n.t.vaultUnknownUser}
      </div>
      <div
        class="text-text-muted font-proto text-smaller mt-1 flex items-center gap-1.5 truncate leading-tight"
      >
        <span class="bg-income size-1.5 shrink-0"></span>
        <span class="truncate">
          {session.raw?.vault_name ?? activeVault?.name ?? i18n.t.vaultDefaultName}
        </span>
      </div>
    </div>

    <!-- Dropdown Chevron Indicator -->
    <div
      class="text-text-muted ml-0.5 inline-flex shrink-0 transition-transform duration-150 {open
        ? 'text-teal rotate-180'
        : 'group-hover:text-text-strong'}"
    >
      <Icon name="chev-down" size={10} />
    </div>
  </button>

  {#if open}
    <div
      class="sharp-card border-line bg-bg-card anim-enter absolute top-full right-0 left-0 z-50 mt-1 border p-2"
      role="listbox"
    >
      <!-- Popover Header -->
      <div class="border-line mb-1.5 flex items-start justify-between border-b px-2 py-1.5">
        <span class="font-proto text-text-dim text-smaller font-semibold tracking-widest uppercase">
          {i18n.t.vault}
        </span>
        <span
          class="font-proto text-income text-smaller flex items-center gap-1 tracking-wider uppercase"
        >
          <span class="bg-income size-1.5 animate-pulse"></span>
          {i18n.t.vaultEncrypted}
        </span>
      </div>

      <!-- Vault List -->
      <div class="max-h-48 space-y-1 overflow-y-auto pr-0.5">
        {#each vaults as v (v.id || v)}
          {@const isActive = isCurrentActive(v)}
          <button
            type="button"
            onclick={() => switchVault(v)}
            title={isActive ? i18n.t.currentActiveVault : i18n.t.switchToVault}
            class="font-proto flex w-full cursor-pointer items-center justify-between border px-2.5 py-2 text-left transition-colors {isActive
              ? 'bg-bg-row-active border-teal/60 text-text-strong'
              : 'bg-bg-card hover:bg-bg-row-active text-text-base border-line/50 hover:border-line'}"
          >
            <div class="min-w-0 flex-1 pr-2">
              <div
                class="text-small truncate leading-tight font-medium {isActive
                  ? 'text-income'
                  : 'text-text-strong'}"
              >
                {v.name}
              </div>
              <div class="text-text-muted font-proto text-smaller mt-1 truncate leading-tight">
                {v.username}
              </div>
            </div>
            {#if isActive}
              <span class="text-income ml-1 inline-flex shrink-0 items-center">
                <Icon name="check" size={11} />
              </span>
            {/if}
          </button>
        {:else}
          <p class="text-text-muted font-proto text-smaller px-2 py-2">
            {i18n.t.noSavedVaults}
          </p>
        {/each}
      </div>

      <!-- Action Footer -->
      <div class="border-line mt-1.5 border-t pt-1.5">
        <Button variant="ghost" size="sm" onclick={handleAddVault} class="w-full gap-1.5">
          <Icon name="plus" size={10} />
          {i18n.t.addVaultBtn}
        </Button>
      </div>
    </div>
  {/if}
</div>

<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import { unlockVault, importVault } from '$lib/core/ipc/bindings';
  import { extractErrorMessage } from '$lib/core/ipc/errors';
  import { session } from '$lib/core/state/session.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { setPref } from '$lib/core/state/prefs';
  import { Button, Icon } from '$lib/components/ui';
  import {
    getKnownVaults,
    getActiveVaultId,
    rememberVault,
    setActiveVault,
    forgetVault,
    clearVaultRegistry,
    syncVaultsFromBackend,
    type KnownVault,
  } from '$lib/features/vault/state/vaultList.svelte';
  import { onMount } from 'svelte';
  import type { AppStateView } from '$lib/core/types';
  import VaultBrandHeader from './VaultBrandHeader.svelte';
  import VaultSlotList from './VaultSlotList.svelte';
  import VaultImportView from './VaultImportView.svelte';

  let {
    initialVaultId = null,
    startInImport = false,
    onSuccess,
    onNavigateSetup,
  }: {
    initialVaultId?: string | null;
    startInImport?: boolean;
    onSuccess?: (st: AppStateView) => void;
    onNavigateSetup?: () => void;
  } = $props();

  let password = $state('');
  let showPassword = $state(false);
  let busy = $state(false);
  let error = $state('');
  let showImport = $state(false);
  let isCapsLock = $state(false);
  let passwordInputEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (activeVault && !showImport && !busy) {
      passwordInputEl?.focus();
    }
  });

  function checkCapsLock(e: KeyboardEvent) {
    if (typeof e.getModifierState === 'function') {
      isCapsLock = e.getModifierState('CapsLock');
    }
  }

  let knownVaults = $state<KnownVault[]>([]);
  let selectedVaultId = $state<string | null>(null);

  $effect(() => {
    if (startInImport) showImport = true;
  });

  $effect(() => {
    if (initialVaultId) selectedVaultId = initialVaultId;
  });

  const activeVault = $derived.by(() => {
    if (selectedVaultId) {
      return knownVaults.find((v) => v.path === selectedVaultId) ?? null;
    }
    const currentActiveId = getActiveVaultId();
    return knownVaults.find((v) => v.path === currentActiveId) ?? knownVaults[0] ?? null;
  });

  onMount(async () => {
    knownVaults = await syncVaultsFromBackend();
    try {
      await session.refresh();
    } catch (e) {
      console.error('Failed to refresh app state:', e);
    }

    if (session.raw && !session.raw.configured) {
      knownVaults = knownVaults.filter((v) => Boolean(v.path && v.path.length > 0));
      setPref('finnca_known_vaults', knownVaults);
      if (knownVaults.length === 0) {
        clearVaultRegistry();
        navigateToVaultSetup();
        return;
      }
    }

    const queryVault = page.url.searchParams.get('vault');
    if (queryVault) {
      selectedVaultId = queryVault;
      setActiveVault(queryVault);
    } else if (!selectedVaultId) {
      selectedVaultId = getActiveVaultId();
    }
    if (page.url.searchParams.get('add') === '1') showImport = true;
    if (session.raw?.configured && session.raw.vault_name && session.raw.vault_path) {
      const vName = session.raw.vault_name;
      rememberVault({
        id: vName,
        name: vName,
        path: session.raw.vault_path,
        username: session.raw.username ?? '',
      });
      knownVaults = getKnownVaults();
    }
  });

  function navigateToVaultSetup() {
    if (onNavigateSetup) {
      onNavigateSetup();
    } else {
      goto(resolve('/setup?from=login'), { replaceState: true });
    }
  }

  async function authenticateVault() {
    if (!password) return;
    if (password.length < 8) {
      error = i18n.t.passwordTooShort;
      return;
    }
    error = '';
    busy = true;
    try {
      if (activeVault?.path) {
        session.raw = await importVault(activeVault.path, password);
      } else {
        session.raw = await unlockVault(password);
      }
      if (session.raw?.vault_name) {
        const id = activeVault?.id ?? session.raw.vault_name;
        rememberVault({
          id,
          name: session.raw.vault_name,
          path: session.raw.vault_path ?? activeVault?.path ?? '',
          username: session.raw.username ?? '',
        });
      }
      if (onSuccess && session.raw) {
        onSuccess(session.raw);
      } else {
        goto(resolve('/app'));
      }
    } catch (e) {
      const msg = extractErrorMessage(e);
      if (msg.toLowerCase().includes('at least 8 characters')) {
        error = i18n.t.passwordTooShort;
      } else {
        error = msg;
      }
      if (error.includes('START FROM SETUP') || error.includes('not found')) {
        await session.refresh();
        if (selectedVaultId) {
          await forgetVault(selectedVaultId);
        }
        knownVaults = getKnownVaults();
        if (knownVaults.length === 0) {
          clearVaultRegistry();
          navigateToVaultSetup();
        } else {
          selectedVaultId = knownVaults[0].id;
        }
      }
    } finally {
      busy = false;
      password = '';
    }
  }
</script>

<div
  class="border-line bg-bg-card relative max-h-[calc(100dvh-2rem)] w-full max-w-4xl overflow-y-auto border"
>
  <div class="grid min-h-125 grid-cols-12 items-stretch">
    <div class="bg-bg-app border-line col-span-5 flex flex-col justify-between border-r p-7">
      <div>
        <VaultBrandHeader tagline={i18n.t.loginTagline} />

        <div class="border-line bg-bg-card/60 text-small space-y-2.5 border p-3.5">
          <VaultSlotList
            {knownVaults}
            bind:selectedVaultId
            activeVaultPath={activeVault?.path}
            onSelectVault={(path) => setActiveVault(path)}
            onNavigateSetup={navigateToVaultSetup}
          />

          <div class="border-line border-t pt-2">
            <span class="text-text-dim text-smaller font-proto block tracking-wider uppercase">
              {i18n.t.encryptionScheme}
            </span>
            <span class="text-text-base text-smaller font-proto"
              >Argon2id · SQLCipher AES-256-GCM</span
            >
          </div>
        </div>
      </div>

      <div class="pt-6">
        <p class="text-text-muted border-line text-smaller border-l pl-3 leading-relaxed">
          {i18n.t.offlineGuarantee}
        </p>
      </div>
    </div>

    <div class="bg-bg-card col-span-7 flex flex-col justify-between p-7">
      {#if showImport}
        <VaultImportView
          onSuccess={(st) => {
            knownVaults = getKnownVaults();
            if (onSuccess) onSuccess(st);
            else goto(resolve('/app'));
          }}
          onCancel={() => {
            showImport = false;
            error = '';
          }}
        />
      {:else}
        <div class="anim-fade-fast flex flex-1 flex-col justify-between">
          <div class="space-y-6">
            <div>
              <h2
                class="text-text-strong font-proto text-medium font-medium tracking-wide uppercase"
              >
                {i18n.t.unlockVaultTitle}: {activeVault?.name ??
                  session.raw?.vault_name ??
                  i18n.t.vault}
              </h2>
              <p class="text-text-dim text-small mt-1">
                {i18n.t.unlockVaultDesc}
              </p>
            </div>

            <form
              onsubmit={(e) => {
                e.preventDefault();
                authenticateVault();
              }}
              class="space-y-4"
            >
              <div>
                <span class="label-xs text-text-base mb-1.5 block">
                  {i18n.t.masterPasswordLabel}
                </span>
                <div class="relative flex items-center">
                  <!-- svelte-ignore a11y_autofocus -->
                  <input
                    bind:this={passwordInputEl}
                    bind:value={password}
                    type={showPassword ? 'text' : 'password'}
                    autofocus
                    autocomplete="current-password"
                    onkeydown={checkCapsLock}
                    onkeyup={checkCapsLock}
                    class="sharp-input text-small h-8 w-full pr-8 pl-2.5"
                    placeholder={i18n.t.enterMasterPassword}
                  />
                  <button
                    type="button"
                    onclick={() => (showPassword = !showPassword)}
                    class="text-text-muted hover:text-text-strong absolute right-1.5 flex h-6 w-6 cursor-pointer items-center justify-center transition-colors"
                    title={showPassword ? i18n.t.hidePassword : i18n.t.showPassword}
                    aria-label={showPassword ? i18n.t.hidePassword : i18n.t.showPassword}
                  >
                    <Icon name={showPassword ? 'eye-off' : 'eye'} size={13} />
                  </button>
                </div>
                {#if isCapsLock}
                  <span
                    class="text-warning font-proto text-smaller mt-1 block font-bold tracking-wider"
                  >
                    {i18n.t.capsLockActive}
                  </span>
                {/if}
                {#if password.length > 0 && password.length < 8}
                  <span class="text-warning font-proto text-smaller mt-1 block">
                    {i18n.t.passwordTooShort} ({password.length}/8)
                  </span>
                {/if}
              </div>

              {#if error}
                <div class="badge-err font-proto text-small px-3 py-2 tracking-wide">
                  {error}
                </div>
              {/if}

              <Button
                type="submit"
                variant="primary"
                size="md"
                class="w-full justify-center"
                loading={busy}
                disabled={busy || password.length < 8}
              >
                <span class="font-proto text-small font-bold tracking-wider">
                  {busy ? i18n.t.decryptingVault : i18n.t.unlockVaultBtn}
                </span>
              </Button>
            </form>
          </div>

          <div class="border-line mt-6 flex items-center justify-between border-t pt-4">
            <Button type="button" variant="ghost" onclick={navigateToVaultSetup}>
              <span class="font-proto text-small inline-flex items-center gap-1.5">
                <Icon name="plus" size={11} />
                {i18n.t.addVaultBtn}
              </span>
            </Button>
            <Button
              type="button"
              variant="ghost"
              onclick={() => {
                showImport = true;
                error = '';
              }}
            >
              <span class="font-proto text-teal text-small inline-flex items-center gap-1.5">
                <Icon name="wallet" size={12} />
                {i18n.t.importVault}
              </span>
            </Button>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

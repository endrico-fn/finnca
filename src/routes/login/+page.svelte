<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import * as api from '$lib/api';
  import { store } from '$lib/stores/app-store.svelte';
  import { i18n } from '$lib/i18n.svelte';
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
  } from '$lib/stores/vault-registry.svelte';
  import { onMount } from 'svelte';
  import { APP_NAME, version_app } from '$lib/types';

  let password = $state('');
  let busy = $state(false);
  let error = $state('');
  let showImport = $state(false);

  let importPath = $state('');
  let importPass = $state('');
  let knownVaults = $state<KnownVault[]>([]);
  let selectedVaultId = $state<string | null>(null);

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
      await store.refresh();
    } catch (e) {
      console.error('Failed to refresh app state:', e);
    }

    // Auto-detect deleted vault: if appState is unconfigured, purge stale local vaults
    if (store.appState && !store.appState.configured) {
      knownVaults = knownVaults.filter((v) => Boolean(v.path && v.path.length > 0));
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('finnca_known_vaults', JSON.stringify(knownVaults));
      }
      if (knownVaults.length === 0) {
        clearVaultRegistry();
        goto(resolve('/setup'), { replaceState: true });
        return;
      }
    }

    const queryVault = page.url.searchParams.get('vault');
    if (queryVault) {
      selectedVaultId = queryVault;
      setActiveVault(queryVault);
    } else {
      selectedVaultId = getActiveVaultId();
    }
    if (page.url.searchParams.get('add') === '1') showImport = true;
    if (store.appState?.configured && store.appState.vault_name && store.appState.vault_path) {
      const vName = store.appState.vault_name;
      rememberVault({
        id: vName,
        name: vName,
        path: store.appState.vault_path,
        username: store.appState.username ?? '',
      });
      knownVaults = getKnownVaults();
    }
  });

  async function submit() {
    if (!password) return;
    error = '';
    busy = true;
    try {
      if (activeVault?.path) {
        store.appState = await api.importVault(activeVault.path, password);
      } else {
        store.appState = await api.unlockVault(password);
      }
      if (store.appState?.vault_name) {
        const id = activeVault?.id ?? store.appState.vault_name;
        rememberVault({
          id,
          name: store.appState.vault_name,
          path: store.appState.vault_path ?? activeVault?.path ?? '',
          username: store.appState.username ?? '',
        });
      }
      goto(resolve('/app'));
    } catch (e) {
      error = String(e).replace('Error: ', '');
      if (error.includes('START FROM SETUP') || error.includes('not found')) {
        await store.refresh();
        if (selectedVaultId) {
          await forgetVault(selectedVaultId);
        }
        knownVaults = getKnownVaults();
        if (knownVaults.length === 0) {
          clearVaultRegistry();
          goto(resolve('/setup'), { replaceState: true });
        } else {
          selectedVaultId = knownVaults[0].id;
        }
      }
    } finally {
      busy = false;
      password = '';
    }
  }

  async function pickImportFolder() {
    const dir = await api.pickDirectory();
    if (dir) importPath = dir;
  }

  async function doImport() {
    error = '';
    if (!importPath) {
      error = i18n.t.selectVaultFolderRequired;
      return;
    }
    if (importPass.length < 8) {
      error = i18n.t.passwordMin8;
      return;
    }
    busy = true;
    try {
      const st = await api.importVault(importPath, importPass);
      store.appState = st;
      if (st.vault_name) {
        const name = st.vault_name;
        rememberVault({
          id: `${name}@${importPath}`,
          name,
          path: importPath,
          username: st.username ?? '',
        });
        knownVaults = getKnownVaults();
      }
      goto(resolve('/app'));
    } catch (e) {
      error = String(e).replace('Error: ', '');
    } finally {
      busy = false;
    }
  }
</script>

<div class="bg-bg-app grid min-h-screen place-items-center p-6 font-mono select-none">
  <div class="border-line bg-bg-card anim-enter relative w-full max-w-4xl overflow-hidden border">
    <div class="grid min-h-135 grid-cols-12">
      <div class="bg-bg-app border-line col-span-5 flex flex-col justify-between border-r p-8">
        <div>
          <div class="mb-6 flex items-center gap-3">
            <div
              class="border-line bg-bg-card anim-pulse-soft grid size-10 place-items-center border"
            >
              <span class="text-teal text-large">◈</span>
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h1 class="text-text-strong text-large font-medium tracking-[0.25em]">
                  {APP_NAME}
                </h1>
                <span
                  class="font-proto text-text-dim border-line bg-bg-card text-smaller border px-1.5 py-0.5"
                >
                  v{version_app}
                </span>
              </div>
              <p class="text-text-dim text-smaller mt-0.5 tracking-wider">
                {i18n.t.loginTagline}
              </p>
            </div>
          </div>

          <div class="border-line bg-bg-card/60 text-small space-y-2.5 border p-4">
            <div>
              <span class="text-text-dim text-smaller block tracking-wider uppercase"
                >{i18n.t.vault}</span
              >
              <span class="text-income font-medium"
                >{activeVault?.name ??
                  store.appState?.vault_name ??
                  (knownVaults.length > 0 ? knownVaults[0].name : i18n.t.vaultDefaultName)}</span
              >
            </div>
            {#if activeVault?.username || store.appState?.username}
              <div>
                <span class="text-text-dim text-smaller block tracking-wider uppercase"
                  >{i18n.t.userAccount}</span
                >
                <span class="text-text-strong"
                  >{activeVault?.username ?? store.appState?.username}</span
                >
              </div>
            {/if}
            {#if knownVaults.length > 0}
              <div class="border-line space-y-1.5 border-t pt-2">
                <div class="flex items-center justify-between">
                  <span
                    class="text-text-dim font-proto text-smaller block tracking-wider uppercase"
                  >
                    {i18n.t.savedVaultsLabel}
                  </span>
                  <a
                    href={resolve('/setup?from=login')}
                    class="text-teal font-proto text-smaller inline-flex items-center gap-1 hover:underline"
                  >
                    <Icon name="plus" size={9} />
                    {i18n.t.addVaultBtn}
                  </a>
                </div>
                <div class="max-h-36 space-y-1 overflow-y-auto pr-1">
                  {#each knownVaults as v (v.id || v)}
                    <button
                      type="button"
                      onclick={() => {
                        selectedVaultId = v.path;
                        setActiveVault(v.path);
                      }}
                      class="font-proto text-small flex w-full cursor-pointer items-center justify-between border px-2 py-1.5 text-left transition-colors {v.path ===
                      (selectedVaultId ?? activeVault?.path)
                        ? 'border-teal/50 bg-bg-row-active text-income'
                        : 'border-line/60 bg-bg-card hover:border-line text-text-base'}"
                    >
                      <span class="truncate">{v.name} · {v.username}</span>
                      {#if v.path === (selectedVaultId ?? activeVault?.path)}
                        <span class="text-income text-smaller ml-1">✓</span>
                      {/if}
                    </button>
                  {/each}
                </div>
              </div>
            {:else}
              <div class="border-line border-t pt-2">
                <a
                  href={resolve('/setup?from=login')}
                  class="text-teal font-proto text-smaller inline-flex items-center gap-1 hover:underline"
                >
                  <Icon name="plus" size={10} />
                  {i18n.t.addVaultBtn}
                </a>
              </div>
            {/if}
            <div class="border-line border-t pt-2">
              <span class="text-text-dim text-smaller font-proto block tracking-wider uppercase"
                >{i18n.t.encryptionScheme}</span
              >
              <span class="text-text-base text-smaller font-proto">AGE • X25519</span>
            </div>
          </div>
        </div>

        <div class="pt-6">
          <p class="text-text-muted border-line text-smaller border-l pl-3 leading-relaxed">
            {i18n.t.offlineGuarantee}
          </p>
        </div>
      </div>

      <div class="bg-bg-card col-span-7 flex flex-col justify-between overflow-y-auto p-8">
        {#if !showImport}
          <div class="space-y-6">
            <div>
              <h2
                class="text-text-strong font-proto text-medium font-medium tracking-wide uppercase"
              >
                {i18n.t.unlockVaultTitle}: {activeVault?.name ??
                  store.appState?.vault_name ??
                  i18n.t.vault}
              </h2>
              <p class="text-text-dim text-small mt-1">
                {i18n.t.unlockVaultDesc}
              </p>
            </div>

            <form
              onsubmit={(e) => {
                e.preventDefault();
                submit();
              }}
              class="space-y-4"
            >
              <div>
                <span class="label-xs text-text-base mb-1.5 block"
                  >{i18n.t.masterPasswordLabel}</span
                >
                <!-- svelte-ignore a11y_autofocus -->
                <input
                  bind:value={password}
                  type="password"
                  autofocus
                  class="sharp-input text-small w-full px-3.5 py-2.5 tracking-widest placeholder:tracking-normal"
                  placeholder={i18n.t.enterMasterPassword}
                />
              </div>

              {#if error}
                <div class="badge-err text-small px-3 py-2 tracking-wide">
                  {error}
                </div>
              {/if}

              <Button type="submit" variant="primary" disabled={busy || !password}>
                <span class="text-small w-full font-medium tracking-wider">
                  {busy ? i18n.t.decryptingVault : i18n.t.unlockVaultBtn}
                </span>
              </Button>
            </form>
          </div>

          <div class="border-line flex items-center justify-between border-t pt-6">
            <Button type="button" variant="ghost" href="/setup?from=login">
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
        {:else}
          <div class="anim-enter space-y-4">
            <div class="border-line flex items-start justify-between border-b pb-2">
              <h2
                class="text-text-strong font-proto text-medium font-medium tracking-wide uppercase"
              >
                {i18n.t.importVaultTitle}
              </h2>
              <button
                type="button"
                onclick={() => {
                  showImport = false;
                  error = '';
                }}
                class="text-text-base hover:text-text-strong font-proto text-smaller inline-flex cursor-pointer items-center gap-1"
              >
                <Icon name="close" size={11} />
                {i18n.t.cancelBtn}
              </button>
            </div>

            <div>
              <span class="label-xs text-text-base font-proto mb-1.5 block"
                >{i18n.t.oldVaultFolder}</span
              >
              <button
                type="button"
                onclick={pickImportFolder}
                class="sharp-input hover:border-teal/40 text-small flex w-full cursor-pointer items-center justify-between px-3.5 py-2.5"
              >
                <span class="truncate {importPath ? 'text-text-strong' : 'text-text-dim'}">
                  {importPath || i18n.t.chooseDirectory}
                </span>
                <span class="text-teal ml-2 inline-flex">
                  <Icon name="wallet" size={13} />
                </span>
              </button>
            </div>

            <div>
              <span class="label-xs text-text-base font-proto mb-1.5 block"
                >{i18n.t.legacyVaultPassword}</span
              >
              <input
                bind:value={importPass}
                type="password"
                class="sharp-input text-small w-full px-3.5 py-2.5"
                placeholder={i18n.t.passwordMin8}
              />
              <p class="text-text-muted font-proto text-smaller mt-1 leading-relaxed">
                {i18n.t.legacyVaultPasswordHint}
              </p>
            </div>

            {#if error}
              <div class="badge-err font-proto text-small px-3 py-2 tracking-wide">
                {error}
              </div>
            {/if}

            <p class="text-text-muted font-proto text-smaller pt-2 text-center">
              {i18n.t.folderMustContainVaultKey}
            </p>
          </div>

          <div class="border-line flex items-center justify-between border-t pt-6">
            <Button
              type="button"
              variant="ghost"
              onclick={() => {
                showImport = false;
                error = '';
              }}
            >
              <span class="font-proto">{i18n.t.cancelBtn}</span>
            </Button>
            <Button variant="primary" onclick={doImport} disabled={busy}>
              <span class="font-proto text-small font-bold tracking-wider">
                {busy ? i18n.t.importingBtn : i18n.t.importVaultBtn}
              </span>
            </Button>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import * as api from '$lib/api';
  import { i18n } from '$lib/i18n.svelte';
  import { Button, Icon } from '$lib/components/ui';
  import { store } from '$lib/stores/app-store.svelte';
  import { onMount } from 'svelte';
  import {
    getKnownVaults,
    rememberVault,
    setActiveVault,
    syncVaultsFromBackend,
  } from '$lib/stores/vault-registry.svelte';
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { APP_NAME, version_app } from '$lib/types';

  const fromSource = $derived(page.url.searchParams.get('from'));
  const knownVaults = $derived(getKnownVaults());

  onMount(async () => {
    await syncVaultsFromBackend();
  });

  let step = $state(0);
  let slideDirection = $state(1);
  let vaultName = $state('');
  let vaultPath = $state('');
  let username = $state('');
  let password = $state('');
  let confirm = $state('');
  let busy = $state(false);
  let error = $state('');

  let showImport = $state(false);
  let importPath = $state('');
  let importPass = $state('');

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
        setActiveVault(`${name}@${importPath}`);
      }
      goto('/app');
    } catch (e) {
      error = String(e).replace('Error: ', '');
    } finally {
      busy = false;
    }
  }

  const slug = $derived(
    vaultName
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '') || 'vault'
  );
  const targetFolderPreview = $derived(
    vaultPath ? `${vaultPath.replace(/\/+$/, '')}/finnca-${slug}` : ''
  );

  async function pickFolder() {
    const dir = await api.pickDirectory();
    if (dir) vaultPath = dir;
  }

  async function next() {
    error = '';
    busy = true;
    try {
      if (step === 0) {
        if (!vaultName.trim()) {
          throw new Error(i18n.t.vaultNameRequired);
        }
        if (!vaultPath) {
          throw new Error(i18n.t.vaultPathRequired);
        }
        await api.createVault(vaultName.trim(), vaultPath);
        slideDirection = 1;
        step = 1;
      } else {
        if (!username.trim()) {
          throw new Error(i18n.t.usernameRequired);
        }
        if (password.length < 8) {
          throw new Error(i18n.t.passwordMin8);
        }
        if (password !== confirm) {
          throw new Error(i18n.t.passwordMismatch);
        }
        const st = await api.createAccount(username.trim(), password);
        store.appState = st;
        const targetDir = vaultPath ? `${vaultPath.replace(/\/+$/, '')}/finnca-${slug}` : '';
        if (st.vault_name) {
          rememberVault({
            id: st.vault_name,
            name: st.vault_name,
            path: targetDir,
            username: st.username ?? username.trim(),
          });
          setActiveVault(st.vault_name);
        }
        goto('/app');
      }
    } catch (e) {
      error = String(e).replace('Error: ', '');
    } finally {
      busy = false;
    }
  }
</script>

<div class="bg-bg-app grid min-h-screen place-items-center p-6 font-mono select-none">
  <div
    class="border-line bg-bg-card anim-enter relative w-full max-w-4xl overflow-hidden border"
  >
    <div class="grid min-h-135 grid-cols-12">
      <!-- Left Column: Branding & Steps Summary -->
      <div class="bg-bg-app border-line col-span-5 flex flex-col justify-between border-r p-8">
        <div>
          <div class="mb-6 flex items-center gap-3">
            <div
              class="border-line bg-bg-card anim-pulse-soft grid size-10 place-items-center border"
            >
              <span class="text-teal text-xl">◈</span>
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h1 class="text-text-strong text-[18px] font-medium tracking-[0.25em]">
                  {APP_NAME}
                </h1>
                <span
                  class="font-proto text-text-dim border-line bg-bg-card border px-1.5 py-0.5 text-[10px]"
                >
                  v{version_app}
                </span>
              </div>
              <p class="text-text-dim mt-0.5 text-[10px] tracking-wider">
                {i18n.t.setupTagline}
              </p>
            </div>
          </div>

          <div class="border-line bg-bg-card/60 space-y-3 border p-4 text-[11px]">
            <div>
              <span class="text-text-dim block text-[10px] tracking-wider uppercase">
                {i18n.t.setupStagesLabel}
              </span>
            </div>
            <div class="space-y-2">
              {#if showImport}
                <div class="flex items-center gap-2.5">
                  <span
                    class="bg-teal text-bg-app grid size-5 place-items-center rounded-none text-[10px] font-bold"
                  >
                    ✓
                  </span>
                  <span class="text-text-strong font-medium">
                    {i18n.t.importVaultTitle}
                  </span>
                </div>
              {:else}
                <div class="flex items-center gap-2.5">
                  <span
                    class="grid size-5 place-items-center rounded-none text-[10px] font-bold {step ===
                    0
                      ? 'bg-teal text-bg-app'
                      : 'border-line bg-bg-card text-text-dim border'}"
                  >
                    1
                  </span>
                  <span class={step === 0 ? 'text-text-strong font-medium' : 'text-text-dim'}>
                    {i18n.t.stepVault}
                  </span>
                </div>
                <div class="flex items-center gap-2.5">
                  <span
                    class="grid size-5 place-items-center rounded-none text-[10px] font-bold {step ===
                    1
                      ? 'bg-teal text-bg-app'
                      : 'border-line bg-bg-card text-text-dim border'}"
                  >
                    2
                  </span>
                  <span class={step === 1 ? 'text-text-strong font-medium' : 'text-text-dim'}>
                    {i18n.t.stepAccount}
                  </span>
                </div>
              {/if}
            </div>

            <div class="border-line border-t pt-3">
              <span class="text-text-dim block text-[10px] tracking-wider uppercase">
                {i18n.t.encryptionScheme}
              </span>
              <span class="text-text-base text-[10px]">AGE • X25519</span>
            </div>
          </div>
        </div>

        <div class="pt-6">
          <p class="text-text-muted border-line border-l-2 pl-3 text-[10px] leading-relaxed">
            {i18n.t.setupSubtitle}
          </p>
        </div>
      </div>

      <!-- Right Column: Form Wizard -->
      <div class="bg-bg-card col-span-7 flex flex-col justify-between overflow-y-auto p-8">
        {#if showImport}
          <div class="anim-enter space-y-4">
            <div class="border-line flex items-start justify-between border-b pb-2">
              <h2
                class="text-text-strong font-proto text-[14px] font-medium tracking-wide uppercase"
              >
                {i18n.t.importVaultTitle}
              </h2>
              <button
                type="button"
                onclick={() => {
                  showImport = false;
                  error = '';
                }}
                class="text-text-icon hover:text-text-strong font-proto inline-flex cursor-pointer items-center gap-1 text-[10px]"
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
                class="sharp-input hover:border-teal/40 flex w-full cursor-pointer items-center justify-between px-3.5 py-2.5 text-[12px]"
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
                class="sharp-input w-full px-3.5 py-2.5 text-[12px]"
                placeholder={i18n.t.passwordMin8}
              />
              <p class="text-text-muted font-proto mt-1 text-[10px] leading-relaxed">
                {i18n.t.legacyVaultPasswordHint}
              </p>
            </div>

            {#if error}
              <div class="badge-err font-proto px-3 py-2 text-[11px] tracking-wide">
                {error}
              </div>
            {/if}

            <p class="text-text-muted font-proto pt-2 text-center text-[10px]">
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
              <span class="font-proto text-[11px] font-bold tracking-wider">
                {busy ? i18n.t.importingBtn : i18n.t.importVaultBtn}
              </span>
            </Button>
          </div>
        {:else}
          <div class="overflow-hidden">
            {#key step}
              <div
                in:fly={{
                  x: 32 * slideDirection,
                  duration: 260,
                  easing: cubicOut,
                  opacity: 0,
                }}
                class="space-y-5"
              >
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <h2
                      class="text-text-strong font-proto text-[15px] font-medium tracking-wide uppercase"
                    >
                      {step === 0 ? i18n.t.stepVault : i18n.t.stepAccount}
                    </h2>
                    <p class="text-text-dim font-proto mt-1 text-[11px]">
                      {step === 0 ? i18n.t.vaultNamePlaceholder : i18n.t.passwordHint}
                    </p>
                  </div>

                  {#if fromSource === 'app'}
                    <button
                      type="button"
                      onclick={() => goto('/app')}
                      class="sharp-btn btn-ghost font-proto text-text-muted hover:text-text-strong inline-flex shrink-0 cursor-pointer items-center gap-1.5 px-2.5 py-1 text-[10px]"
                    >
                      <Icon name="chev-left" size={12} />
                      <span>{i18n.t.backToApp}</span>
                    </button>
                  {:else if fromSource === 'login' || knownVaults.length > 0}
                    <button
                      type="button"
                      onclick={() => goto('/login')}
                      class="sharp-btn btn-ghost font-proto text-text-muted hover:text-text-strong inline-flex shrink-0 cursor-pointer items-center gap-1.5 px-2.5 py-1 text-[10px]"
                    >
                      <Icon name="chev-left" size={12} />
                      <span>{i18n.t.backToLogin}</span>
                    </button>
                  {/if}
                </div>

                {#if step === 0}
                  <!-- Step 1: Vault -->
                  <div class="space-y-4">
                    <div>
                      <span class="label-xs text-text-base font-proto mb-1.5 block"
                        >{i18n.t.vaultNameLabel}</span
                      >
                      <input
                        bind:value={vaultName}
                        class="sharp-input w-full px-3.5 py-2.5 text-[12px] font-mono"
                        placeholder={i18n.t.vaultNamePlaceholder}
                      />
                    </div>

                    <div>
                      <span class="label-xs text-text-base font-proto mb-1.5 block"
                        >{i18n.t.storageLocation}</span
                      >
                      <button
                        type="button"
                        onclick={pickFolder}
                        class="sharp-input hover:border-teal/40 flex w-full cursor-pointer items-center justify-between px-3.5 py-2.5 text-[12px]"
                      >
                        <span class="truncate {vaultPath ? 'text-text-strong' : 'text-text-dim'}">
                          {vaultPath || i18n.t.chooseDirectory}
                        </span>
                        <span class="text-teal ml-2 shrink-0">⬡</span>
                      </button>
                    </div>

                    {#if targetFolderPreview}
                      <div
                        class="border-line bg-bg-app font-proto text-text-base border px-3.5 py-2.5 text-[11px]"
                      >
                        <span class="text-text-icon mb-0.5 block text-[10px]"
                          >{i18n.t.targetVaultFolder}:</span
                        >
                        <span class="text-income break-all">{targetFolderPreview}</span>
                      </div>
                    {/if}

                    <p
                      class="border-line bg-bg-app text-text-icon font-proto border p-3 text-[10px] leading-relaxed"
                    >
                      {i18n.t.vaultStorageDesc}
                    </p>
                  </div>
                {:else}
                  <!-- Step 2: Account -->
                  <div class="space-y-4">
                    <div>
                      <span class="label-xs text-text-base font-proto mb-1.5 block"
                        >{i18n.t.usernameLabel}</span
                      >
                      <input
                        bind:value={username}
                        class="sharp-input w-full px-3.5 py-2.5 text-[12px] font-mono"
                        placeholder={i18n.t.usernameInputPlaceholder}
                      />
                    </div>

                    <div>
                      <span class="label-xs text-text-base font-proto mb-1.5 block"
                        >{i18n.t.masterPasswordLabel}</span
                      >
                      <input
                        bind:value={password}
                        type="password"
                        class="sharp-input w-full px-3.5 py-2.5 text-[12px]"
                        placeholder={i18n.t.passwordMin8}
                      />
                    </div>

                    <div>
                      <span class="label-xs text-text-base font-proto mb-1.5 block"
                        >{i18n.t.confirmPasswordLabel}</span
                      >
                      <input
                        bind:value={confirm}
                        type="password"
                        class="sharp-input w-full px-3.5 py-2.5 text-[12px]"
                        placeholder={i18n.t.reEnterPasswordPlaceholder}
                      />
                    </div>
                  </div>
                {/if}

                <!-- Error -->
                {#if error}
                  <div class="badge-err font-proto px-3 py-2 text-[11px] tracking-wide">
                    {error}
                  </div>
                {/if}
              </div>
            {/key}
          </div>

          <div class="border-line flex items-center justify-between border-t pt-6">
            <span class="text-text-muted font-proto text-[10px] tracking-wider uppercase">
              {i18n.t.stepProgressLabel.replace('{step}', String(step + 1)).replace('{total}', '2')}
            </span>

            <div class="flex items-center gap-2">
              {#if step === 0}
                <Button
                  type="button"
                  variant="ghost"
                  onclick={() => {
                    showImport = true;
                    error = '';
                  }}
                >
                  <span class="font-proto inline-flex items-center gap-1.5">
                    <Icon name="wallet" size={12} />
                    {i18n.t.importVaultTitle}
                  </span>
                </Button>
              {:else}
                <Button
                  type="button"
                  variant="ghost"
                  onclick={() => {
                    slideDirection = -1;
                    step = 0;
                    error = '';
                  }}
                >
                  <span class="font-proto">← {i18n.t.prev}</span>
                </Button>
              {/if}

              <Button type="button" variant="primary" disabled={busy} onclick={next}>
                <span class="font-proto text-[11px] font-medium tracking-wider">
                  {busy
                    ? i18n.t.creatingVault
                    : step === 0
                      ? i18n.t.btnNextAccount
                      : i18n.t.btnCreateVault}
                </span>
              </Button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

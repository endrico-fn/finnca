<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import { createVault as apiCreateVault, createAccount } from '$lib/core/ipc/bindings';
  import { extractErrorMessage } from '$lib/core/ipc/errors';
  import { pickDirectory } from '$lib/core/dialog';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Button, Icon } from '$lib/components/ui';
  import { session } from '$lib/core/state/session.svelte';
  import { onMount } from 'svelte';
  import {
    getKnownVaults,
    rememberVault,
    syncVaultsFromBackend,
  } from '$lib/features/vault/state/vaultList.svelte';
  import { APP_SLUG, type AppStateView } from '$lib/core/types';
  import VaultBrandHeader from './VaultBrandHeader.svelte';
  import VaultImportView from './VaultImportView.svelte';
  import RegisterFields from './RegisterFields.svelte';

  let {
    from = null,
    onSuccess,
    onCancel,
  }: {
    from?: 'app' | 'login' | null;
    onSuccess?: (st: AppStateView) => void;
    onCancel?: () => void;
  } = $props();

  const effectiveFrom = $derived(from ?? page.url.searchParams.get('from'));
  const knownVaults = $derived(getKnownVaults());

  onMount(async () => {
    await syncVaultsFromBackend();
  });

  let step = $state<1 | 2>(1);
  let vaultName = $state('');
  let vaultPath = $state('');
  let username = $state('');
  let password = $state('');
  let confirm = $state('');
  let templateLanguage = $state<'en' | 'id'>(i18n.locale === 'id' ? 'id' : 'en');
  let accountProfile = $state<'personal' | 'freelance' | 'business' | 'minimal'>('personal');
  let busy = $state(false);
  let error = $state('');
  let showImport = $state(false);

  const slug = $derived(
    vaultName
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '') || 'vault'
  );

  const targetFolderPreview = $derived.by(() => {
    if (!vaultPath) return '';
    const cleanPath = vaultPath.replace(/\/+$/, '');
    if (cleanPath.split('/').pop()?.startsWith(`${APP_SLUG}-`)) {
      return cleanPath;
    }
    return `${cleanPath}/${APP_SLUG}-${slug}`;
  });

  async function browseVaultFolder() {
    const dir = await pickDirectory();
    if (dir) vaultPath = dir;
  }

  function proceedToNextStep() {
    error = '';
    if (!vaultName.trim()) {
      error = i18n.t.vaultNameRequired;
      return;
    }
    if (!vaultPath) {
      error = i18n.t.vaultPathRequired;
      return;
    }
    if (!username.trim()) {
      error = i18n.t.usernameRequired;
      return;
    }
    if (password.length < 8) {
      error = i18n.t.passwordMin8;
      return;
    }
    if (password !== confirm) {
      error = i18n.t.passwordMismatch;
      return;
    }
    step = 2;
  }

  async function createVault() {
    error = '';
    if (!vaultName.trim()) {
      error = i18n.t.vaultNameRequired;
      return;
    }
    if (!vaultPath) {
      error = i18n.t.vaultPathRequired;
      return;
    }
    if (!username.trim()) {
      error = i18n.t.usernameRequired;
      return;
    }
    if (password.length < 8) {
      error = i18n.t.passwordMin8;
      return;
    }
    if (password !== confirm) {
      error = i18n.t.passwordMismatch;
      return;
    }

    busy = true;
    try {
      await apiCreateVault(vaultName.trim(), vaultPath);
      const st = await createAccount(username.trim(), password, templateLanguage, accountProfile);
      session.raw = st;
      if (st.vault_name) {
        rememberVault({
          id: st.vault_name,
          name: st.vault_name,
          path: targetFolderPreview,
          username: st.username ?? username.trim(),
        });
      }
      if (onSuccess) {
        onSuccess(st);
      } else {
        goto(resolve('/app'));
      }
    } catch (e) {
      error = extractErrorMessage(e);
    } finally {
      busy = false;
    }
  }

  function cancelVaultRegistration() {
    if (onCancel) {
      onCancel();
    } else if (effectiveFrom === 'app') {
      goto(resolve('/app'));
    } else {
      goto(resolve('/login'));
    }
  }
</script>

<div
  class="border-line bg-bg-card relative max-h-[calc(100dvh-2rem)] w-full max-w-4xl overflow-y-auto border"
>
  <div class="grid min-h-135 grid-cols-12 items-stretch">
    <div class="bg-bg-app border-line col-span-5 flex flex-col justify-between border-r p-7">
      <div>
        <VaultBrandHeader tagline={i18n.t.setupTagline} />

        <div class="border-line bg-bg-card/60 text-small space-y-3 border p-4">
          <div>
            <span class="text-text-dim text-smaller font-proto block tracking-wider uppercase">
              {i18n.t.setupStagesLabel}
            </span>
            <span class="text-text-strong font-proto text-small mt-1 block font-medium">
              {#if showImport}
                {i18n.t.importVaultTitle}
              {:else if step === 1}
                STEP 1: IDENTITY
              {:else}
                STEP 2: LEDGER
              {/if}
            </span>
          </div>

          <div class="border-line border-t pt-3">
            <span class="text-text-dim text-smaller font-proto block tracking-wider uppercase">
              {i18n.t.encryptionScheme}
            </span>
            <span class="text-text-base text-smaller font-proto"
              >Argon2id · SQLCipher AES-256-GCM</span
            >
          </div>

          <div class="border-line border-t pt-2">
            <p class="text-text-muted font-proto text-smaller leading-relaxed">
              {i18n.t.vaultStorageDesc}
            </p>
          </div>
        </div>
      </div>

      <div class="pt-6">
        <p class="text-text-muted border-line text-smaller border-l pl-3 leading-relaxed">
          {i18n.t.setupSubtitle}
        </p>
      </div>
    </div>

    <div class="bg-bg-card col-span-7 flex flex-col justify-between p-7">
      {#if showImport}
        <VaultImportView
          {onSuccess}
          onCancel={() => {
            showImport = false;
            error = '';
          }}
        />
      {:else}
        <div class="anim-fade-fast flex flex-1 flex-col justify-between">
          <div class="space-y-3.5">
            <div class="border-line flex items-start justify-between gap-3 border-b pb-2.5">
              <div>
                <h2
                  class="text-text-strong font-proto text-medium font-medium tracking-wide uppercase"
                >
                  {#if step === 1}
                    {i18n.t.stepVault} & IDENTITY
                  {:else}
                    LEDGER CONFIGURATION
                  {/if}
                </h2>
                <p class="text-text-dim font-proto text-smaller mt-0.5">
                  {i18n.t.setupFormSubtitle}
                </p>
              </div>

              {#if effectiveFrom === 'app'}
                <button
                  type="button"
                  onclick={cancelVaultRegistration}
                  class="sharp-btn btn-ghost font-proto text-text-muted hover:text-text-strong text-smaller inline-flex shrink-0 cursor-pointer items-center gap-1 px-2.5 py-1"
                >
                  <Icon name="chev-left" size={11} />
                  <span>{i18n.t.backToApp}</span>
                </button>
              {:else if effectiveFrom === 'login' || knownVaults.length > 0}
                <button
                  type="button"
                  onclick={cancelVaultRegistration}
                  class="sharp-btn btn-ghost font-proto text-text-muted hover:text-text-strong text-smaller inline-flex shrink-0 cursor-pointer items-center gap-1 px-2.5 py-1"
                >
                  <Icon name="chev-left" size={11} />
                  <span>{i18n.t.backToLogin}</span>
                </button>
              {/if}
            </div>

            <RegisterFields
              {step}
              bind:vaultName
              bind:vaultPath
              bind:username
              bind:password
              bind:confirm
              bind:templateLanguage
              bind:accountProfile
              {targetFolderPreview}
              onPickFolder={browseVaultFolder}
            />

            {#if error}
              <div class="badge-err font-proto text-small px-3 py-2 tracking-wide">
                {error}
              </div>
            {/if}
          </div>

          <div class="border-line mt-5 flex items-center justify-between border-t pt-4">
            {#if step === 1}
              <Button
                type="button"
                variant="ghost"
                onclick={() => {
                  showImport = true;
                  error = '';
                }}
              >
                <span class="font-proto text-teal text-small inline-flex items-center gap-1.5">
                  <Icon name="wallet" size={13} />
                  {i18n.t.importVaultTitle}
                </span>
              </Button>

              <Button type="button" variant="primary" onclick={proceedToNextStep}>
                <span class="font-proto text-small font-bold tracking-wider">
                  NEXT STEP &rarr;
                </span>
              </Button>
            {:else}
              <Button
                type="button"
                variant="ghost"
                onclick={() => {
                  step = 1;
                  error = '';
                }}
              >
                <span
                  class="font-proto text-text-strong text-small inline-flex items-center gap-1.5"
                >
                  <Icon name="chev-left" size={13} />
                  BACK
                </span>
              </Button>

              <Button variant="primary" onclick={createVault} disabled={busy}>
                <span class="font-proto text-small font-bold tracking-wider">
                  {busy ? i18n.t.creatingVault : i18n.t.btnCreateVault}
                </span>
              </Button>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

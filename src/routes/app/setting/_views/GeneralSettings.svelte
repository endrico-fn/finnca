<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from '$lib/stores/app-store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import * as api from '$lib/api';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import { Card } from '$lib/components/ui';

  let username = $state(store.appState?.username || '');
  let usernameError = $state('');
  let usernameSaving = $state(false);
  let usernameSuccess = $state('');

  let vaultName = $state(store.appState?.vault_name || '');
  let vaultNameError = $state('');
  let vaultNameSaving = $state(false);
  let vaultNameSuccess = $state('');

  // Formatting preferences
  let numberFormat = $state<'dot' | 'comma'>('comma');
  let dateFormat = $state<'iso' | 'slash'>('iso');

  onMount(() => {
    if (typeof localStorage !== 'undefined') {
      numberFormat = (localStorage.getItem('finnca_num_format') as 'dot' | 'comma') || 'comma';
      dateFormat = (localStorage.getItem('finnca_date_format') as 'iso' | 'slash') || 'iso';
    }
  });

  function setNumberFormat(fmt: 'dot' | 'comma') {
    numberFormat = fmt;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('finnca_num_format', fmt);
    }
  }

  function setDateFormat(fmt: 'iso' | 'slash') {
    dateFormat = fmt;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('finnca_date_format', fmt);
    }
  }

  async function handleSaveUsername() {
    if (!username.trim()) {
      usernameError = i18n.t.usernameEmptyError;
      return;
    }
    if (username.trim().length < 2) {
      usernameError = i18n.t.usernameTooShortError;
      return;
    }
    usernameSaving = true;
    usernameError = '';
    usernameSuccess = '';
    try {
      const state = await api.renameUser(username.trim());
      store.appState = state;

      const { syncVaultsFromBackend } = await import('$lib/stores/vault-registry.svelte');
      await syncVaultsFromBackend();

      usernameSuccess = i18n.t.settingsSavedOk;
      setTimeout(() => (usernameSuccess = ''), 3000);
    } catch (e: any) {
      usernameError = e.message;
    } finally {
      usernameSaving = false;
    }
  }

  async function handleSaveVaultName() {
    if (!vaultName.trim()) {
      vaultNameError = i18n.t.vaultNameEmpty;
      return;
    }
    vaultNameSaving = true;
    vaultNameError = '';
    vaultNameSuccess = '';
    try {
      const state = await api.renameVault(vaultName.trim());
      store.appState = state;

      const { syncVaultsFromBackend } = await import('$lib/stores/vault-registry.svelte');
      await syncVaultsFromBackend();

      vaultNameSuccess = i18n.t.vaultRenamedOk;
      setTimeout(() => (vaultNameSuccess = ''), 3000);
    } catch (e: any) {
      vaultNameError = e.message;
    } finally {
      vaultNameSaving = false;
    }
  }

  async function handleOpenVaultFolder() {
    try {
      await api.openVaultFolder();
    } catch (e: any) {
      console.error(e);
    }
  }
</script>

<div class="flex flex-col gap-2 select-none">
  <!-- ROW 1: USER PROFILE (LEFT) & VAULT IDENTITY (RIGHT) -->
  <div class="grid grid-cols-1 gap-2 lg:grid-cols-2">
    <!-- 1. USER PROFILE CARD -->
    <Card title={i18n.t.userProfile} badge="IDENTITY" class="justify-between">
      <div class="flex flex-col gap-2.5">
        <!-- Username Input -->
        <div>
          <label for="username-input" class="label-xs text-text-muted mb-1 block">
            {i18n.t.username}
          </label>
          <div class="flex items-center gap-2">
            <input
              id="username-input"
              type="text"
              class="sharp-input font-mono flex-1 px-3 py-1.5 text-[13px]"
              bind:value={username}
              onkeydown={(e) => e.key === 'Enter' && handleSaveUsername()}
            />
            <button
              type="button"
              class="sharp-btn btn-primary font-proto h-8 shrink-0 px-3 text-[10px] disabled:opacity-40"
              disabled={usernameSaving || username === store.appState?.username}
              onclick={handleSaveUsername}
            >
              {#if usernameSaving}<span class="spinner-sm"></span>{:else}{i18n.t
                  .saveChanges}{/if}
            </button>
          </div>
          {#if usernameError}
            <p class="text-expense font-proto mt-1 text-[9px]">{usernameError}</p>
          {/if}
          {#if usernameSuccess}
            <p class="text-income font-proto mt-1 text-[9px]">{usernameSuccess}</p>
          {/if}
        </div>

        <!-- Role / Status Read-only Field -->
        <div>
          <span class="label-xs text-text-muted mb-1 block">
            {i18n.t.role}
          </span>
          <div class="flex items-center gap-2">
            <div
              class="sharp-input text-text-muted bg-bg-app border-line flex-1 border px-2.5 py-1.5 font-mono text-[10px]"
            >
              FINNCA_VAULT_OWNER [OFFLINE_ADMIN]
            </div>
            <span
              class="border-income/40 text-income bg-income/10 font-proto border px-2 py-1 text-[9px] tracking-wider uppercase leading-none"
            >
              ACTIVE
            </span>
          </div>
        </div>
      </div>

      <p class="text-text-dim border-line/30 mt-auto border-t pt-2 font-mono text-[9px]">
        {i18n.locale === 'id'
          ? 'Identitas akun lokal, tersimpan offline di konfigurasi perangkat.'
          : 'Local account identity, stored offline in local device configuration.'}
      </p>
    </Card>

    <!-- 2. VAULT IDENTITY & STORAGE CARD -->
    <Card title={i18n.t.vaultIdentity} badge="VAULT" class="justify-between">
      <div class="flex flex-col gap-2.5">
        <!-- Vault Name Field -->
        <div>
          <label for="vault-name-input" class="label-xs text-text-muted mb-1 block">
            {i18n.t.vaultName}
          </label>
          <div class="flex items-center gap-2">
            <input
              id="vault-name-input"
              type="text"
              class="sharp-input font-mono flex-1 px-3 py-1.5 text-[13px]"
              bind:value={vaultName}
              onkeydown={(e) => e.key === 'Enter' && handleSaveVaultName()}
            />
            <button
              type="button"
              class="sharp-btn btn-primary font-proto h-8 shrink-0 px-3 text-[10px] disabled:opacity-40"
              disabled={vaultNameSaving || vaultName === store.appState?.vault_name}
              onclick={handleSaveVaultName}
            >
              {#if vaultNameSaving}<span class="spinner-sm"></span>{:else}{i18n.t
                  .saveChanges}{/if}
            </button>
          </div>
          {#if vaultNameError}
            <p class="text-expense font-proto mt-1 text-[9px]">{vaultNameError}</p>
          {/if}
          {#if vaultNameSuccess}
            <p class="text-income font-proto mt-1 text-[9px]">{vaultNameSuccess}</p>
          {/if}
        </div>

        <!-- Storage Location Row -->
        <div>
          <p class="label-xs text-text-muted mb-1 block">
            {i18n.t.vaultPathLabel}
          </p>
          <div class="flex items-center gap-2">
            <div
              class="sharp-input text-text-muted bg-bg-app border-line flex-1 truncate border px-2.5 py-1.5 font-mono text-[10px]"
            >
              {store.appState?.vault_name
                ? `finnca-${store.appState.vault_name.toLowerCase().replace(/\s+/g, '-')}`
                : 'finnca-vault'}
            </div>
            <button
              type="button"
              class="sharp-btn btn-ghost font-proto border-line hover:border-text-dim h-8 shrink-0 px-3 text-[10px]"
              onclick={handleOpenVaultFolder}
            >
              {i18n.t.openFolderBtn}
            </button>
          </div>
        </div>
      </div>

      <p class="text-text-dim border-line/30 mt-auto border-t pt-2 font-mono text-[9px]">
        {i18n.locale === 'id'
          ? 'Direktori file lokal terenkripsi XChaCha20-Poly1305.'
          : 'Local encrypted database directory with XChaCha20-Poly1305.'}
      </p>
    </Card>
  </div>

  <!-- ROW 2: SYSTEM LANGUAGE & DISPLAY FORMATS -->
  <Card title="{i18n.t.language} & {i18n.t.displayPreferences}" badge="LOCALIZATION">
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      <!-- Language Buttons -->
      <div>
        <p class="label-xs text-text-muted mb-1.5 block">{i18n.t.primaryLanguageDesc}</p>
        <div class="grid grid-cols-2 gap-2">
          <button
            type="button"
            class="sharp-card border p-2.5 text-left transition-all {i18n.locale === 'en'
              ? 'border-teal/60 bg-teal/5 ring-teal/30 ring-1'
              : 'border-line hover:border-text-dim bg-bg-app'}"
            onclick={() => {
              i18n.locale = 'en';
              localStorage.setItem('finnca_locale', 'en');
            }}
          >
            <div class="mb-1 flex items-center justify-between">
              <span
                class="font-proto border-line bg-bg-card text-text-muted border px-1.5 py-0.5 text-[9px] font-bold"
                >EN</span
              >
              {#if i18n.locale === 'en'}<span class="bg-teal size-1.5 animate-pulse"></span>{/if}
            </div>
            <div class="font-proto text-text-strong text-[11px] font-bold">
              {i18n.t.langEnglish}
            </div>
          </button>

          <button
            type="button"
            class="sharp-card border p-2.5 text-left transition-all {i18n.locale === 'id'
              ? 'border-teal/60 bg-teal/5 ring-teal/30 ring-1'
              : 'border-line hover:border-text-dim bg-bg-app'}"
            onclick={() => {
              i18n.locale = 'id';
              localStorage.setItem('finnca_locale', 'id');
            }}
          >
            <div class="mb-1 flex items-center justify-between">
              <span
                class="font-proto border-line bg-bg-card text-text-muted border px-1.5 py-0.5 text-[9px] font-bold"
                >ID</span
              >
              {#if i18n.locale === 'id'}<span class="bg-teal size-1.5 animate-pulse"></span>{/if}
            </div>
            <div class="font-proto text-text-strong text-[11px] font-bold">
              {i18n.t.langIndonesia}
            </div>
          </button>
        </div>
      </div>

      <!-- Number and Date Formatting Preferences -->
      <div class="flex flex-col gap-2">
        <div>
          <p class="label-xs text-text-muted mb-1.5 block">
            {i18n.t.numberFormatLabel}
          </p>
          <div class="grid grid-cols-2 gap-2">
            <button
              type="button"
              onclick={() => setNumberFormat('comma')}
              class="sharp-card font-proto flex items-center justify-between border p-2 text-left text-[10px] {numberFormat ===
              'comma'
                ? 'border-teal text-text-white bg-teal/5'
                : 'border-line text-text-base hover:border-text-dim'}"
            >
              <span>1,234,567.89</span>
              {#if numberFormat === 'comma'}<span class="text-teal font-bold">✓</span>{/if}
            </button>
            <button
              type="button"
              onclick={() => setNumberFormat('dot')}
              class="sharp-card font-proto flex items-center justify-between border p-2 text-left text-[10px] {numberFormat ===
              'dot'
                ? 'border-teal text-text-white bg-teal/5'
                : 'border-line text-text-base hover:border-text-dim'}"
            >
              <span>1.234.567,89</span>
              {#if numberFormat === 'dot'}<span class="text-teal font-bold">✓</span>{/if}
            </button>
          </div>
        </div>

        <div>
          <p class="label-xs text-text-muted mb-1.5 block">
            {i18n.t.dateFormatLabel}
          </p>
          <div class="grid grid-cols-2 gap-2">
            <button
              type="button"
              onclick={() => setDateFormat('iso')}
              class="sharp-card font-proto flex items-center justify-between border p-2 text-left text-[10px] {dateFormat ===
              'iso'
                ? 'border-teal text-text-white bg-teal/5'
                : 'border-line text-text-base hover:border-text-dim'}"
            >
              <span>YYYY-MM-DD</span>
              {#if dateFormat === 'iso'}<span class="text-teal font-bold">✓</span>{/if}
            </button>
            <button
              type="button"
              onclick={() => setDateFormat('slash')}
              class="sharp-card font-proto flex items-center justify-between border p-2 text-left text-[10px] {dateFormat ===
              'slash'
                ? 'border-teal text-text-white bg-teal/5'
                : 'border-line text-text-base hover:border-text-dim'}"
            >
              <span>DD/MM/YYYY</span>
              {#if dateFormat === 'slash'}<span class="text-teal font-bold">✓</span>{/if}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Card>
</div>

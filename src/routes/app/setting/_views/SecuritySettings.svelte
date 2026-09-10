<script lang="ts">
  import { store } from '$lib/stores/app-store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import * as api from '$lib/api';
  import type { Settings } from '$lib/types';
  import { SelectDropdown, Card } from '$lib/components/ui';

  let mode = $state<Settings['auto_lock_mode']>(
    store.appState?.settings?.auto_lock_mode || 'always'
  );
  let saved = $state(false);
  let busy = $state(false);
  let error = $state('');
  let sessionTimeout = $state('15');

  // Change password state
  let oldPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let pwdBusy = $state(false);
  let pwdError = $state('');
  let pwdSuccess = $state('');

  function loadLocalPrefs() {
    if (typeof localStorage === 'undefined') return;
    sessionTimeout = localStorage.getItem('finnca_lock_timeout') ?? '15';
  }

  export function onMountLogic() {
    mode = store.appState?.settings?.auto_lock_mode || 'always';
    loadLocalPrefs();
  }

  async function handleSaveSettings() {
    busy = true;
    error = '';
    saved = false;
    try {
      const st = await api.setAutoLockMode(mode);
      store.appState = st;
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('finnca_lock_timeout', sessionTimeout);
      }
      saved = true;
      setTimeout(() => (saved = false), 3000);
    } catch (e: any) {
      error = e.message;
    } finally {
      busy = false;
    }
  }

  async function handleChangePassword() {
    if (!oldPassword) {
      pwdError = i18n.t.currentPasswordRequired;
      return;
    }
    if (newPassword.length < 8) {
      pwdError = i18n.t.passwordTooShort;
      return;
    }
    if (newPassword !== confirmPassword) {
      pwdError = i18n.t.passwordsDoNotMatch;
      return;
    }
    pwdBusy = true;
    pwdError = '';
    pwdSuccess = '';
    try {
      const st = await api.changePassword(oldPassword, newPassword);
      store.appState = st;
      oldPassword = '';
      newPassword = '';
      confirmPassword = '';
      pwdSuccess = i18n.t.passwordChangedOk;
      setTimeout(() => (pwdSuccess = ''), 4000);
    } catch (e: any) {
      pwdError =
        e.message ||
        i18n.t.genericChangePasswordFail;
    } finally {
      pwdBusy = false;
    }
  }
</script>

<div class="grid grid-cols-1 gap-2 select-none lg:grid-cols-2">
  <!-- LEFT COLUMN: ACCESS CONTROL & SESSION TIMEOUT -->
  <Card title={i18n.t.autoLockTitle} badge="ACCESS CONTROL" class="justify-between">
    <div>
      <p class="text-text-dim mb-3 font-mono text-[11px]">{i18n.t.autoLockDesc}</p>

      <div class="flex flex-col gap-2">
        <label
          class="flex cursor-pointer items-start gap-2 border p-2.5 transition-all {mode ===
          'always'
            ? 'border-teal/60 bg-teal/5 ring-teal/30 ring-1'
            : 'border-line hover:border-text-dim bg-bg-app'}"
        >
          <input type="radio" bind:group={mode} value="always" class="accent-teal mt-0.5" />
          <div>
            <p class="font-proto text-text-strong mb-1 text-[11px] leading-none font-bold">
              {i18n.t.lockEveryOpen}
            </p>
            <p class="text-text-muted font-mono text-[9px] leading-relaxed">
              {i18n.t.lockEveryOpenDesc}
            </p>
          </div>
        </label>

        <label
          class="flex cursor-pointer items-start gap-2 border p-2.5 transition-all {mode ===
          'on-reboot'
            ? 'border-teal/60 bg-teal/5 ring-teal/30 ring-1'
            : 'border-line hover:border-text-dim bg-bg-app'}"
        >
          <input type="radio" bind:group={mode} value="on-reboot" class="accent-teal mt-0.5" />
          <div>
            <p class="font-proto text-text-strong mb-1 text-[11px] leading-none font-bold">
              {i18n.t.lockReboot}
            </p>
            <p class="text-text-muted font-mono text-[9px] leading-relaxed">
              {i18n.t.lockRebootDesc}
            </p>
          </div>
        </label>
      </div>

      <!-- SESSION TIMEOUT DROPDOWN -->
      {#if mode === 'always'}
        <div class="border-line/40 mt-3 flex flex-col gap-1.5 border-t pt-3">
          <label for="timeout-select" class="label-xs text-text-muted block">
            {i18n.t.sessionTimeoutTitle}
          </label>
          <div class="flex items-center gap-2">
            <SelectDropdown
              bind:value={sessionTimeout}
              class="flex-1"
              options={[
                { value: '5', label: i18n.t.timeout5min },
                { value: '15', label: i18n.t.timeout15min },
                { value: '30', label: i18n.t.timeout30min },
                { value: '60', label: i18n.t.timeout1hr },
                { value: '120', label: i18n.t.timeout2hr },
                { value: 'never', label: i18n.t.timeoutNever },
              ]}
            />

            <button
              type="button"
              onclick={handleSaveSettings}
              disabled={busy}
              class="sharp-btn btn-primary font-proto h-8 shrink-0 px-3.5 text-[10px]"
            >
              {#if busy}<span class="spinner-sm"></span>{:else}{i18n.t.saveSettings}{/if}
            </button>
          </div>
        </div>
      {:else}
        <div class="border-line/40 mt-3 flex items-center justify-between border-t pt-3">
          <button
            type="button"
            onclick={handleSaveSettings}
            disabled={busy}
            class="sharp-btn btn-primary font-proto h-8 px-3.5 text-[10px]"
          >
            {#if busy}<span class="spinner-sm"></span>{:else}{i18n.t.saveSettings}{/if}
          </button>
        </div>
      {/if}

      {#if error}<p class="text-expense font-proto mt-2 text-[10px]">{error}</p>{/if}
      {#if saved}<p class="badge-ok font-proto mt-2 px-2.5 py-1 text-[10px]">
          {i18n.t.settingsSavedOk}
        </p>{/if}
    </div>

    <p class="text-text-dim border-line/30 mt-auto border-t pt-2 font-mono text-[9px]">
        {i18n.t.inactivityHeartbeatNote}
      </p>
  </Card>

  <!-- RIGHT COLUMN: CHANGE MASTER PASSWORD -->
  <Card title={i18n.t.changePasswordTitle} badge="SECURITY KEY" class="justify-between">
    <div>
      <p class="text-text-dim mb-3 font-mono text-[11px]">{i18n.t.changePasswordNotice}</p>

      <div class="flex flex-col gap-2.5">
        <div>
          <label for="old-pwd" class="label-xs text-text-muted mb-1 block"
            >{i18n.t.currentPasswordLabel}</label
          >
          <input
            id="old-pwd"
            type="password"
            bind:value={oldPassword}
            class="sharp-input w-full px-3 py-1.5 font-mono text-[12px]"
            placeholder="••••••••"
          />
        </div>

        <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
          <div>
            <label for="new-pwd" class="label-xs text-text-muted mb-1 block"
              >{i18n.t.newPasswordLabel}</label
            >
            <input
              id="new-pwd"
              type="password"
              bind:value={newPassword}
              class="sharp-input w-full px-3 py-1.5 font-mono text-[12px]"
              placeholder="••••••••"
            />
          </div>

          <div>
            <label for="confirm-pwd" class="label-xs text-text-muted mb-1 block"
              >{i18n.t.confirmNewPasswordLabel}</label
            >
            <input
              id="confirm-pwd"
              type="password"
              bind:value={confirmPassword}
              class="sharp-input w-full px-3 py-1.5 font-mono text-[12px]"
              placeholder="••••••••"
            />
          </div>
        </div>
      </div>

      <div class="mt-1 flex items-center gap-2 pt-3">
        <button
          type="button"
          onclick={handleChangePassword}
          disabled={pwdBusy || !oldPassword || !newPassword}
          class="sharp-btn btn-primary font-proto h-8 px-4 text-[10px] disabled:opacity-40"
        >
          {#if pwdBusy}<span class="spinner-sm"></span>{:else}{i18n.t.changePasswordBtn}{/if}
        </button>

        {#if pwdError}<p class="text-expense font-proto text-[10px]">{pwdError}</p>{/if}
        {#if pwdSuccess}<p class="badge-ok font-proto px-2.5 py-1 text-[10px]">{pwdSuccess}</p>{/if}
      </div>
    </div>

    <p class="text-text-dim border-line/30 mt-auto border-t pt-2 font-mono text-[9px]">
        {i18n.t.reEncryptNote}
      </p>
  </Card>
</div>

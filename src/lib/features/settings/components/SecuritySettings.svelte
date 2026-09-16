<script lang="ts">
  import { session } from '$lib/core/state/session.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { changePassword } from '$lib/core/ipc/bindings';
  import { Card, Button } from '$lib/components/ui';
  import AutoLockSettings from '$lib/features/security/components/AutoLockSettings.svelte';

  let oldPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let pwdBusy = $state(false);
  let pwdError = $state('');
  let pwdSuccess = $state('');

  const canChangePassword = $derived(
    !pwdBusy && oldPassword !== '' && newPassword.length >= 8 && newPassword === confirmPassword
  );

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
      const st = await changePassword(oldPassword, newPassword);
      session.raw = st;
      oldPassword = '';
      newPassword = '';
      confirmPassword = '';
      pwdSuccess = i18n.t.passwordChangedOk;
      setTimeout(() => (pwdSuccess = ''), 4000);
    } catch (e: unknown) {
      pwdError = (e instanceof Error ? e.message : String(e)) || i18n.t.genericChangePasswordFail;
    } finally {
      pwdBusy = false;
    }
  }
</script>

<div class="grid grid-cols-1 gap-2 select-none lg:grid-cols-2">
  <AutoLockSettings />

  <!-- RIGHT COLUMN: CHANGE MASTER PASSWORD -->
  <Card title={i18n.t.changePasswordTitle} badge={i18n.t.securityKeyBadge} class="justify-between">
    <div>
      <p class="text-text-dim text-small font-aux mb-3">{i18n.t.changePasswordNotice}</p>

      <div class="flex flex-col gap-2.5">
        <div>
          <label for="old-pwd" class="label-xs text-text-muted mb-1 block"
            >{i18n.t.currentPasswordLabel}</label
          >
          <input
            id="old-pwd"
            type="password"
            bind:value={oldPassword}
            class="sharp-input text-small w-full px-2.5 py-1.5"
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
              class="sharp-input text-small w-full px-2.5 py-1.5"
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
              class="sharp-input text-small w-full px-2.5 py-1.5"
              placeholder="••••••••"
            />
          </div>
        </div>
      </div>

      <div class="border-line/40 mt-3 flex items-center justify-between gap-2 border-t pt-2.5">
        <div class="min-h-5 flex items-center">
          {#if pwdError}<p class="text-expense font-proto text-smaller">{pwdError}</p>{/if}
          {#if pwdSuccess}<p class="text-income font-proto text-smaller">{pwdSuccess}</p>{/if}
        </div>
        <Button
          variant="primary"
          class="font-proto text-small h-8 px-3 font-bold"
          disabled={!canChangePassword}
          onclick={handleChangePassword}
        >
          {i18n.t.changePasswordBtn}
        </Button>
      </div>
    </div>

    <p class="text-text-dim border-line/30 text-smaller font-aux mt-auto border-t pt-2">
      {i18n.t.reEncryptNote}
    </p>
  </Card>
</div>

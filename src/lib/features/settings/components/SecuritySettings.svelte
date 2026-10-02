<script lang="ts">
  import { session } from '$lib/core/state/session.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { changePassword } from '$lib/core/ipc/bindings';
  import { Button, Badge } from '$lib/components/ui';
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

<section class="border-line bg-bg-app flex flex-col border p-4 sm:p-6 select-none max-w-4xl">
  <AutoLockSettings />

  <div class="flex flex-col border-b border-line pb-6 mb-6 last:border-0 last:mb-0 last:pb-0">
    <div class="flex items-start justify-between mb-4">
      <div class="flex flex-col gap-1">
        <h3 class="font-proto text-text-strong text-small tracking-widest uppercase">{i18n.t.changePasswordTitle}</h3>
        <p class="text-text-dim text-smaller font-aux">
          {i18n.t.reEncryptNote}
        </p>
      </div>
      <Badge size="m" tone="neutral">{i18n.t.securityKeyBadge}</Badge>
    </div>
    
    <div>
      <p class="text-text-dim text-small font-aux mb-3">{i18n.t.changePasswordNotice}</p>

      <div class="flex flex-col gap-2.5">
        <div>
          <label for="old-pwd" class="font-proto text-text-muted text-smaller tracking-widest uppercase mb-1 block"
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
            <label for="new-pwd" class="font-proto text-text-muted text-smaller tracking-widest uppercase mb-1 block"
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
            <label for="confirm-pwd" class="font-proto text-text-muted text-smaller tracking-widest uppercase mb-1 block"
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
        <div class="flex min-h-5 items-center">
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
  </div>
</section>

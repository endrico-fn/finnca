<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { Icon } from '$lib/components/ui';

  let {
    step = 1,
    vaultName = $bindable(''),
    vaultPath = $bindable(''),
    username = $bindable(''),
    password = $bindable(''),
    confirm = $bindable(''),
    templateLanguage = $bindable<'en' | 'id'>('en'),
    accountProfile = $bindable<'personal' | 'freelance' | 'business' | 'minimal'>('personal'),
    targetFolderPreview,
    onPickFolder,
  }: {
    step?: number;
    vaultName: string;
    vaultPath: string;
    username: string;
    password: string;
    confirm: string;
    templateLanguage?: 'en' | 'id';
    accountProfile?: 'personal' | 'freelance' | 'business' | 'minimal';
    targetFolderPreview: string;
    onPickFolder: () => void;
  } = $props();

  let showPassword = $state(false);
  let isCapsLock = $state(false);
  let vaultInputEl: HTMLInputElement | null = $state(null);

  $effect(() => {
    if (step === 1 && vaultInputEl) {
      vaultInputEl.focus();
    }
  });

  function checkCapsLock(e: KeyboardEvent) {
    if (typeof e.getModifierState === 'function') {
      isCapsLock = e.getModifierState('CapsLock');
    }
  }
</script>

<div class="flex flex-col gap-4">
  {#if step === 1}
    <div class="anim-fade-fast flex flex-col gap-4">
      <!-- SECTION 1: VAULT IDENTITY -->
      <div class="flex flex-col gap-3">
        <div>
          <span class="label-xs text-text-base font-proto mb-1 block">
            {i18n.t.vaultNameLabel}
          </span>
          <input
            bind:this={vaultInputEl}
            bind:value={vaultName}
            class="sharp-input text-small h-8 w-full px-2.5"
            placeholder={i18n.t.vaultNamePlaceholder}
            autocomplete="off"
          />
        </div>

        <div>
          <span class="label-xs text-text-base font-proto mb-1 block">
            {i18n.t.storageLocation}
          </span>
          {#if !vaultPath}
            <button
              type="button"
              onclick={onPickFolder}
              class="sharp-input hover:border-teal/50 text-small flex h-8 w-full cursor-pointer items-center justify-between px-2.5 transition-colors"
            >
              <span class="text-text-dim">{i18n.t.chooseDirectory}</span>
              <span class="text-teal font-proto text-smaller flex items-center gap-1.5 uppercase">
                <span>{i18n.t.browseBtn}</span>
                <Icon name="folder" size={13} />
              </span>
            </button>
          {:else}
            <div class="border-line bg-bg-app flex h-8 items-center justify-between border px-2.5">
              <div class="flex min-w-0 items-center gap-2">
                <span class="text-teal inline-flex shrink-0">
                  <Icon name="folder" size={13} />
                </span>
                <span
                  class="font-proto text-small text-text-strong truncate break-all normal-case"
                  title={targetFolderPreview}
                >
                  {targetFolderPreview}
                </span>
              </div>
              <button
                type="button"
                onclick={onPickFolder}
                class="sharp-btn btn-ghost font-proto text-text-muted hover:text-text-strong text-smaller ml-2 shrink-0 cursor-pointer px-2 py-0.5 uppercase"
              >
                {i18n.t.changeFolder}
              </button>
            </div>
          {/if}
        </div>
      </div>

      <!-- SECTION 2: CREDENTIALS -->
      <div class="border-line/30 flex flex-col gap-3 border-t pt-4">
        <div>
          <span class="label-xs text-text-base font-proto mb-1 block">
            {i18n.t.usernameLabel}
          </span>
          <input
            bind:value={username}
            class="sharp-input text-small h-8 w-full px-2.5"
            placeholder={i18n.t.usernameInputPlaceholder}
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <span class="label-xs text-text-base font-proto mb-1 block">
              {i18n.t.masterPasswordLabel}
            </span>
            <div class="relative flex items-center">
              <input
                bind:value={password}
                type={showPassword ? 'text' : 'password'}
                onkeydown={checkCapsLock}
                onkeyup={checkCapsLock}
                class="sharp-input text-small h-8 w-full pr-8 pl-2.5"
                placeholder={i18n.t.passwordPlaceholder}
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
          </div>
          <div>
            <span class="label-xs text-text-base font-proto mb-1 block">
              {i18n.t.confirmPasswordLabel}
            </span>
            <div class="relative flex items-center">
              <input
                bind:value={confirm}
                type={showPassword ? 'text' : 'password'}
                onkeydown={checkCapsLock}
                onkeyup={checkCapsLock}
                class="sharp-input text-small h-8 w-full pr-8 pl-2.5"
                placeholder={i18n.t.passwordPlaceholder}
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
          </div>
        </div>
        {#if isCapsLock}
          <span class="text-warning font-proto text-smaller -mt-2 block font-bold tracking-wider">
            {i18n.t.capsLockActive}
          </span>
        {/if}
      </div>
    </div>
  {/if}

  {#if step === 2}
    <div class="anim-fade-fast flex flex-col gap-4">
      <!-- SECTION 3: LEDGER CONFIGURATION -->
      <div>
        <span class="label-xs text-text-base font-proto mb-1.5 block">
          {i18n.t.accountProfileLabel}
        </span>
        <div class="grid grid-cols-2 gap-2">
          <button
            type="button"
            onclick={() => (accountProfile = 'personal')}
            class="cursor-pointer border p-2.5 text-left transition-all {accountProfile ===
            'personal'
              ? 'border-teal bg-teal/10'
              : 'border-line/60 bg-bg-app hover:border-teal/50 hover:bg-bg-btn'}"
          >
            <span
              class="font-proto text-smaller block tracking-wide {accountProfile === 'personal'
                ? 'text-teal font-bold'
                : 'text-text-strong font-medium'}"
            >
              {i18n.t.profilePersonal}
            </span>
            <span
              class="font-aux text-text-dim text-smaller mt-0.5 line-clamp-2 block leading-snug"
            >
              {i18n.t.profilePersonalDesc}
            </span>
          </button>

          <button
            type="button"
            onclick={() => (accountProfile = 'freelance')}
            class="cursor-pointer border p-2.5 text-left transition-all {accountProfile ===
            'freelance'
              ? 'border-teal bg-teal/10'
              : 'border-line/60 bg-bg-app hover:border-teal/50 hover:bg-bg-btn'}"
          >
            <span
              class="font-proto text-smaller block tracking-wide {accountProfile === 'freelance'
                ? 'text-teal font-bold'
                : 'text-text-strong font-medium'}"
            >
              {i18n.t.profileFreelance}
            </span>
            <span
              class="font-aux text-text-dim text-smaller mt-0.5 line-clamp-2 block leading-snug"
            >
              {i18n.t.profileFreelanceDesc}
            </span>
          </button>

          <button
            type="button"
            onclick={() => (accountProfile = 'business')}
            class="cursor-pointer border p-2.5 text-left transition-all {accountProfile ===
            'business'
              ? 'border-teal bg-teal/10'
              : 'border-line/60 bg-bg-app hover:border-teal/50 hover:bg-bg-btn'}"
          >
            <span
              class="font-proto text-smaller block tracking-wide {accountProfile === 'business'
                ? 'text-teal font-bold'
                : 'text-text-strong font-medium'}"
            >
              {i18n.t.profileBusiness}
            </span>
            <span
              class="font-aux text-text-dim text-smaller mt-0.5 line-clamp-2 block leading-snug"
            >
              {i18n.t.profileBusinessDesc}
            </span>
          </button>

          <button
            type="button"
            onclick={() => (accountProfile = 'minimal')}
            class="cursor-pointer border p-2.5 text-left transition-all {accountProfile ===
            'minimal'
              ? 'border-teal bg-teal/10'
              : 'border-line/60 bg-bg-app hover:border-teal/50 hover:bg-bg-btn'}"
          >
            <span
              class="font-proto text-smaller block tracking-wide {accountProfile === 'minimal'
                ? 'text-teal font-bold'
                : 'text-text-strong font-medium'}"
            >
              {i18n.t.profileMinimal}
            </span>
            <span
              class="font-aux text-text-dim text-smaller mt-0.5 line-clamp-2 block leading-snug"
            >
              {i18n.t.profileMinimalDesc}
            </span>
          </button>
        </div>
      </div>

      <div>
        <span class="label-xs text-text-base font-proto mb-1.5 block">
          {i18n.t.accountTemplateLanguage}
        </span>
        <div class="grid grid-cols-2">
          <button
            type="button"
            onclick={() => (templateLanguage = 'en')}
            class="font-proto text-smaller flex h-8 cursor-pointer items-center justify-center gap-1.5 border border-r-0 px-2.5 tracking-wide uppercase transition-colors {templateLanguage ===
            'en'
              ? 'border-teal bg-teal text-bg-app font-bold'
              : 'border-line bg-bg-app text-text-muted hover:text-text-strong hover:bg-bg-btn'}"
          >
            <span>[EN] {i18n.t.templateLangEn}</span>
          </button>
          <button
            type="button"
            onclick={() => (templateLanguage = 'id')}
            class="font-proto text-smaller flex h-8 cursor-pointer items-center justify-center gap-1.5 border px-2.5 tracking-wide uppercase transition-colors {templateLanguage ===
            'id'
              ? 'border-teal bg-teal text-bg-app font-bold'
              : 'border-line bg-bg-app text-text-muted hover:text-text-strong hover:bg-bg-btn'}"
          >
            <span>[ID] {i18n.t.templateLangId}</span>
          </button>
        </div>
        <span class="text-text-dim font-proto text-smaller mt-1 block leading-tight">
          {i18n.t.templateLangHint}
        </span>
      </div>
    </div>
  {/if}
</div>

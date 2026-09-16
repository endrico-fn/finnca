<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { Icon } from '$lib/components/ui';

  let {
    vaultName = $bindable(''),
    vaultPath = $bindable(''),
    username = $bindable(''),
    password = $bindable(''),
    confirm = $bindable(''),
    templateLanguage = $bindable<'en' | 'id'>('en'),
    targetFolderPreview,
    onPickFolder,
  }: {
    vaultName: string;
    vaultPath: string;
    username: string;
    password: string;
    confirm: string;
    templateLanguage?: 'en' | 'id';
    targetFolderPreview: string;
    onPickFolder: () => void;
  } = $props();
</script>

<div class="space-y-3.5">
  <!-- Field 1: Vault Name -->
  <div>
    <span class="label-xs text-text-base font-proto mb-1 block">
      {i18n.t.vaultNameLabel}
    </span>
    <input
      bind:value={vaultName}
      class="sharp-input text-small w-full h-10 px-3"
      placeholder={i18n.t.vaultNamePlaceholder}
    />
  </div>

  <!-- Field 2: Storage Location -->
  <div>
    <span class="label-xs text-text-base font-proto mb-1 block">
      {i18n.t.storageLocation}
    </span>
    {#if !vaultPath}
      <button
        type="button"
        onclick={onPickFolder}
        class="sharp-input h-10 hover:border-teal/50 text-small flex w-full cursor-pointer items-center justify-between px-3 transition-colors"
      >
        <span class="text-text-dim">{i18n.t.chooseDirectory}</span>
        <span class="text-teal font-proto text-smaller flex items-center gap-1.5 uppercase">
          <span>{i18n.t.browseBtn}</span>
          <Icon name="folder" size={13} />
        </span>
      </button>
    {:else}
      <div class="border-line bg-bg-app h-10 flex items-center justify-between border px-3">
        <div class="flex min-w-0 items-center gap-2">
          <span class="text-teal shrink-0 inline-flex">
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

  <!-- Field 3: Username -->
  <div>
    <span class="label-xs text-text-base font-proto mb-1 block">
      {i18n.t.usernameLabel}
    </span>
    <input
      bind:value={username}
      class="sharp-input text-small w-full h-10 px-3"
      placeholder={i18n.t.usernameInputPlaceholder}
    />
  </div>

  <!-- Field 4 & 5: Password & Confirm Password -->
  <div class="grid grid-cols-2 gap-3">
    <div>
      <span class="label-xs text-text-base font-proto mb-1 block">
        {i18n.t.masterPasswordLabel}
      </span>
      <input
        bind:value={password}
        type="password"
        class="sharp-input text-small w-full h-10 px-3"
        placeholder={i18n.t.passwordPlaceholder}
      />
    </div>
    <div>
      <span class="label-xs text-text-base font-proto mb-1 block">
        {i18n.t.confirmPasswordLabel}
      </span>
      <input
        bind:value={confirm}
        type="password"
        class="sharp-input text-small w-full h-10 px-3"
        placeholder={i18n.t.passwordPlaceholder}
      />
    </div>
  </div>

  <!-- Field 6: Account Template Language -->
  <div>
    <div class="flex items-center justify-between mb-1">
      <span class="label-xs text-text-base font-proto block">
        {i18n.t.accountTemplateLanguage}
      </span>
      <span class="text-text-dim font-proto text-smaller">
        {templateLanguage === 'en' ? 'EN' : 'ID'}
      </span>
    </div>
    <div class="grid grid-cols-2 gap-2">
      <button
        type="button"
        onclick={() => (templateLanguage = 'en')}
        class="h-9 px-2.5 border font-proto text-smaller uppercase tracking-wide cursor-pointer transition-colors flex items-center justify-center gap-1.5 {templateLanguage === 'en' ? 'border-teal bg-teal/10 text-teal font-semibold' : 'border-line bg-bg-app text-text-muted hover:text-text-base'}"
      >
        <span class="font-bold">[EN]</span>
        <span>{i18n.t.templateLangEn}</span>
      </button>
      <button
        type="button"
        onclick={() => (templateLanguage = 'id')}
        class="h-9 px-2.5 border font-proto text-smaller uppercase tracking-wide cursor-pointer transition-colors flex items-center justify-center gap-1.5 {templateLanguage === 'id' ? 'border-teal bg-teal/10 text-teal font-semibold' : 'border-line bg-bg-app text-text-muted hover:text-text-base'}"
      >
        <span class="font-bold">[ID]</span>
        <span>{i18n.t.templateLangId}</span>
      </button>
    </div>
    <span class="text-text-dim font-proto text-smaller mt-1 block leading-tight">
      {i18n.t.templateLangHint}
    </span>
  </div>
</div>

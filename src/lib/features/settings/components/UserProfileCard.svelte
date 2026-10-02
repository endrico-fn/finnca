<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { Badge } from '$lib/components/ui';
  import { generalSettingsState } from '../state/settings.svelte';
</script>

<div class="flex flex-col border-b border-line pb-6 mb-6 last:border-0 last:mb-0 last:pb-0">
  <div class="flex items-start justify-between mb-4">
    <div class="flex flex-col gap-1">
    <h3 class="font-proto text-text-strong text-small tracking-widest uppercase">{i18n.t.userProfile}</h3>
      <p class="text-text-dim text-smaller font-aux">
        {i18n.t.accountIdentityNote}
      </p>
    </div>
    <Badge size="m" tone="neutral">{i18n.t.badgeIdentity}</Badge>
  </div>
  <div class="flex flex-col gap-3 sm:flex-row sm:gap-4">
    <div class="flex-1">
      <label for="username-input" class="font-proto text-text-muted text-smaller tracking-widest uppercase mb-1 block">
        {i18n.t.username}
      </label>
      <input
        id="username-input"
        type="text"
        class="sharp-input font-proto text-small h-8 w-full px-2.5"
        bind:value={generalSettingsState.username}
        onkeydown={(e) =>
          e.key === 'Enter' && generalSettingsState.canSave && generalSettingsState.save()}
      />
      {#if generalSettingsState.error}
        <p class="text-expense font-proto text-smaller mt-1">{generalSettingsState.error}</p>
      {/if}
    </div>

    <div class="flex-1">
      <span class="font-proto text-text-muted text-smaller tracking-widest uppercase mb-1 block">
        {i18n.t.role}
      </span>
      <div class="border-line bg-bg-app flex h-8 w-full items-center justify-between border px-2.5">
        <span class="text-text-strong font-proto text-small uppercase tracking-wider truncate">
          {i18n.t.roleOwner}
        </span>
        <Badge size="m" tone="ok">{i18n.t.activeStatusWord}</Badge>
      </div>
    </div>
  </div>

  </div>

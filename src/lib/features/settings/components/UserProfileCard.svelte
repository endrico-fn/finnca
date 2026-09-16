<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { Badge, Card } from '$lib/components/ui';
  import { generalSettingsState } from '../state/settings.svelte';
</script>

<Card title={i18n.t.userProfile} badge={i18n.t.badgeIdentity} class="justify-between">
  <div class="flex flex-col gap-2.5">
    <div>
      <label for="username-input" class="label-xs text-text-muted mb-1 block">
        {i18n.t.username}
      </label>
      <input
        id="username-input"
        type="text"
        class="sharp-input text-small w-full px-2.5 py-1.5"
        bind:value={generalSettingsState.username}
        onkeydown={(e) => e.key === 'Enter' && generalSettingsState.canSave && generalSettingsState.save()}
      />
      {#if generalSettingsState.error}
        <p class="text-expense font-proto text-smaller mt-1">{generalSettingsState.error}</p>
      {/if}
    </div>

    <div>
      <span class="label-xs text-text-muted mb-1 block">
        {i18n.t.role}
      </span>
      <div
        class="border-line bg-bg-app flex h-8 w-full items-center justify-between border px-2.5"
      >
        <span class="text-text-muted font-aux text-smaller truncate">
          {i18n.t.roleOwner}
        </span>
        <Badge size="m" tone="ok">{i18n.t.activeStatusWord}</Badge>
      </div>
    </div>
  </div>

  <p class="text-text-dim border-line/30 text-smaller font-aux mt-auto border-t pt-2">
    {i18n.t.accountIdentityNote}
  </p>
</Card>

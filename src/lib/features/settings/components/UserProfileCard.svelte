<script lang="ts">
  import { session } from '$lib/core/state/session.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { renameUser } from '$lib/core/ipc/bindings';
  import { Badge, Card, Button } from '$lib/components/ui';
  import { eventBus } from '$lib/core/events/eventBus.svelte';

  let username = $state(session.currentUser || '');
  let usernameError = $state('');
  let usernameSaving = $state(false);
  let usernameSuccess = $state('');

  const isDirty = $derived(username.trim() !== '' && username.trim() !== (session.currentUser || ''));
  const canSave = $derived(!usernameSaving && username.trim().length >= 2 && isDirty);

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
      const state = await renameUser(username.trim());
      session.raw = state;
      eventBus.emit('vault:registry_changed', undefined);
      usernameSuccess = i18n.t.settingsSavedOk;
      setTimeout(() => (usernameSuccess = ''), 3000);
    } catch (e: unknown) {
      usernameError = e instanceof Error ? e.message : String(e);
    } finally {
      usernameSaving = false;
    }
  }
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
        bind:value={username}
        onkeydown={(e) => e.key === 'Enter' && canSave && handleSaveUsername()}
      />
    </div>

    <div>
      <span class="label-xs text-text-muted mb-1 block">
        {i18n.t.role}
      </span>
      <div class="flex items-center gap-2">
        <div
          class="sharp-input text-text-muted bg-bg-app border-line text-smaller font-aux flex-1 border px-2.5 py-1.5"
        >
          {i18n.t.roleOwner}
        </div>
        <Badge size="m" tone="ok">{i18n.t.activeStatusWord}</Badge>
      </div>
    </div>

    <div class="border-line/40 mt-1 flex items-center justify-between gap-2 border-t pt-2.5">
      <div class="min-h-5 flex items-center">
        {#if usernameError}
          <p class="text-expense font-proto text-smaller">{usernameError}</p>
        {:else if usernameSuccess}
          <p class="text-income font-proto text-smaller">{usernameSuccess}</p>
        {/if}
      </div>
      <Button
        variant="primary"
        class="font-proto text-small h-8 px-3 font-bold"
        disabled={!canSave}
        onclick={handleSaveUsername}
      >
        {i18n.t.saveChanges}
      </Button>
    </div>
  </div>

  <p class="text-text-dim border-line/30 text-smaller font-aux mt-auto border-t pt-2">
    {i18n.t.accountIdentityNote}
  </p>
</Card>

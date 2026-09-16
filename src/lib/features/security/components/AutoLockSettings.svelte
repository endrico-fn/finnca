<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { setAutoLockMode } from '$lib/core/ipc/bindings';
  import type { Settings } from '$lib/core/types';
  import { Card, Button } from '$lib/components/ui';
  import { lockPolicy } from '$lib/features/security/state/lockPolicy.svelte';
  import { session } from '$lib/core/state/session.svelte';

  let mode = $state<Settings['auto_lock_mode']>('always');
  let sessionTimeout = $state('15');
  let saved = $state(false);
  let busy = $state(false);
  let error = $state('');

  const timeoutPresets = $derived([
    { value: '5', short: '5M', label: i18n.t.timeout5min },
    { value: '15', short: '15M', label: i18n.t.timeout15min },
    { value: '30', short: '30M', label: i18n.t.timeout30min },
    { value: '60', short: '1H', label: i18n.t.timeout1hr },
    { value: '120', short: '2H', label: i18n.t.timeout2hr },
    { value: 'never', short: 'OFF', label: i18n.t.timeoutNever },
  ]);

  const selectedPresetLabel = $derived(
    timeoutPresets.find((p) => p.value === sessionTimeout)?.label ?? ''
  );

  const isDirty = $derived(
    mode !== (session.settings?.auto_lock_mode || 'always') ||
      sessionTimeout !== lockPolicy.timeoutMinutes
  );
  const canSave = $derived(!busy && isDirty);

  onMount(() => {
    mode = session.settings?.auto_lock_mode || 'always';
    lockPolicy.loadFromStorage();
    sessionTimeout = lockPolicy.timeoutMinutes;
  });

  async function handleSaveSettings() {
    busy = true;
    error = '';
    saved = false;
    try {
      await setAutoLockMode(mode);
      lockPolicy.saveTimeout(sessionTimeout);
      lockPolicy.mode = mode;
      await session.refresh();
      saved = true;
      setTimeout(() => (saved = false), 3000);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<Card title={i18n.t.autoLockTitle} badge={i18n.t.badgeAccessControl} class="justify-between">
  <div>
    <p class="text-text-dim text-small font-aux mb-3">{i18n.t.autoLockDesc}</p>

    <div class="flex flex-col gap-2">
      <label
        class="flex cursor-pointer items-start gap-2 border p-2.5 transition-colors {mode ===
        'always'
          ? 'border-teal/60 bg-bg-row-active'
          : 'border-line hover:border-text-dim bg-bg-app'}"
      >
        <input type="radio" bind:group={mode} value="always" class="accent-teal mt-0.5" />
        <div>
          <p class="font-proto text-text-strong text-small mb-1 leading-none font-bold">
            {i18n.t.lockEveryOpen}
          </p>
          <p class="text-text-muted text-smaller font-aux leading-relaxed">
            {i18n.t.lockEveryOpenDesc}
          </p>
        </div>
      </label>

      <label
        class="flex cursor-pointer items-start gap-2 border p-2.5 transition-colors {mode ===
        'on-reboot'
          ? 'border-teal/60 bg-bg-row-active'
          : 'border-line hover:border-text-dim bg-bg-app'}"
      >
        <input type="radio" bind:group={mode} value="on-reboot" class="accent-teal mt-0.5" />
        <div>
          <p class="font-proto text-text-strong text-small mb-1 leading-none font-bold">
            {i18n.t.lockReboot}
          </p>
          <p class="text-text-muted text-smaller font-aux leading-relaxed">
            {i18n.t.lockRebootDesc}
          </p>
        </div>
      </label>
    </div>

    {#if mode === 'always'}
      <div class="border-line/40 mt-3 flex flex-col gap-2 border-t pt-3">
        <div class="flex items-center justify-between">
          <span class="label-xs text-text-muted block">
            {i18n.t.sessionTimeoutTitle}
          </span>
          <span class="font-proto text-teal text-smaller font-bold">
            {selectedPresetLabel}
          </span>
        </div>

        <!-- Segmented Preset Buttons -->
        <div class="grid grid-cols-6 gap-1">
          {#each timeoutPresets as opt (opt.value)}
            <button
              type="button"
              onclick={() => (sessionTimeout = opt.value)}
              title={opt.label}
              class="sharp-card font-proto text-smaller flex h-7 items-center justify-center border transition-colors {sessionTimeout ===
              opt.value
                ? 'border-teal bg-teal/15 text-teal font-bold'
                : 'border-line bg-bg-app hover:border-text-dim text-text-muted hover:text-text-base'}"
            >
              {opt.short}
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <div class="border-line/40 mt-3 flex items-center justify-between gap-2 border-t pt-2.5">
      <div class="min-h-5 flex items-center">
        {#if error}<p class="text-expense font-proto text-smaller">{error}</p>{/if}
        {#if saved}<p class="text-income font-proto text-smaller">{i18n.t.settingsSavedOk}</p>{/if}
      </div>
      <Button
        variant="primary"
        class="font-proto text-small h-8 px-3 font-bold"
        disabled={!canSave}
        onclick={handleSaveSettings}
      >
        {i18n.t.saveChanges}
      </Button>
    </div>
  </div>

  <p class="text-text-dim border-line/30 text-smaller font-aux mt-auto border-t pt-2">
    {i18n.t.inactivityHeartbeatNote}
  </p>
</Card>

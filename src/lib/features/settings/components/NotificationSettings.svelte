<script lang="ts">
  import { Badge } from '$lib/components/ui';
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { getPref, setPref } from '$lib/core/state/prefs';
  

  let notifToggles = $state({
    due: true,
    fx: true,
    spike: true,
    integrity: true,
    autoUpdate: true,
  });

  function loadLocalPrefs() {
    const n = getPref('finnca_notif_toggles', {});
    notifToggles = { ...notifToggles, ...n };
  }

  onMount(loadLocalPrefs);

  function saveNotifToggles() {
    setPref('finnca_notif_toggles', notifToggles);
  }

  function toggle(key: keyof typeof notifToggles) {
    notifToggles[key] = !notifToggles[key];
    saveNotifToggles();
  }
</script>

<div class="flex flex-col border-b border-line pb-6 mb-6 last:border-0 last:mb-0 last:pb-0">
  <div class="flex items-start justify-between mb-4">
    <div class="flex flex-col gap-1">
    <h3 class="font-proto text-text-strong text-small tracking-widest uppercase">{i18n.t.notificationsTitle}</h3>
      <p class="text-text-dim text-smaller font-aux">
        {i18n.t.notifStoredLocally}
      </p>
    </div>
    <Badge size="m" tone="neutral">{i18n.t.badgeAlerts}</Badge>
  </div>
  <div>
    <div class="flex flex-col gap-2">
      <button
        type="button"
        onclick={() => toggle('due')}
        class="border flex items-center justify-between p-2.5 transition-colors {notifToggles.due ? 'border-teal bg-teal/5' : 'border-line bg-bg-app hover:border-text-dim'}"
      >
        <span class="font-proto text-small {notifToggles.due ? 'text-teal font-bold' : 'text-text-strong'} tracking-wider uppercase text-left">
          {i18n.t.notifDueDatesLabel}
        </span>
        <div class="flex items-center gap-1 font-proto text-smaller font-bold">
          <span class="px-2 py-0.5 border {notifToggles.due ? 'border-teal bg-teal text-bg-app' : 'border-transparent text-text-dim'}">ON</span>
          <span class="px-2 py-0.5 border {!notifToggles.due ? 'border-line bg-bg-row-active text-text-base' : 'border-transparent text-text-dim'}">OFF</span>
        </div>
      </button>

      <button
        type="button"
        onclick={() => toggle('fx')}
        class="border flex items-center justify-between p-2.5 transition-colors {notifToggles.fx ? 'border-teal bg-teal/5' : 'border-line bg-bg-app hover:border-text-dim'}"
      >
        <span class="font-proto text-small {notifToggles.fx ? 'text-teal font-bold' : 'text-text-strong'} tracking-wider uppercase text-left">
          {i18n.t.notifFxAlertsLabel}
        </span>
        <div class="flex items-center gap-1 font-proto text-smaller font-bold">
          <span class="px-2 py-0.5 border {notifToggles.fx ? 'border-teal bg-teal text-bg-app' : 'border-transparent text-text-dim'}">ON</span>
          <span class="px-2 py-0.5 border {!notifToggles.fx ? 'border-line bg-bg-row-active text-text-base' : 'border-transparent text-text-dim'}">OFF</span>
        </div>
      </button>

      <button
        type="button"
        onclick={() => toggle('spike')}
        class="border flex items-center justify-between p-2.5 transition-colors {notifToggles.spike ? 'border-teal bg-teal/5' : 'border-line bg-bg-app hover:border-text-dim'}"
      >
        <span class="font-proto text-small {notifToggles.spike ? 'text-teal font-bold' : 'text-text-strong'} tracking-wider uppercase text-left">
          {i18n.t.notifExpenseSpikeLabel}
        </span>
        <div class="flex items-center gap-1 font-proto text-smaller font-bold">
          <span class="px-2 py-0.5 border {notifToggles.spike ? 'border-teal bg-teal text-bg-app' : 'border-transparent text-text-dim'}">ON</span>
          <span class="px-2 py-0.5 border {!notifToggles.spike ? 'border-line bg-bg-row-active text-text-base' : 'border-transparent text-text-dim'}">OFF</span>
        </div>
      </button>

      <button
        type="button"
        onclick={() => toggle('integrity')}
        class="border flex items-center justify-between p-2.5 transition-colors {notifToggles.integrity ? 'border-teal bg-teal/5' : 'border-line bg-bg-app hover:border-text-dim'}"
      >
        <span class="font-proto text-small {notifToggles.integrity ? 'text-teal font-bold' : 'text-text-strong'} tracking-wider uppercase text-left">
          {i18n.t.notifLedgerIntegrityLabel}
        </span>
        <div class="flex items-center gap-1 font-proto text-smaller font-bold">
          <span class="px-2 py-0.5 border {notifToggles.integrity ? 'border-teal bg-teal text-bg-app' : 'border-transparent text-text-dim'}">ON</span>
          <span class="px-2 py-0.5 border {!notifToggles.integrity ? 'border-line bg-bg-row-active text-text-base' : 'border-transparent text-text-dim'}">OFF</span>
        </div>
      </button>

      <button
        type="button"
        onclick={() => toggle('autoUpdate')}
        class="border flex items-center justify-between p-2.5 transition-colors {notifToggles.autoUpdate ? 'border-teal bg-teal/5' : 'border-line bg-bg-app hover:border-text-dim'}"
      >
        <div class="flex flex-col items-start text-left">
          <span class="font-proto text-small {notifToggles.autoUpdate ? 'text-teal font-bold' : 'text-text-strong'} tracking-wider uppercase">
            {i18n.t.notifAutoUpdateLabel}
          </span>
          <span class="font-aux text-text-muted text-smaller mt-0.5">
            {i18n.t.notifAutoUpdateDesc}
          </span>
        </div>
        <div class="flex items-center gap-1 font-proto text-smaller font-bold shrink-0 ml-4">
          <span class="px-2 py-0.5 border {notifToggles.autoUpdate ? 'border-teal bg-teal text-bg-app' : 'border-transparent text-text-dim'}">ON</span>
          <span class="px-2 py-0.5 border {!notifToggles.autoUpdate ? 'border-line bg-bg-row-active text-text-base' : 'border-transparent text-text-dim'}">OFF</span>
        </div>
      </button>
    </div>
  </div>

  </div>

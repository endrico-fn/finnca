<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { getPref, setPref } from '$lib/core/state/prefs';
  import { Card } from '$lib/components/ui';

  let notifToggles = $state({
    due: true,
    fx: true,
    spike: true,
    integrity: true,
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

<Card title={i18n.t.notificationsTitle} badge={i18n.t.badgeAlerts} class="justify-between">
  <div>
    <div class="flex flex-col gap-2">
      <label
        class="border-line/40 bg-bg-app hover:border-text-dim group flex cursor-pointer items-center justify-between border p-2.5 transition-colors"
      >
        <div class="flex items-center gap-2.5">
          <input
            type="checkbox"
            checked={notifToggles.due}
            onchange={() => toggle('due')}
            class="accent-teal size-3.5"
          />
          <span
            class="font-proto text-text-strong group-hover:text-text-white text-small transition-colors"
          >
            {i18n.t.notifDueDatesLabel}
          </span>
        </div>
        <span
          class="font-proto text-smaller {notifToggles.due
            ? 'text-teal font-bold'
            : 'text-text-muted'}"
        >
          {notifToggles.due ? i18n.t.enabledLabel : i18n.t.disabledLabel}
        </span>
      </label>

      <label
        class="border-line/40 bg-bg-app hover:border-text-dim group flex cursor-pointer items-center justify-between border p-2.5 transition-colors"
      >
        <div class="flex items-center gap-2.5">
          <input
            type="checkbox"
            checked={notifToggles.fx}
            onchange={() => toggle('fx')}
            class="accent-teal size-3.5"
          />
          <span
            class="font-proto text-text-strong group-hover:text-text-white text-small transition-colors"
          >
            {i18n.t.notifFxAlertsLabel}
          </span>
        </div>
        <span
          class="font-proto text-smaller {notifToggles.fx
            ? 'text-teal font-bold'
            : 'text-text-muted'}"
        >
          {notifToggles.fx ? i18n.t.enabledLabel : i18n.t.disabledLabel}
        </span>
      </label>

      <label
        class="border-line/40 bg-bg-app hover:border-text-dim group flex cursor-pointer items-center justify-between border p-2.5 transition-colors"
      >
        <div class="flex items-center gap-2.5">
          <input
            type="checkbox"
            checked={notifToggles.spike}
            onchange={() => toggle('spike')}
            class="accent-teal size-3.5"
          />
          <span
            class="font-proto text-text-strong group-hover:text-text-white text-small transition-colors"
          >
            {i18n.t.notifExpenseSpikeLabel}
          </span>
        </div>
        <span
          class="font-proto text-smaller {notifToggles.spike
            ? 'text-teal font-bold'
            : 'text-text-muted'}"
        >
          {notifToggles.spike ? i18n.t.enabledLabel : i18n.t.disabledLabel}
        </span>
      </label>

      <label
        class="border-line/40 bg-bg-app hover:border-text-dim group flex cursor-pointer items-center justify-between border p-2.5 transition-colors"
      >
        <div class="flex items-center gap-2.5">
          <input
            type="checkbox"
            checked={notifToggles.integrity}
            onchange={() => toggle('integrity')}
            class="accent-teal size-3.5"
          />
          <span
            class="font-proto text-text-strong group-hover:text-text-white text-small transition-colors"
          >
            {i18n.t.notifLedgerIntegrityLabel}
          </span>
        </div>
        <span
          class="font-proto text-smaller {notifToggles.integrity
            ? 'text-teal font-bold'
            : 'text-text-muted'}"
        >
          {notifToggles.integrity ? i18n.t.enabledLabel : i18n.t.disabledLabel}
        </span>
      </label>
    </div>
  </div>

  <p class="text-text-muted text-smaller font-aux border-line/40 mt-auto border-t pt-2">
    {i18n.t.notifStoredLocally}
  </p>
</Card>

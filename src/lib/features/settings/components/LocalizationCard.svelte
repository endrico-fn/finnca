<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { getPref, setPref } from '$lib/core/state/prefs';
  import { Card, Badge } from '$lib/components/ui';

  let numberFormat = $state<'dot' | 'comma'>('comma');
  let dateFormat = $state<'iso' | 'slash'>('iso');

  onMount(() => {
    numberFormat = getPref('finnca_num_format', 'comma');
    dateFormat = getPref('finnca_date_format', 'iso');
  });

  function setNumberFormat(fmt: 'dot' | 'comma') {
    numberFormat = fmt;
    setPref('finnca_num_format', fmt);
  }

  function setDateFormat(fmt: 'iso' | 'slash') {
    dateFormat = fmt;
    setPref('finnca_date_format', fmt);
  }
</script>

<Card title="{i18n.t.language} & {i18n.t.displayPreferences}" badge={i18n.t.badgeLocalization}>
  <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
    <!-- Language Buttons -->
    <div>
      <p class="label-xs text-text-muted mb-1.5 block">{i18n.t.primaryLanguageDesc}</p>
      <div class="grid grid-cols-2 gap-2">
        <button
          type="button"
          class="sharp-card border p-2.5 text-left transition-colors {i18n.locale === 'en'
            ? 'border-teal/60 bg-bg-row-active'
            : 'border-line hover:border-text-dim bg-bg-app'}"
          onclick={() => {
            i18n.setLocale('en');
          }}
        >
          <div class="mb-1 flex items-center justify-between">
            <Badge size="m" tone="neutral">EN</Badge>
            {#if i18n.locale === 'en'}<span class="bg-teal size-1.5 animate-pulse"></span>{/if}
          </div>
          <div class="font-proto text-text-strong text-small font-bold">
            {i18n.t.langEnglish}
          </div>
        </button>

        <button
          type="button"
          class="sharp-card border p-2.5 text-left transition-colors {i18n.locale === 'id'
            ? 'border-teal/60 bg-bg-row-active'
            : 'border-line hover:border-text-dim bg-bg-app'}"
          onclick={() => {
            i18n.setLocale('id');
          }}
        >
          <div class="mb-1 flex items-center justify-between">
            <Badge size="m" tone="neutral">ID</Badge>
            {#if i18n.locale === 'id'}<span class="bg-teal size-1.5 animate-pulse"></span>{/if}
          </div>
          <div class="font-proto text-text-strong text-small font-bold">
            {i18n.t.langIndonesia}
          </div>
        </button>
      </div>
    </div>

    <!-- Number and Date Formatting Preferences -->
    <div class="flex flex-col gap-2">
      <div>
        <p class="label-xs text-text-muted mb-1.5 block">
          {i18n.t.numberFormatLabel}
        </p>
        <div class="grid grid-cols-2 gap-2">
          <button
            type="button"
            onclick={() => setNumberFormat('comma')}
            class="sharp-card font-proto text-smaller flex items-center justify-between border p-2 text-left {numberFormat ===
            'comma'
              ? 'border-teal text-text-white bg-bg-row-active'
              : 'border-line text-text-base hover:border-text-dim'}"
          >
            <span>1,234,567.89 <span class="text-text-muted font-proto text-smaller ml-1">[COMMA]</span></span>
            {#if numberFormat === 'comma'}<span class="text-teal font-bold">✓</span>{/if}
          </button>
          <button
            type="button"
            onclick={() => setNumberFormat('dot')}
            class="sharp-card font-proto text-smaller flex items-center justify-between border p-2 text-left {numberFormat ===
            'dot'
              ? 'border-teal text-text-white bg-bg-row-active'
              : 'border-line text-text-base hover:border-text-dim'}"
          >
            <span>1.234.567,89 <span class="text-text-muted font-proto text-smaller ml-1">[DOT]</span></span>
            {#if numberFormat === 'dot'}<span class="text-teal font-bold">✓</span>{/if}
          </button>
        </div>
      </div>

      <div>
        <p class="label-xs text-text-muted mb-1.5 block">
          {i18n.t.dateFormatLabel}
        </p>
        <div class="grid grid-cols-2 gap-2">
          <button
            type="button"
            onclick={() => setDateFormat('iso')}
            class="sharp-card font-proto text-smaller flex items-center justify-between border p-2 text-left {dateFormat ===
            'iso'
              ? 'border-teal text-text-white bg-bg-row-active'
              : 'border-line text-text-base hover:border-text-dim'}"
          >
            <span>YYYY-MM-DD <span class="text-text-muted font-proto text-smaller ml-1">[ISO]</span></span>
            {#if dateFormat === 'iso'}<span class="text-teal font-bold">✓</span>{/if}
          </button>
          <button
            type="button"
            onclick={() => setDateFormat('slash')}
            class="sharp-card font-proto text-smaller flex items-center justify-between border p-2 text-left {dateFormat ===
            'slash'
              ? 'border-teal text-text-white bg-bg-row-active'
              : 'border-line text-text-base hover:border-text-dim'}"
          >
            <span>DD/MM/YYYY <span class="text-text-muted font-proto text-smaller ml-1">[SLASH]</span></span>
            {#if dateFormat === 'slash'}<span class="text-teal font-bold">✓</span>{/if}
          </button>
        </div>
      </div>
    </div>
  </div>
</Card>

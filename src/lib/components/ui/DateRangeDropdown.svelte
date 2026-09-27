<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import Icon from './Icon.svelte';
  import DateRangeCalendarPicker from './DateRangeCalendarPicker.svelte';
  import {
    formatRangeDisplayLabel,
    getThisMonthRange,
    getLastMonthRange,
    getThisYearRange,
    getLast30DaysRange,
  } from './dateRangePresets';

  let {
    from = $bindable(),
    to = $bindable(),
    onChange,
    size = 'sm',
  } = $props<{
    from: string;
    to: string;
    onChange?: () => void;
    size?: 'sm' | 'md';
  }>();

  let open = $state(false);
  let containerEl: HTMLElement | null = $state(null);

  function applyPreset(presetFn: () => [string, string]) {
    const [s, e] = presetFn();
    from = s;
    to = e;
    open = false;
    onChange?.();
  }

  function setAllTime() {
    from = '';
    to = '';
    open = false;
    onChange?.();
  }

  function handleClickOutside(e: MouseEvent) {
    if (containerEl && !containerEl.contains(e.target as Node)) {
      open = false;
    }
  }

  onMount(() => {
    window.addEventListener('click', handleClickOutside);
    return () => window.removeEventListener('click', handleClickOutside);
  });

  const displayLabel = $derived(formatRangeDisplayLabel(from, to, i18n.t));
</script>

<div class="relative inline-block text-left" bind:this={containerEl}>
  <button
    type="button"
    onclick={() => (open = !open)}
    class="sharp-btn font-proto text-smaller inline-flex cursor-pointer items-center gap-2 border px-2.5 transition-all select-none {size ===
    'md'
      ? 'h-7'
      : 'h-6'} {open
      ? 'border-teal/60 bg-bg-row-active text-text-strong font-semibold'
      : 'border-line bg-bg-card text-text-strong hover:border-text-dim hover:text-text-white'}"
  >
    <span class="text-text-base inline-flex">
      <Icon name="calendar" size={10} />
    </span>
    <span class="font-semibold tracking-wide uppercase">{displayLabel}</span>
    <span
      class="text-text-muted inline-flex transition-transform duration-150 {open
        ? 'rotate-180'
        : ''}"
    >
      <Icon name="chev-down" size={8} />
    </span>
  </button>

  {#if open}
    <div
      class="bg-bg-card border-line font-proto text-smaller absolute right-0 z-50 mt-1 flex w-84 border select-none"
    >
      <div class="border-line bg-bg-app flex w-24 shrink-0 flex-col border-r">
        <button
          type="button"
          onclick={setAllTime}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {!from &&
          !to
            ? 'text-teal bg-teal/10 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.allTime}
        </button>
        <button
          type="button"
          onclick={() => applyPreset(getThisMonthRange)}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {displayLabel ===
          i18n.t.thisMonth
            ? 'text-teal bg-teal/10 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.thisMonth}
        </button>
        <button
          type="button"
          onclick={() => applyPreset(getLastMonthRange)}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {displayLabel ===
          i18n.t.lastMonth
            ? 'text-teal bg-teal/10 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.lastMonth}
        </button>
        <button
          type="button"
          onclick={() => applyPreset(getLast30DaysRange)}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {displayLabel ===
          i18n.t.last30Days
            ? 'text-teal bg-teal/10 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.last30Days}
        </button>
        <button
          type="button"
          onclick={() => applyPreset(getThisYearRange)}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {displayLabel ===
          i18n.t.thisYear
            ? 'text-teal bg-teal/10 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.thisYear}
        </button>
      </div>

      <DateRangeCalendarPicker
        bind:from
        bind:to
        {open}
        onComplete={() => (open = false)}
        {onChange}
      />
    </div>
  {/if}
</div>

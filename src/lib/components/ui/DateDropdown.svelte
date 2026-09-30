<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { buildCalendarDays, parseLocalDateParts } from '$lib/core/format/date';
  import Icon from './Icon.svelte';
  import Button from './Button.svelte';

  let {
    value = $bindable(),
    onChange,
    onSelect,
  } = $props<{
    value: string;
    onChange?: () => void;
    onSelect?: (v: string) => void;
  }>();

  let open = $state(false);
  let containerEl: HTMLElement | null = $state(null);

  function handleClickOutside(e: MouseEvent) {
    if (containerEl && !containerEl.contains(e.target as Node)) {
      open = false;
    }
  }

  onMount(() => {
    window.addEventListener('click', handleClickOutside);
    return () => window.removeEventListener('click', handleClickOutside);
  });

  function setToday() {
    const now = new Date();
    const y = now.getFullYear();
    const m = String(now.getMonth() + 1).padStart(2, '0');
    const d = String(now.getDate()).padStart(2, '0');
    value = `${y}-${m}-${d}`;
    onChange?.();
    onSelect?.(value);
  }

  function setEndOfLastMonth() {
    const now = new Date();
    const lastDay = new Date(now.getFullYear(), now.getMonth(), 0);
    const y = lastDay.getFullYear();
    const m = String(lastDay.getMonth() + 1).padStart(2, '0');
    const d = String(lastDay.getDate()).padStart(2, '0');
    value = `${y}-${m}-${d}`;
    onChange?.();
    onSelect?.(value);
  }

  function setEndOfLastYear() {
    const y = new Date().getFullYear() - 1;
    value = `${y}-12-31`;
    onChange?.();
    onSelect?.(value);
  }

  let viewYear = $state(new Date().getFullYear());
  let viewMonth = $state(new Date().getMonth());

  $effect(() => {
    if (open) {
      if (value) {
        const parts = parseLocalDateParts(value);
        if (parts) {
          viewYear = parts.y;
          viewMonth = parts.m - 1;
        }
      } else {
        viewYear = new Date().getFullYear();
        viewMonth = new Date().getMonth();
      }
    }
  });

  const calendarDays = $derived(buildCalendarDays(viewYear, viewMonth));
  const monthName = $derived(
    new Date(viewYear, viewMonth, 1)
      .toLocaleDateString(i18n.locale === 'id' ? 'id-ID' : 'en-US', {
        month: 'short',
        year: 'numeric',
      })
      .toUpperCase()
  );

  function prevMonth() {
    if (viewMonth === 0) {
      viewMonth = 11;
      viewYear--;
    } else {
      viewMonth--;
    }
  }

  function nextMonth() {
    if (viewMonth === 11) {
      viewMonth = 0;
      viewYear++;
    } else {
      viewMonth++;
    }
  }

  function selectDate(dateStr: string) {
    value = dateStr;
    open = false;
    onChange?.();
    onSelect?.(dateStr);
  }

  const displayLabel = $derived.by(() => {
    if (!value) return i18n.t.asOfToday;

    const now = new Date();
    const y = now.getFullYear();
    const m = String(now.getMonth() + 1).padStart(2, '0');
    const d = String(now.getDate()).padStart(2, '0');
    if (value === `${y}-${m}-${d}`) {
      return i18n.t.today || 'TODAY';
    }

    const lastDay = new Date(now.getFullYear(), now.getMonth(), 0);
    const lmy = lastDay.getFullYear();
    const lmm = String(lastDay.getMonth() + 1).padStart(2, '0');
    const lmd = String(lastDay.getDate()).padStart(2, '0');
    if (value === `${lmy}-${lmm}-${lmd}`) {
      return i18n.t.endOfLastMonth;
    }

    if (value === `${y - 1}-12-31`) return i18n.t.endOfLastYear;

    return value;
  });
</script>

<div class="relative inline-flex items-center text-left" bind:this={containerEl}>
  <button
    type="button"
    onclick={() => (open = !open)}
    class="sharp-btn font-proto text-smaller inline-flex h-6 cursor-pointer items-center gap-2 border px-2.5 transition-all select-none {open
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
          onclick={() => {
            setToday();
            open = false;
          }}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {displayLabel ===
          (i18n.t.today || 'TODAY')
            ? 'text-teal bg-teal/10 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.today || 'TODAY'}
        </button>
        <button
          type="button"
          onclick={() => {
            setEndOfLastMonth();
            open = false;
          }}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {displayLabel ===
          i18n.t.endOfLastMonth
            ? 'text-teal bg-teal/10 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.endOfLastMonth}
        </button>
        <button
          type="button"
          onclick={() => {
            setEndOfLastYear();
            open = false;
          }}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {displayLabel ===
          i18n.t.endOfLastYear
            ? 'text-teal bg-teal/10 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.endOfLastYear}
        </button>
      </div>

      <div class="flex-1 p-2">
        <div class="mb-2 flex gap-1">
          <div class="border-teal/60 bg-teal/10 text-teal flex-1 border px-1.5 py-1 text-center">
            {value || '--'}
          </div>
        </div>

        <div class="mb-2 flex items-center justify-between px-1">
          <Button
            variant="pager"
            size="icon"
            onclick={prevMonth}
            ariaLabel={i18n.t.prevMonth}
            title={i18n.t.prevMonth}><Icon name="chev-left" size={14} /></Button
          >
          <span class="text-text-strong tracking-widest">{monthName}</span>
          <Button
            variant="pager"
            size="icon"
            onclick={nextMonth}
            ariaLabel={i18n.t.nextMonth}
            title={i18n.t.nextMonth}><Icon name="chev-right" size={14} /></Button
          >
        </div>

        <div class="bg-line border-line grid grid-cols-7 gap-px border">
          {#each i18n.t.calendarDayInitials as dh (dh)}
            <div class="bg-bg-app text-text-dim text-smaller shrink-0 py-1 text-center">
              {dh}
            </div>
          {/each}
          {#each calendarDays as day (day.dateStr || day)}
            {@const isSelected = day.dateStr === value}
            <button
              type="button"
              onclick={() => selectDate(day.dateStr)}
              class="flex h-6 items-center justify-center transition-colors
                {isSelected
                ? 'bg-teal text-bg-app font-bold'
                : !day.isCurrentMonth
                  ? 'bg-bg-app text-text-dim/30'
                  : 'bg-bg-card hover:bg-line/30 text-text-base'}"
            >
              {day.dayNum}
            </button>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>

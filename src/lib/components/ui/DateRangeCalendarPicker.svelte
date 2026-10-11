<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { buildCalendarDays, parseLocalDateParts } from '$lib/core/format/date';
  import Icon from './Icon.svelte';
  import Button from './Button.svelte';

  let {
    from = $bindable(),
    to = $bindable(),
    open,
    onComplete,
    onChange,
  }: {
    from: string;
    to: string;
    open: boolean;
    onComplete: () => void;
    onChange?: () => void;
  } = $props();

  let activeInput = $state<'from' | 'to'>('from');
  let viewYear = $state(new Date().getFullYear());
  let viewMonth = $state(new Date().getMonth());

  $effect(() => {
    if (open) {
      const targetDate = activeInput === 'from' ? from : to;
      if (targetDate) {
        const parts = parseLocalDateParts(targetDate);
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
    if (activeInput === 'from') {
      from = dateStr;
      if (to && from > to) to = from;
      activeInput = 'to';
    } else {
      to = dateStr;
      if (from && to < from) from = to;
      onComplete();
    }
    onChange?.();
  }
</script>

<div class="flex-1 p-2">
  <div class="mb-2 flex gap-1">
    <button
      type="button"
      onclick={() => (activeInput = 'from')}
      class="flex-1 border px-1.5 py-1 text-center transition-colors {activeInput === 'from'
        ? 'border-teal/60 text-teal bg-teal/10'
        : 'border-line text-text-muted hover:border-text-dim hover:bg-line/10'}"
    >
      {i18n.t.pickFrom}: {from || '--'}
    </button>
    <button
      type="button"
      onclick={() => (activeInput = 'to')}
      class="flex-1 border px-1.5 py-1 text-center transition-colors {activeInput === 'to'
        ? 'border-teal/60 text-teal bg-teal/10'
        : 'border-line text-text-muted hover:border-text-dim hover:bg-line/10'}"
    >
      {i18n.t.pickTo}: {to || '--'}
    </button>
  </div>

  <div class="mb-2 flex items-center justify-between px-1">
    <Button
      variant="pager"
      size="icon"
      onclick={prevMonth}
      ariaLabel={i18n.t.prevMonth}
      title={i18n.t.prevMonth}
    >
      <Icon name="chev-left" size={14} />
    </Button>
    <span class="text-text-strong tracking-widest">{monthName}</span>
    <Button
      variant="pager"
      size="icon"
      onclick={nextMonth}
      ariaLabel={i18n.t.nextMonth}
      title={i18n.t.nextMonth}
    >
      <Icon name="chev-right" size={14} />
    </Button>
  </div>

  <div class="bg-line border-line grid grid-cols-7 gap-px border">
    {#each i18n.t.calendarDayInitials as dh (dh)}
      <div class="bg-bg-app text-text-dim text-smaller shrink-0 py-1 text-center">
        {dh}
      </div>
    {/each}
    {#each calendarDays as day (day.dateStr || day)}
      {@const isSelected = day.dateStr === from || day.dateStr === to}
      {@const isInRange = from && to && day.dateStr > from && day.dateStr < to}
      <button
        type="button"
        onclick={() => selectDate(day.dateStr)}
        class="flex h-6 items-center justify-center transition-colors
          {isSelected
          ? 'bg-teal text-bg-app font-bold'
          : isInRange
            ? 'bg-teal/15 text-teal'
            : !day.isCurrentMonth
              ? 'bg-bg-app text-text-dim/30'
              : 'bg-bg-card hover:bg-line/30 text-text-base'}"
      >
        {day.dayNum}
      </button>
    {/each}
  </div>
</div>

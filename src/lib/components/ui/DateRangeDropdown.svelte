<script lang="ts">
  import { i18n } from '$lib/i18n.svelte';
  import { buildCalendarDays } from '$lib/accounting/finance';
  import Icon from './Icon.svelte';
  import Button from './Button.svelte';

  let { from = $bindable(), to = $bindable() } = $props<{
    from: string;
    to: string;
  }>();

  let open = $state(false);

  function setThisMonth() {
    const now = new Date();
    const y = now.getFullYear();
    const m = String(now.getMonth() + 1).padStart(2, '0');
    const lastDay = new Date(y, now.getMonth() + 1, 0).getDate();
    from = `${y}-${m}-01`;
    to = `${y}-${m}-${String(lastDay).padStart(2, '0')}`;
  }

  function setLastMonth() {
    const now = new Date();
    const y = now.getMonth() === 0 ? now.getFullYear() - 1 : now.getFullYear();
    const m = now.getMonth() === 0 ? 12 : now.getMonth();
    const mStr = String(m).padStart(2, '0');
    const lastDay = new Date(y, m, 0).getDate();
    from = `${y}-${mStr}-01`;
    to = `${y}-${mStr}-${String(lastDay).padStart(2, '0')}`;
  }

  function setThisYear() {
    const y = new Date().getFullYear();
    from = `${y}-01-01`;
    to = `${y}-12-31`;
  }

  function setAllTime() {
    from = '';
    to = '';
  }

  let activeInput = $state<'from' | 'to'>('from');
  let viewYear = $state(new Date().getFullYear());
  let viewMonth = $state(new Date().getMonth());

  $effect(() => {
    if (open) {
      const targetDate = activeInput === 'from' ? from : to;
      if (targetDate) {
        const d = new Date(targetDate);
        if (!isNaN(d.getTime())) {
          viewYear = d.getFullYear();
          viewMonth = d.getMonth();
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
      if (to && from > to) to = from; // auto correct if from is after to
      activeInput = 'to';
    } else {
      to = dateStr;
      if (from && to < from) from = to; // auto correct if to is before from
      open = false; // complete selection
    }
  }

  const displayLabel = $derived.by(() => {
    if (!from && !to) return i18n.t.allTime;

    const now = new Date();
    const y = now.getFullYear();
    const m = now.getMonth() + 1;
    const mStr = String(m).padStart(2, '0');
    const thisMonthLast = String(new Date(y, m, 0).getDate()).padStart(2, '0');
    if (from === `${y}-${mStr}-01` && to === `${y}-${mStr}-${thisMonthLast}`) {
      return i18n.t.thisMonth;
    }

    const prevY = now.getMonth() === 0 ? y - 1 : y;
    const prevM = now.getMonth() === 0 ? 12 : now.getMonth();
    const prevMStr = String(prevM).padStart(2, '0');
    const prevMonthLast = String(new Date(prevY, prevM, 0).getDate()).padStart(2, '0');
    if (from === `${prevY}-${prevMStr}-01` && to === `${prevY}-${prevMStr}-${prevMonthLast}`) {
      return i18n.t.lastMonth;
    }

    if (from === `${y}-01-01` && to === `${y}-12-31`) return i18n.t.thisYear;

    if (from && to) return `${from} → ${to}`;

    return `${from || '...'} → ${to || '...'}`;
  });
</script>

<div class="relative inline-block text-left">
  <!-- Trigger Button -->
  <button
    type="button"
    onclick={() => (open = !open)}
    class="sharp-btn font-proto inline-flex h-6 cursor-pointer items-center gap-2 border px-2.5 text-[10px] transition-all select-none {open
      ? 'border-teal bg-teal/5 text-teal font-semibold'
      : 'border-line bg-bg-card text-text-strong hover:border-text-dim hover:text-text-white'}"
  >
    <span class="inline-flex {open ? 'text-teal' : 'text-text-icon'}">
      <Icon name="calendar" size={10} />
    </span>
    <span class="font-bold tracking-wide uppercase">{displayLabel}</span>
    <span
      class="text-text-muted inline-flex transition-transform duration-150 {open
        ? 'text-teal rotate-180'
        : ''}"
    >
      <Icon name="chev-down" size={8} />
    </span>
  </button>

  {#if open}
    <!-- Invisible overlay to close on outside click -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-40" onclick={() => (open = false)}></div>

    <div
      class="bg-bg-card border-line font-proto absolute right-0 z-50 mt-1 flex w-[330px] border text-[10px] select-none"
    >
      <!-- Quick Filters Sidebar -->
      <div class="border-line bg-bg-app flex w-24 shrink-0 flex-col border-r">
        <button
          type="button"
          onclick={() => {
            setAllTime();
            open = false;
          }}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {!from &&
          !to
            ? 'text-teal bg-teal/5 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.allTime}
        </button>
        <button
          type="button"
          onclick={() => {
            setThisMonth();
            open = false;
          }}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {displayLabel ===
          i18n.t.thisMonth
            ? 'text-teal bg-teal/5 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.thisMonth}
        </button>
        <button
          type="button"
          onclick={() => {
            setLastMonth();
            open = false;
          }}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {displayLabel ===
          i18n.t.lastMonth
            ? 'text-teal bg-teal/5 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.lastMonth}
        </button>
        <button
          type="button"
          onclick={() => {
            setThisYear();
            open = false;
          }}
          class="border-line/50 hover:bg-line/20 border-b px-2.5 py-2 text-left transition-colors {displayLabel ===
          i18n.t.thisYear
            ? 'text-teal bg-teal/5 font-semibold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.thisYear}
        </button>
      </div>

      <!-- Calendar Picker -->
      <div class="flex-1 p-2">
        <div class="mb-2 flex gap-1">
          <button
            type="button"
            onclick={() => (activeInput = 'from')}
            class="flex-1 border px-1.5 py-1 text-center transition-colors {activeInput === 'from'
              ? 'border-teal text-teal bg-teal/5'
              : 'border-line text-text-muted hover:border-text-dim hover:bg-line/10'}"
          >
            FR: {from || '--'}
          </button>
          <button
            type="button"
            onclick={() => (activeInput = 'to')}
            class="flex-1 border px-1.5 py-1 text-center transition-colors {activeInput === 'to'
              ? 'border-teal text-teal bg-teal/5'
              : 'border-line text-text-muted hover:border-text-dim hover:bg-line/10'}"
          >
            TO: {to || '--'}
          </button>
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
          {#each ['SU', 'MO', 'TU', 'WE', 'TH', 'FR', 'SA'] as dh (dh)}
            <div class="bg-bg-app text-text-dim shrink-0 py-1 text-center text-[8px]">
              {dh}
            </div>
          {/each}
          {#each calendarDays as day (day.dateStr || day)}
            {@const isSelected = day.dateStr === from || day.dateStr === to}
            {@const isInRange = from && to && day.dateStr > from && day.dateStr < to}
            <button
              type="button"
              onclick={() => selectDate(day.dateStr)}
              class="flex h-5.5 items-center justify-center transition-colors
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
    </div>
  {/if}
</div>

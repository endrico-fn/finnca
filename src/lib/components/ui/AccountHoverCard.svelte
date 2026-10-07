<script lang="ts">
  import { fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import Badge from './Badge.svelte';
  import MiniSparkline from '$lib/components/charts/MiniSparkline.svelte';
  import type { BadgeTone } from './badgeTone';

  let {
    account,
    directBalance = 0,
    rollupBalance = 0,
    currency = 'IDR',
    sparklineData = undefined,
    children,
    class: className = '',
  }: {
    account: {
      code: string;
      name: string;
      account_type: string;
      placeholder?: boolean;
      hidden?: boolean;
      currency?: string;
    };
    directBalance?: number;
    rollupBalance?: number;
    currency?: string;
    sparklineData?: number[];
    children?: import('svelte').Snippet;
    class?: string;
  } = $props();

  let visible = $state(false);
  let containerEl: HTMLElement | null = $state(null);
  let coords = $state({ top: 0, left: 0, openUpward: true });
  let timer: ReturnType<typeof setTimeout> | null = null;

  const sparkPoints = $derived(
    sparklineData && sparklineData.length >= 2
      ? sparklineData
      : [rollupBalance * 0.96, rollupBalance * 0.98, rollupBalance * 1.01, rollupBalance]
  );

  function handleMouseEnter() {
    timer = setTimeout(() => {
      if (!containerEl) return;
      const rect = containerEl.getBoundingClientRect();
      const openUpward = rect.bottom + 180 > window.innerHeight;
      coords = {
        top: openUpward ? rect.top - 8 : rect.bottom + 8,
        left: Math.min(Math.max(16, rect.left), window.innerWidth - 280),
        openUpward,
      };
      visible = true;
    }, 200);
  }

  function handleMouseLeave() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    visible = false;
  }

  const badgeTone = $derived<BadgeTone>(
    (account.account_type.toLowerCase() as BadgeTone) || 'neutral'
  );
</script>

<div
  bind:this={containerEl}
  onmouseenter={handleMouseEnter}
  onmouseleave={handleMouseLeave}
  class="inline-contents {className}"
  role="presentation"
>
  {@render children?.()}
</div>

{#if visible}
  <div
    transition:fade={{ duration: 100, easing: cubicOut }}
    style="top: {coords.openUpward ? 'auto' : `${coords.top}px`}; bottom: {coords.openUpward
      ? `${typeof window !== 'undefined' ? window.innerHeight - coords.top : 0}px`
      : 'auto'}; left: {coords.left}px;"
    class="border-line bg-bg-card font-proto pointer-events-none fixed z-[var(--z-tooltip)] flex w-64 flex-col gap-2 border p-3 shadow-xl select-none"
  >
    <div class="border-line flex items-center justify-between border-b pb-2">
      <div class="flex items-center gap-1.5">
        <span class="text-text-muted text-smaller">{account.code}</span>
        <Badge tone={badgeTone} size="s">
          {account.account_type}
        </Badge>
        {#if account.placeholder}
          <Badge tone="neutral" size="s">{i18n.t.badgePh}</Badge>
        {/if}
      </div>
      <span class="text-text-dim text-smaller tracking-wider uppercase">
        {i18n.t.accountDetailsHover}
      </span>
    </div>

    <div class="flex flex-col">
      <span class="text-text-white font-aux text-small truncate font-medium">
        {account.name}
      </span>
    </div>

    <div class="bg-bg-app border-line/60 text-smaller flex flex-col gap-1 border p-2">
      <div class="flex items-center justify-between gap-3">
        <span class="text-text-dim uppercase">{i18n.t.rollupBalance}:</span>
        <span class="text-text-white font-bold tabular-nums">
          {formatMinorToDisplay(rollupBalance, account.currency || currency)}
        </span>
      </div>
      <div class="flex items-center justify-between gap-3">
        <span class="text-text-dim uppercase">{i18n.t.directBalance}:</span>
        <span class="text-text-dim tabular-nums">
          {formatMinorToDisplay(directBalance, account.currency || currency)}
        </span>
      </div>
    </div>

    <div class="border-line/40 flex items-center justify-between border-t pt-1.5">
      <span class="font-proto text-text-dim text-smaller uppercase">30D TREND</span>
      <MiniSparkline
        data={sparkPoints}
        width={110}
        height={20}
        color={account.account_type === 'EXPENSE' ? 'var(--color-expense)' : 'var(--color-income)'}
      />
    </div>
  </div>
{/if}

<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import Button from './Button.svelte';
  import Icon from './Icon.svelte';

  let {
    label = '',
    variant = 'boxed',
    onPrev,
    onNext,
    onToday = null,
    iconSize = 12,
  }: {
    label?: string;
    variant?: 'boxed' | 'bare';
    onPrev: () => void | Promise<void>;
    onNext: () => void | Promise<void>;
    onToday?: (() => void | Promise<void>) | null;
    iconSize?: number;
  } = $props();
</script>

{#if variant === 'boxed'}
  <div class="border-line bg-bg-card flex h-7 items-center gap-1 border px-1">
    <Button
      variant="pager"
      size="icon"
      onclick={onPrev}
      ariaLabel={i18n.t.prevMonth}
      title={i18n.t.prevMonth}
    >
      <Icon name="chev-left" size={iconSize} />
    </Button>
    <span
      class="font-proto text-text-strong text-smaller min-w-28 text-center font-bold tracking-widest tabular-nums"
    >
      {label}
    </span>
    <Button
      variant="pager"
      size="icon"
      onclick={onNext}
      ariaLabel={i18n.t.nextMonth}
      title={i18n.t.nextMonth}
    >
      <Icon name="chev-right" size={iconSize} />
    </Button>
  </div>
{:else}
  <div class="flex items-center gap-1.5">
    <Button
      variant="pager"
      size="icon"
      onclick={onPrev}
      ariaLabel={i18n.t.prevMonth}
      title={i18n.t.prevMonth}
    >
      <Icon name="chev-left" size={iconSize} />
    </Button>
    {#if onToday}
      <Button variant="pager" onclick={onToday}>
        {i18n.t.today}
      </Button>
    {/if}
    <Button
      variant="pager"
      size="icon"
      onclick={onNext}
      ariaLabel={i18n.t.nextMonth}
      title={i18n.t.nextMonth}
    >
      <Icon name="chev-right" size={iconSize} />
    </Button>
  </div>
{/if}

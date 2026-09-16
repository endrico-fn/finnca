<script lang="ts">
  import type { Snippet } from 'svelte';
  import Badge from './Badge.svelte';

  let {
    children,
    title,
    count,
    description,
    header,
    badge,
    badgeTone = 'neutral',
    actions,
    value,
    valueClass = 'text-text-base',
    divided = false,
    padding = true,
    borderHeader = false,
    class: className = '',
  }: {
    children: Snippet;
    title?: string;
    count?: number | string;
    description?: string;
    header?: Snippet;
    badge?: string;
    badgeTone?: 'neutral' | 'ok' | 'err' | 'warn';
    actions?: Snippet;
    value?: string;
    valueClass?: string;
    divided?: boolean;
    padding?: boolean;
    borderHeader?: boolean;
    class?: string;
  } = $props();

  const bodyCls = $derived(
    divided
      ? 'divide-line/40 divide-y overflow-y-auto px-3 pt-1.5 pb-2.5'
      : padding
        ? 'px-3 pt-2 pb-2.5'
        : ''
  );

  const valueCls = $derived(
    `font-proto text-medium leading-none font-bold whitespace-nowrap tabular-nums ${valueClass}`
  );
</script>

<section class="sharp-card flex flex-col overflow-hidden {className}">
  {#if title || header || badge || value || actions}
    <div
      class="flex shrink-0 items-center justify-between gap-2 px-3 pt-2.5 {borderHeader || divided
        ? 'border-line/60 border-b pb-2'
        : 'pb-0'}"
    >
      {#if title}
        <div class="flex min-w-0 flex-col">
          <div class="flex min-h-6 min-w-0 shrink-0 items-center gap-1.5">
            <p class="label-title truncate leading-none uppercase">{title}</p>
            {#if count !== undefined}
              <span class="text-text-dim font-proto text-small font-normal tabular-nums">{count}</span>
            {/if}
          </div>
          {#if description}
            <p class="text-text-muted text-smaller font-aux mt-0.5 leading-tight">{description}</p>
          {/if}
        </div>
      {/if}
      {#if header}
        <div
          class="flex min-w-0 items-center gap-2 {title
            ? 'flex-1 justify-end'
            : 'flex-1 justify-between'}"
        >
          {@render header()}
          {#if actions}
            {@render actions()}
          {/if}
        </div>
      {:else if value}
        <div class={title ? '' : 'flex w-full justify-end'}>
          <span class="{valueCls} {title ? 'shrink-0' : ''}">
            {value}
          </span>
        </div>
      {:else if badge || actions}
        <div class="flex items-center gap-2 {title ? 'shrink-0' : 'w-full justify-end'}">
          {#if badge}
            <Badge size="m" tone={badgeTone}>{badge}</Badge>
          {/if}
          {#if actions}
            {@render actions()}
          {/if}
        </div>
      {/if}
    </div>
  {/if}

  <div class="flex min-h-0 flex-1 flex-col {bodyCls}">
    {@render children()}
  </div>
</section>

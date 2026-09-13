<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    children,
    title,
    description,
    header,
    badge,
    badgeTone = 'neutral',
    value,
    valueClass = 'text-text-base',
    divided = false,
    padding = true,
    class: className = '',
  }: {
    children: Snippet;
    title?: string;
    description?: string;
    header?: Snippet;
    badge?: string;
    badgeTone?: 'neutral' | 'ok' | 'err' | 'warn';
    value?: string;
    valueClass?: string;
    divided?: boolean;
    padding?: boolean;
    class?: string;
  } = $props();

  const bodyCls = $derived(
    divided
      ? 'divide-line/40 divide-y overflow-y-auto px-3 pt-1.5 pb-2.5'
      : padding
        ? 'px-3 pt-2 pb-2.5'
        : ''
  );

  const badgeCls = $derived(
    `font-proto text-smaller border px-1.5 py-0.5 leading-none tracking-wider uppercase ${badgeTone === 'ok' ? 'badge-ok' : badgeTone === 'err' ? 'badge-err' : badgeTone === 'warn' ? 'badge-warn' : 'border-line bg-bg-app text-text-muted'}`
  );

  const valueCls = $derived(
    `font-proto text-medium leading-none font-bold whitespace-nowrap tabular-nums ${valueClass}`
  );
</script>

<section class="sharp-card flex flex-col overflow-hidden {className}">
  {#if title || header || badge || value}
    <div class="flex shrink-0 items-center justify-between gap-2 px-3 pt-2.5 pb-0">
      {#if title}
        <div class="flex min-h-6 min-w-0 shrink-0 items-center">
          <p class="label-title truncate leading-none uppercase">{title}</p>
          {#if description}
            <p class="text-text-muted text-smaller font-proto mt-1 leading-tight">{description}</p>
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
        </div>
      {:else if value}
        <div class={title ? '' : 'flex w-full justify-end'}>
          <span class="{valueCls} {title ? 'shrink-0' : ''}">
            {value}
          </span>
        </div>
      {:else if badge}
        <div class={title ? '' : 'flex w-full justify-end'}>
          <span class={badgeCls}>
            {badge}
          </span>
        </div>
      {/if}
    </div>
  {/if}

  <div class="flex min-h-0 flex-1 flex-col {bodyCls}">
    {@render children()}
  </div>
</section>

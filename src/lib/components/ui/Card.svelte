<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    children,
    title,
    description,
    header,
    badge,
    badgeTone = 'neutral',
    padding = true,
    class: className = '',
  }: {
    children: Snippet;
    title?: string;
    description?: string;
    header?: Snippet;
    badge?: string;
    badgeTone?: 'neutral' | 'ok' | 'err' | 'warn' | 'info';
    padding?: boolean;
    class?: string;
  } = $props();
</script>

<section class="sharp-card flex flex-col overflow-hidden {className}">
  {#if title || header || badge}
    <div class="flex shrink-0 items-start justify-between px-3 pt-2.5 pb-0">
      {#if title}
        <div class="min-w-0">
          <p class="label-title truncate leading-none uppercase">{title}</p>
          {#if description}
            <p class="text-text-muted mt-1 text-[10px] leading-tight">{description}</p>
          {/if}
        </div>
        {#if header}
          <div class="flex items-center gap-2">
            {@render header()}
          </div>
        {:else if badge}
          <span
            class="font-proto border px-1.5 py-0.5 text-[9px] uppercase leading-none tracking-wider {badgeTone === 'ok'
              ? 'badge-ok'
              : badgeTone === 'err'
                ? 'badge-err'
                : badgeTone === 'warn'
                  ? 'badge-warn'
                  : 'border-line bg-bg-app text-text-muted'}"
          >
            {badge}
          </span>
        {/if}
      {:else if header}
        <div class="flex flex-1 min-w-0 items-center justify-between">
          {@render header()}
        </div>
      {:else if badge}
        <div class="flex w-full justify-end">
          <span
            class="font-proto border px-1.5 py-0.5 text-[9px] uppercase leading-none tracking-wider {badgeTone === 'ok'
              ? 'badge-ok'
              : badgeTone === 'err'
                ? 'badge-err'
                : badgeTone === 'warn'
                  ? 'badge-warn'
                  : 'border-line bg-bg-app text-text-muted'}"
          >
            {badge}
          </span>
        </div>
      {/if}
    </div>
  {/if}

  <div class="flex min-h-0 flex-1 flex-col {padding ? 'px-3 pt-2 pb-2.5' : ''}">
    {@render children()}
  </div>
</section>

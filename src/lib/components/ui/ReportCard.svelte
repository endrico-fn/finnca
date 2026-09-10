<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    title,
    description,
    value,
    children,
    headerRight,
    valueClass = 'text-text-base',
    class: className = '',
  }: {
    title: string;
    description?: string;
    value?: string;
    children?: Snippet;
    headerRight?: Snippet;
    valueClass?: string;
    class?: string;
  } = $props();
</script>

<div class="sharp-card flex min-h-0 flex-col overflow-hidden {className}">
  {#if title || headerRight || value}
    <div class="flex shrink-0 items-start justify-between px-3 pt-2.5 pb-0">
      <div>
        <p class="label-title truncate leading-none">{title}</p>
        {#if description}
          <p class="text-text-muted mt-1 text-[10px]">
            {description}
          </p>
        {/if}
      </div>
      {#if headerRight}
        <div class="flex items-center gap-2">
          {@render headerRight()}
        </div>
      {:else if value}
        <span class="font-proto text-[14px] font-bold leading-none {valueClass}">
          {value}
        </span>
      {/if}
    </div>
  {/if}
  <div class="divide-line/40 flex min-h-0 flex-1 flex-col divide-y overflow-y-auto px-3 pt-1.5 pb-2.5">
    {@render children?.()}
  </div>
</div>

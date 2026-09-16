<script lang="ts">
  let {
    label,
    selected = false,
    tone = 'teal',
    dot,
    count,
    check = true,
    title,
    pressed,
    onclick,
  }: {
    label: string;
    selected?: boolean;
    tone?: 'teal' | 'warning';
    dot?: string;
    count?: number;
    check?: boolean;
    title?: string;
    pressed?: boolean;
    onclick?: (e: MouseEvent) => void;
  } = $props();

  const selectedCls = $derived(
    tone === 'warning'
      ? 'border-warning/60 bg-warning-bg text-warning font-semibold'
      : 'bg-teal/10 text-teal font-semibold border-transparent'
  );
</script>

<button
  type="button"
  {onclick}
  {title}
  aria-pressed={pressed ?? selected}
  class="font-proto text-smaller inline-flex h-6 w-full cursor-pointer items-center gap-1.5 border px-2 whitespace-nowrap uppercase transition-colors select-none {selected
    ? selectedCls
    : 'border-line text-text-muted hover:border-text-base hover:text-text-base hover:bg-bg-card/40'}"
>
  {#if dot}
    <span class="size-1.5 shrink-0" style="background:{dot}"></span>
  {/if}
  <span class="truncate">{label}</span>
  {#if count != null}
    <span class="ml-auto tabular-nums opacity-60">({count})</span>
  {/if}
  {#if selected && check}
    <span class="text-smaller shrink-0 font-bold">✓</span>
  {/if}
</button>

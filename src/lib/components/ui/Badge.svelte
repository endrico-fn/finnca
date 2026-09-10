<script lang="ts">
  let {
    tone = 'ok',
    class: extraClass = '',
    children,
  }: {
    tone?: 'ok' | 'warn' | 'err' | 'info' | 'neutral';
    class?: string;
    children: import('svelte').Snippet;
  } = $props();

  const cls = $derived(
    tone === 'ok'
      ? 'badge-ok'
      : tone === 'warn'
        ? 'badge-warn'
        : tone === 'err'
          ? 'badge-err'
          : tone === 'info'
            ? 'badge-info'
            : 'badge-neutral'
  );
</script>

<span
  class="{cls} font-proto inline-flex items-center gap-1 px-2 py-0.5 text-[10px] tracking-wider uppercase {extraClass}"
>
  {@render children()}
</span>

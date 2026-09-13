<script lang="ts">
  let {
    tone = 'ok',
    class: extraClass = '',
    children,
  }: {
    tone?: 'ok' | 'warn' | 'err' | 'neutral';
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
          : 'border-line bg-bg-app text-text-muted'
  );
</script>

<span
  class="{cls} font-proto text-smaller inline-flex items-center gap-1 px-2 py-0.5 tracking-wider uppercase {extraClass}"
>
  {@render children()}
</span>

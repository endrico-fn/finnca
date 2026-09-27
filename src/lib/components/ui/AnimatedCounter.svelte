<script lang="ts">
  import { Tween, prefersReducedMotion } from 'svelte/motion';
  import { expoOut } from 'svelte/easing';
  import { formatMinorToDisplay } from '$lib/core/format/currency';

  let {
    value,
    currency = 'IDR',
    duration = 650,
    class: extraClass = '',
    formatFn,
  }: {
    value: number;
    currency?: string;
    duration?: number;
    class?: string;
    formatFn?: (val: number) => string;
  } = $props();

  const tween = new Tween(0, {
    duration: () => (prefersReducedMotion.current ? 0 : duration),
    easing: expoOut,
  });

  $effect(() => {
    void tween.set(value);
  });

  const display = $derived(
    formatFn
      ? formatFn(Math.round(tween.current))
      : formatMinorToDisplay(Math.round(tween.current), currency)
  );
</script>

<span class="font-proto tabular-nums {extraClass}">{display}</span>

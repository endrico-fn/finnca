<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import Button from './Button.svelte';

  let {
    message,
    onRetry,
  }: {
    message: string;
    onRetry?: () => void;
  } = $props();
</script>

<div class="flex flex-1 flex-col items-center justify-center gap-3 py-16">
  <p class="badge-err font-proto text-small max-w-sm px-3 py-2 text-center">{message}</p>
  {#if onRetry}
    <Button variant="ghost" onclick={onRetry}>{i18n.t.commonRetry}</Button>
  {/if}
</div>

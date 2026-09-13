<script lang="ts">
  import { i18n } from '$lib/i18n.svelte';
  import Spinner from './Spinner.svelte';
  import Button from './Button.svelte';

  let {
    label = '',
    error = '',
    onRetry,
    variant = 'fullscreen',
  }: {
    label?: string;
    error?: string;
    onRetry?: () => void;
    variant?: 'fullscreen' | 'inline';
  } = $props();
</script>

<div
  class={variant === 'fullscreen'
    ? 'bg-bg-app grid h-screen place-items-center'
    : 'flex flex-1 items-center justify-center py-16'}
>
  <div class="flex flex-col items-center gap-4">
    <Spinner />
    {#if label}
      <p class="font-proto text-text-base text-small tracking-[0.16em] uppercase">
        {label}
      </p>
    {/if}
    {#if error}
      <p class="badge-err text-small max-w-sm px-3 py-2 text-center">
        {error}
      </p>
      {#if onRetry}
        <Button variant="ghost" onclick={onRetry}>{i18n.t.commonRetry}</Button>
      {/if}
    {/if}
  </div>
</div>

<script lang="ts">
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
      <p class="font-proto text-text-icon text-[11px] tracking-[0.16em] uppercase">
        {label}
      </p>
    {/if}
    {#if error}
      <p class="badge-err max-w-sm px-3 py-2 text-center text-[11px]">
        {error}
      </p>
      {#if onRetry}
        <Button variant="ghost" onclick={onRetry}>RETRY</Button>
      {/if}
    {/if}
  </div>
</div>

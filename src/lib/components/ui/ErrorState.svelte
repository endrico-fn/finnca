<script lang="ts">
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
  <p class="badge-err max-w-sm px-3 py-2 text-center text-[11px]">{message}</p>
  {#if onRetry}
    <Button variant="ghost" onclick={onRetry}>RETRY</Button>
  {/if}
</div>

<script lang="ts">
  import type { Snippet } from 'svelte';
  import CloseButton from './CloseButton.svelte';

  let {
    open = $bindable(false),
    title,
    onClose,
    children,
    maxWidth = 'max-w-md',
  }: {
    open: boolean;
    title: string;
    onClose?: () => void;
    children: Snippet;
    maxWidth?: string;
  } = $props();

  function close() {
    open = false;
    onClose?.();
  }
  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
</script>

<svelte:window onkeydown={handleKey} />

{#if open}
  <div
    class="bg-overlay fixed inset-0 z-50 flex items-center justify-center p-4 font-mono"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) close();
    }}
    onkeydown={handleKey}
  >
    <div
      class="sharp-card w-full {maxWidth} anim-modal flex max-h-[90vh] flex-col overflow-y-auto p-4"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      tabindex="-1"
    >
      <div class="flex h-8 shrink-0 items-center justify-between pb-2.5">
        <div class="flex items-center gap-2.5">
          <span class="bg-income size-2 shrink-0"></span>
          <h2 class="label-title text-[13px] leading-none">{title}</h2>
        </div>
        <CloseButton onclick={close} />
      </div>
      {@render children()}
    </div>
  </div>
{/if}

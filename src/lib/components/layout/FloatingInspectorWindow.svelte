<script lang="ts">
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';

  let {
    open = false,
    title = '',
    onClose,
    children,
  }: {
    open?: boolean;
    title?: string;
    onClose?: () => void;
    children: Snippet;
  } = $props();

  let windowWidth = $state<number>(typeof window !== 'undefined' ? window.innerWidth : 1280);

  function handleResize() {
    if (typeof window !== 'undefined') {
      windowWidth = window.innerWidth;
    }
  }

  onMount(() => {
    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  });

  const isDocked = $derived(
    modalState.inspectorLayout === 'docked' ||
      (modalState.inspectorLayout === 'adaptive' && windowWidth >= 1280)
  );
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === 'Escape' && open && onClose) {
      onClose();
    }
  }}
/>

{#if open}
  {#if isDocked}
    <div
      class="border-line bg-bg-app animate-in slide-in-from-right fixed inset-y-0 right-0 z-50 flex w-full max-w-full flex-col overflow-hidden rounded-none border-l-2 transition-all duration-150 sm:w-(--layout-inspector-docked)"
      role="dialog"
      tabindex="-1"
      aria-modal="false"
      aria-label={title || i18n.t.entryInspectorTitle}
    >
      <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
        {@render children()}
      </div>
    </div>
  {:else}
    <div
      class="bg-overlay backdrop-blur-subtle fixed inset-0 z-50 flex items-center justify-center p-3 transition-opacity sm:p-4"
      role="presentation"
      onclick={(e) => {
        if (e.target === e.currentTarget && onClose) {
          onClose();
        }
      }}
    >
      <div
        class="border-line bg-bg-app flex h-(--layout-inspector-height) max-h-[calc(100dvh-2.5rem)] w-full max-w-(--layout-inspector-max) flex-col overflow-hidden rounded-none border-2 sm:w-(--layout-inspector-float)"
        role="dialog"
        tabindex="-1"
        aria-modal="true"
        aria-label={title || i18n.t.entryInspectorTitle}
      >
        <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
          {@render children()}
        </div>
      </div>
    </div>
  {/if}
{/if}

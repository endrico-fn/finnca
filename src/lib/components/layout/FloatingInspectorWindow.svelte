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
      class="fixed inset-y-0 right-0 z-50 flex flex-col border-l-2 border-line bg-bg-app rounded-none w-[560px] max-w-[96vw] shadow-2xl overflow-hidden transition-all duration-150 animate-in slide-in-from-right"
      role="dialog"
      tabindex="-1"
      aria-modal="false"
      aria-label={title || i18n.t.entryInspectorTitle}
    >
      <div class="min-h-0 flex-1 flex flex-col overflow-hidden">
        {@render children()}
      </div>
    </div>
  {:else}
    <div
      class="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-4 bg-black/60 backdrop-blur-[2px] transition-opacity"
      role="presentation"
      onclick={(e) => {
        if (e.target === e.currentTarget && onClose) {
          onClose();
        }
      }}
    >
      <div
        class="flex flex-col border-2 border-line bg-bg-app rounded-none w-[96vw] sm:w-[680px] max-w-[740px] h-[88vh] max-h-[calc(100dvh-2.5rem)] shadow-2xl overflow-hidden"
        role="dialog"
        tabindex="-1"
        aria-modal="true"
        aria-label={title || i18n.t.entryInspectorTitle}
      >
        <div class="min-h-0 flex-1 flex flex-col overflow-hidden">
          {@render children()}
        </div>
      </div>
    </div>
  {/if}
{/if}

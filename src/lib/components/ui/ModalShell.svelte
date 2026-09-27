<script module lang="ts">
  export const OVERLAY_SIZES = {
    xs: 'max-w-xs',
    md: 'max-w-md',
    lg: 'max-w-lg',
    xl: 'max-w-xl',
    wide: 'max-w-4xl',
    full: 'max-w-5xl',
  } as const;

  export type OverlaySize = keyof typeof OVERLAY_SIZES;
</script>

<script lang="ts">
  import type { Snippet } from 'svelte';
  import CloseButton from './CloseButton.svelte';

  export type ModalTone = 'neutral' | 'ok' | 'err' | 'warn' | 'teal';

  const TONE_CLASSES: Record<ModalTone, string> = {
    neutral: 'bg-text-dim',
    ok: 'bg-income',
    err: 'bg-expense',
    warn: 'bg-warning',
    teal: 'bg-teal',
  };

  const FOCUSABLE =
    'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])';

  let {
    open = $bindable(false),
    title,
    tone = 'teal',
    onClose,
    children,
    size = 'md',
  }: {
    open: boolean;
    title: string;
    tone?: ModalTone;
    onClose?: () => void;
    children: Snippet;
    size?: OverlaySize;
  } = $props();

  let dialogEl: HTMLDivElement | null = $state(null);
  let prevFocus: Element | null = null;

  function close() {
    open = false;
    onClose?.();
  }
  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }

  function handleDialogKey(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Escape') {
      close();
      return;
    }
    if (e.key !== 'Tab' || !dialogEl) return;
    const items = [...dialogEl.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
      (el) => el.offsetParent !== null
    );
    if (items.length === 0) {
      e.preventDefault();
      return;
    }
    const first = items[0];
    const last = items[items.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }

  $effect(() => {
    if (open && dialogEl) {
      prevFocus = document.activeElement;
      dialogEl.focus({ preventScroll: true });
      return () => {
        (prevFocus as HTMLElement | null)?.focus?.({ preventScroll: true });
      };
    }
  });
</script>

<svelte:window onkeydown={handleKey} />

{#if open}
  <div
    class="bg-overlay fixed inset-0 z-50 flex items-center justify-center p-4"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) close();
    }}
    onkeydown={handleKey}
  >
    <div
      bind:this={dialogEl}
      class="sharp-card w-full {OVERLAY_SIZES[
        size
      ]} anim-modal flex max-h-[calc(100dvh-2rem)] flex-col overflow-y-auto p-4"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      onclick={(e) => e.stopPropagation()}
      onkeydown={handleDialogKey}
      tabindex="-1"
    >
      <div class="flex h-8 shrink-0 items-center justify-between pb-2.5">
        <div class="flex items-center gap-2.5">
          <span class="{TONE_CLASSES[tone]} size-2 shrink-0"></span>
          <h2 class="label-title text-medium leading-none">{title}</h2>
        </div>
        <CloseButton onclick={close} />
      </div>
      {@render children()}
    </div>
  </div>
{/if}

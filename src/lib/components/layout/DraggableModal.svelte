<script lang="ts">
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';

  const FOCUSABLE =
    'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])';

  let {
    open = $bindable(false),
    title = '',
    onClose,
    children,
    widthClass = 'w-[880px] max-w-[calc(100vw-2rem)]',
    heightClass = 'h-[min(85vh,800px)] max-h-[85vh]',
    zIndex = 'z-50',
  }: {
    open?: boolean;
    title?: string;
    onClose?: () => void;
    children: Snippet;
    widthClass?: string;
    heightClass?: string;
    zIndex?: string;
  } = $props();

  let windowEl: HTMLDivElement | null = $state(null);
  let posX = $state<number | null>(null);
  let posY = $state<number | null>(null);
  let isDragging = $state(false);

  let startX = 0;
  let startY = 0;
  let startLeft = 0;
  let startTop = 0;
  let cachedWidth = 0;
  let cachedHeight = 0;
  let rafId = 0;
  let pendingClientX = 0;
  let pendingClientY = 0;
  let hasDragged = false;
  let activeDragHandle: HTMLElement | null = null;
  let backdropPointerDown = false;
  let prevFocus: Element | null = null;

  function close() {
    open = false;
    onClose?.();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && open) {
      e.stopPropagation();
      close();
    }
  }

  function handleDialogKey(e: KeyboardEvent) {
    if (e.key !== 'Tab' || !windowEl) return;
    const items = [...windowEl.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
      (el) => el.offsetWidth > 0 || el.offsetHeight > 0 || el.getClientRects().length > 0
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

  function handleWindowResize() {
    if (!windowEl || posX === null || posY === null) return;
    const rect = windowEl.getBoundingClientRect();
    const maxLeft = Math.max(0, window.innerWidth - rect.width);
    const maxTop = Math.max(0, window.innerHeight - rect.height);
    posX = Math.round(Math.max(0, Math.min(maxLeft, posX)));
    posY = Math.round(Math.max(0, Math.min(maxTop, posY)));
  }

  function handlePointerDown(e: PointerEvent) {
    if (e.button !== 0) return;

    const target = e.target as HTMLElement | null;
    if (!target) return;

    // Do not initiate drag when clicking on interactive elements
    if (target.closest('button, input, select, textarea, a, [data-no-drag]')) {
      return;
    }

    // Must be clicking on a header or explicit drag handle inside the window
    const dragHandle = target.closest('[data-drag-handle]') ?? target.closest('header');
    if (!dragHandle || !windowEl?.contains(dragHandle)) {
      return;
    }

    // If it's a generic header without explicit data-drag-handle, only allow top-level window header
    if (!dragHandle.hasAttribute('data-drag-handle')) {
      const allHeaders = windowEl.querySelectorAll('header');
      if (allHeaders.length > 0 && dragHandle !== allHeaders[0]) {
        return;
      }
    }

    if (!windowEl) return;

    e.preventDefault();

    const rect = windowEl.getBoundingClientRect();
    cachedWidth = rect.width;
    cachedHeight = rect.height;
    startLeft = posX !== null ? posX : rect.left;
    startTop = posY !== null ? posY : rect.top;
    startX = e.clientX;
    startY = e.clientY;
    hasDragged = false;
    isDragging = true;

    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);
    window.addEventListener('pointercancel', handlePointerUp);

    try {
      const handleEl = dragHandle as HTMLElement;
      handleEl.setPointerCapture(e.pointerId);
      activeDragHandle = handleEl;
    } catch {
      activeDragHandle = null;
    }
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isDragging || !windowEl) return;

    pendingClientX = e.clientX;
    pendingClientY = e.clientY;

    if (!rafId) {
      rafId = requestAnimationFrame(() => {
        rafId = 0;
        if (!isDragging || !windowEl) return;

        const deltaX = pendingClientX - startX;
        const deltaY = pendingClientY - startY;

        if (!hasDragged) {
          if (Math.hypot(deltaX, deltaY) < 3) return;
          hasDragged = true;
        }

        const maxLeft = Math.max(0, window.innerWidth - cachedWidth);
        const maxTop = Math.max(0, window.innerHeight - cachedHeight);

        posX = Math.round(Math.max(0, Math.min(maxLeft, startLeft + deltaX)));
        posY = Math.round(Math.max(0, Math.min(maxTop, startTop + deltaY)));
      });
    }
  }

  function handlePointerUp(e?: PointerEvent) {
    if (!isDragging) return;
    isDragging = false;

    if (rafId) {
      cancelAnimationFrame(rafId);
      rafId = 0;
    }

    window.removeEventListener('pointermove', handlePointerMove);
    window.removeEventListener('pointerup', handlePointerUp);
    window.removeEventListener('pointercancel', handlePointerUp);

    if (activeDragHandle) {
      try {
        if (e && typeof e.pointerId === 'number') {
          activeDragHandle.releasePointerCapture(e.pointerId);
        }
      } catch {
        // Pointer was already detached
      }
      activeDragHandle = null;
    }

    // Reset hasDragged after gesture completes so subsequent backdrop clicks close normally
    setTimeout(() => {
      hasDragged = false;
    }, 50);
  }

  function handleBackdropPointerDown(e: PointerEvent) {
    if (e.target === e.currentTarget && e.button === 0) {
      backdropPointerDown = true;
    } else {
      backdropPointerDown = false;
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && backdropPointerDown && !isDragging && !hasDragged) {
      close();
    }
    backdropPointerDown = false;
    hasDragged = false;
  }

  $effect(() => {
    if (open && windowEl) {
      if (typeof document !== 'undefined') {
        prevFocus = document.activeElement;
      }
      windowEl.focus({ preventScroll: true });
      return () => {
        if (rafId) {
          cancelAnimationFrame(rafId);
          rafId = 0;
        }
        if (isDragging) {
          handlePointerUp();
        }
        isDragging = false;
        hasDragged = false;
        posX = null;
        posY = null;
        if (prevFocus && typeof (prevFocus as HTMLElement).focus === 'function') {
          (prevFocus as HTMLElement).focus({ preventScroll: true });
        }
      };
    }
  });

  onMount(() => {
    return () => {
      if (rafId) {
        cancelAnimationFrame(rafId);
        rafId = 0;
      }
      window.removeEventListener('pointermove', handlePointerMove);
      window.removeEventListener('pointerup', handlePointerUp);
      window.removeEventListener('pointercancel', handlePointerUp);
    };
  });

  const positionStyle = $derived(
    posX !== null && posY !== null
      ? `position: fixed; left: ${posX}px; top: ${posY}px; margin: 0;`
      : `position: fixed; left: 50%; top: 50%; transform: translate(-50%, -50%); margin: 0;`
  );
</script>

<svelte:window onkeydown={handleKeydown} onresize={handleWindowResize} />

{#if open}
  <div
    class="bg-overlay fixed inset-0 {zIndex} flex overflow-hidden"
    role="presentation"
    onpointerdown={handleBackdropPointerDown}
    onclick={handleBackdropClick}
  >
    <div
      bind:this={windowEl}
      class="border-line bg-bg-app flex flex-col overflow-hidden rounded-none border-2 shadow-2xl transition-shadow outline-none {widthClass} {heightClass} {isDragging
        ? 'pointer-events-auto select-none'
        : ''}"
      style={positionStyle}
      role="dialog"
      aria-modal="true"
      aria-label={title}
      tabindex="-1"
      onpointerdown={handlePointerDown}
      onkeydown={handleDialogKey}
      onclick={(e) => e.stopPropagation()}
    >
      <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
        {@render children()}
      </div>
    </div>
  </div>
{/if}

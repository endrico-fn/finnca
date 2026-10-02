<script lang="ts">
  import { type Snippet, untrack } from 'svelte';
  import { prefersReducedMotion } from 'svelte/motion';

  let {
    content,
    children,
    placement = 'top',
    delay = 150,
    class: extraClass = '',
  }: {
    content?: string;
    children: Snippet;
    placement?: 'top' | 'bottom' | 'left' | 'right';
    delay?: number;
    class?: string;
  } = $props();

  let triggerEl = $state<HTMLDivElement | null>(null);
  let tooltipEl = $state<HTMLDivElement | null>(null);
  let visible = $state(false);
  let tooltipX = $state(-9999);
  let tooltipY = $state(-9999);
  let timeoutId = $state<ReturnType<typeof setTimeout> | null>(null);

  function computePosition() {
    if (!triggerEl || !tooltipEl) return;
    const tr = triggerEl.getBoundingClientRect();
    const tt = tooltipEl.getBoundingClientRect();
    const gap = 6;

    let x: number;
    let y: number;
    let effective = placement;

    if (placement === 'bottom' && tr.bottom + gap + tt.height > window.innerHeight - 6) {
      effective = 'top';
    } else if (placement === 'top' && tr.top - gap - tt.height < 6) {
      effective = 'bottom';
    }

    if (effective === 'top') {
      x = tr.left + tr.width / 2 - tt.width / 2;
      y = tr.top - tt.height - gap;
    } else if (effective === 'bottom') {
      x = tr.left + tr.width / 2 - tt.width / 2;
      y = tr.bottom + gap;
    } else if (effective === 'left') {
      x = tr.left - tt.width - gap;
      y = tr.top + tr.height / 2 - tt.height / 2;
    } else {
      x = tr.right + gap;
      y = tr.top + tr.height / 2 - tt.height / 2;
    }

    const vw = window.innerWidth;
    const vh = window.innerHeight;
    x = Math.max(6, Math.min(vw - tt.width - 6, x));
    y = Math.max(6, Math.min(vh - tt.height - 6, y));

    tooltipX = x;
    tooltipY = y;
  }

  function show() {
    if (!content) return;
    if (timeoutId) clearTimeout(timeoutId);
    const d = prefersReducedMotion.current ? 0 : delay;
    timeoutId = setTimeout(() => {
      visible = true;
      requestAnimationFrame(computePosition);
    }, d);
  }

  function hide() {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = null;
    visible = false;
    tooltipX = -9999;
    tooltipY = -9999;
  }

  $effect(() => {
    if (visible && tooltipEl) {
      untrack(computePosition);
      const onReposition = () => computePosition();
      window.addEventListener('scroll', onReposition, true);
      window.addEventListener('resize', onReposition);
      return () => {
        window.removeEventListener('scroll', onReposition, true);
        window.removeEventListener('resize', onReposition);
      };
    }
  });
</script>

<div
  bind:this={triggerEl}
  role="presentation"
  class="relative inline-flex {extraClass}"
  onmouseenter={show}
  onmouseleave={hide}
  onfocusin={show}
  onfocusout={hide}
>
  {@render children()}
</div>

{#if visible && content}
  <div
    bind:this={tooltipEl}
    class="hover-overlay font-proto pointer-events-none fixed"
    style="left: {tooltipX}px; top: {tooltipY}px; z-index: var(--z-palette);"
    role="tooltip"
    aria-hidden="true"
  >
    <div class="text-smaller text-text-strong px-2.5 py-1.5 whitespace-nowrap">
      {content}
    </div>
  </div>
{/if}

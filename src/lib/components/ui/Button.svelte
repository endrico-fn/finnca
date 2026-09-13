<script lang="ts">
  import type { Snippet } from 'svelte';
  import { resolve } from '$app/paths';

  let {
    variant = 'primary',
    size = 'md',
    disabled = false,
    type = 'button',
    onclick,
    href,
    ariaLabel,
    pressed,
    title,
    class: extraClass = '',
    children,
  }: {
    variant?: 'primary' | 'ghost' | 'danger' | 'pager' | 'tactical';
    size?: 'sm' | 'md' | 'icon';
    disabled?: boolean;
    type?: 'button' | 'submit';
    onclick?: (e: MouseEvent) => void;
    href?: string;
    ariaLabel?: string;
    pressed?: boolean;
    title?: string;
    class?: string;
    children: Snippet;
  } = $props();

  const cls = $derived.by(() => {
    if (variant === 'pager') {
      const icon = size === 'icon' ? 'btn-pager-icon' : '';
      return `sharp-btn btn-pager font-proto ${icon} ${extraClass}`.trim().replace(/\s+/g, ' ');
    }
    const sizing =
      size === 'sm'
        ? 'h-7 px-2.5 text-smaller'
        : size === 'icon'
          ? 'w-7 h-7 p-0 shrink-0'
          : 'h-8 px-3.5 text-small';
    const dim = disabled ? 'opacity-40 cursor-not-allowed' : '';
    return `sharp-btn btn-${variant} font-proto ${sizing} ${dim} ${extraClass}`
      .trim()
      .replace(/\s+/g, ' ');
  });
</script>

{#if href}
  <a
    href={resolve(href as '/app')}
    class={cls}
    aria-disabled={disabled}
    aria-label={ariaLabel}
    {title}
  >
    {@render children()}
  </a>
{:else}
  <button
    {type}
    {disabled}
    {onclick}
    class={cls}
    aria-label={ariaLabel}
    aria-pressed={pressed}
    {title}
  >
    {@render children()}
  </button>
{/if}

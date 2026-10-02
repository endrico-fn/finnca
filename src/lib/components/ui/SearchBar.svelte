<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import Icon from './Icon.svelte';

  let {
    value = $bindable(''),
    placeholder = '',
    scope,
    class: extraClass = 'max-w-70',
    oninput,
    onclear,
  }: {
    value: string;
    placeholder?: string;
    scope?: {
      value: string;
      options: Array<{ value: string; label: string; id?: string }>;
      onchange?: (v: string) => void;
    };
    class?: string;
    oninput?: (e: Event) => void;
    onclear?: () => void;
  } = $props();

  let scopeOpen = $state(false);
  let scopeEl: HTMLElement | null = $state(null);

  const selectedScope = $derived(
    scope?.options.find((o) => o.value === scope.value) ?? scope?.options[0]
  );

  function handleClickOutside(e: MouseEvent) {
    if (scopeEl && !scopeEl.contains(e.target as Node)) {
      scopeOpen = false;
    }
  }

  onMount(() => {
    window.addEventListener('click', handleClickOutside);
    return () => window.removeEventListener('click', handleClickOutside);
  });
</script>

<div
  class="bg-bg-app border-line focus-within:border-teal flex h-8 w-full min-w-40 items-center border transition-colors {extraClass}"
>
  {#if scope}
    <div
      class="border-line bg-bg-btn/60 relative inline-flex h-full shrink-0 items-center border-r"
      bind:this={scopeEl}
    >
      <button
        type="button"
        onclick={() => (scopeOpen = !scopeOpen)}
        class="text-text-base hover:text-text-strong text-smaller font-proto inline-flex h-full cursor-pointer items-center gap-1.5 bg-transparent pr-2 pl-2 uppercase select-none"
      >
        <span>{selectedScope?.label ?? scope.value}</span>
        <span
          class="text-text-base inline-flex transition-transform duration-150 {scopeOpen
            ? 'text-teal rotate-180'
            : ''}"
        >
          <Icon name="chev-down" size={9} />
        </span>
      </button>

      {#if scopeOpen}
        <div
          class="border-line bg-bg-card text-smaller font-proto absolute top-full left-0 z-50 mt-1 min-w-32 border py-1 select-none"
        >
          {#each scope.options as opt (opt.id || opt)}
            <button
              type="button"
              onclick={() => {
                scope?.onchange?.(opt.value);
                scopeOpen = false;
              }}
              class="text-smaller font-proto flex w-full cursor-pointer items-center justify-between px-3 py-1.5 text-left uppercase transition-colors {opt.value ===
              scope.value
                ? 'bg-teal/10 text-teal font-semibold'
                : 'text-text-base hover:bg-line/40 hover:text-text-strong'}"
            >
              <span>{opt.label}</span>
              {#if opt.value === scope.value}
                <span class="text-teal text-smaller font-bold">✓</span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {:else}
    <span class="text-text-base inline-flex pl-2">
      <Icon name="search" size={12} />
    </span>
  {/if}
  <input
    type="text"
    bind:value
    {oninput}
    placeholder={placeholder || i18n.t.searchAllPlaceholder}
    aria-label={placeholder || i18n.t.searchAriaFallback}
    autocomplete="off"
    spellcheck="false"
    class="text-text-strong placeholder:text-text-muted text-small font-aux h-full w-full bg-transparent px-2 outline-none"
  />
  {#if value}
    <button
      type="button"
      onclick={() => {
        value = '';
        onclear?.();
      }}
      class="text-text-base hover:text-text-strong inline-flex h-full items-center px-2"
      aria-label={i18n.t.searchClearLabel}
    >
      <Icon name="close" size={12} />
    </button>
  {:else}
    <div class="flex h-full shrink-0 items-center justify-center px-2">
      <kbd class="border-line/50 text-text-muted text-smaller font-proto border px-1 py-0.5"
        >{i18n.t.searchKbdHint}</kbd
      >
    </div>
  {/if}
</div>

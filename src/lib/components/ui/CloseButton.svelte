<script lang="ts">
  import { i18n } from '$lib/i18n.svelte';
  import Button from './Button.svelte';
  import Icon from './Icon.svelte';

  let {
    onclick,
    label,
    title,
    iconSize = 12,
    class: extraClass = '',
  }: {
    onclick: () => void;
    label?: string;
    title?: string;
    iconSize?: number;
    class?: string;
  } = $props();

  const ariaLabel = $derived(label ?? i18n.t.closeBtn);
  const titleText = $derived(title ?? ariaLabel);
</script>

<Button
  variant="pager"
  size="icon"
  {onclick}
  ariaLabel={ariaLabel}
  title={titleText}
  class={extraClass}><Icon name="close" size={iconSize} /></Button
>

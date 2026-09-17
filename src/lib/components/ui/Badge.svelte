<script lang="ts">
  import { BADGE_TONES, BADGE_SIZES, type BadgeTone, type BadgeSize } from './badgeTone';

  let {
    tone = 'neutral',
    size = 'm',
    class: extraClass = '',
    children,
  }: {
    tone?: BadgeTone;
    size?: BadgeSize;
    class?: string;
    children: import('svelte').Snippet;
  } = $props();
</script>

<span
  class="{BADGE_TONES[
    tone
  ]} font-proto inline-flex items-center gap-1 border uppercase {BADGE_SIZES[size]} {extraClass}"
>
  {@render children()}
</span>

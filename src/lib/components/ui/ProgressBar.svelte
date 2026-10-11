<script lang="ts">
  export type ProgressTone = 'teal' | 'income' | 'expense' | 'warning';
  export type ProgressTrack = 'line' | 'card' | 'app';
  export type ProgressSize = 'xs' | 's' | 'm';

  const TONE_CLASSES: Record<ProgressTone, string> = {
    teal: 'bg-teal',
    income: 'bg-income',
    expense: 'bg-expense',
    warning: 'bg-warning',
  };

  const TRACK_CLASSES: Record<ProgressTrack, string> = {
    line: 'bg-line/40',
    card: 'bg-bg-card border-line border',
    app: 'bg-bg-app border-line border',
  };

  const SIZE_CLASSES: Record<ProgressSize, string> = {
    xs: 'h-1',
    s: 'h-1.5',
    m: 'h-2',
  };

  let {
    value,
    max = 100,
    tone = 'teal',
    track = 'card',
    size = 's',
    class: className = '',
  }: {
    value: number;
    max?: number;
    tone?: ProgressTone;
    track?: ProgressTrack;
    size?: ProgressSize;
    class?: string;
  } = $props();

  const pct = $derived(Math.max(0, Math.min(100, max > 0 ? (value / max) * 100 : 0)));
</script>

<div class="{TRACK_CLASSES[track]} {SIZE_CLASSES[size]} w-full overflow-hidden {className}">
  <div class="{TONE_CLASSES[tone]} h-full transition-all duration-300" style="width: {pct}%"></div>
</div>

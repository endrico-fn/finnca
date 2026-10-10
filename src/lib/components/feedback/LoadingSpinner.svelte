<script lang="ts">
  type SpinnerSize = 'sm' | 'md' | 'lg' | 'xl';
  type SpinnerTone = 'teal' | 'neutral' | 'ok' | 'err' | 'warn' | 'current';

  let {
    size = 'lg',
    tone = 'teal',
    class: customClass = '',
    label = 'Loading...',
  }: {
    size?: SpinnerSize;
    tone?: SpinnerTone;
    class?: string;
    label?: string;
  } = $props();

  const sizeMap: Record<
    SpinnerSize,
    {
      container: string;
      outer: string;
      inner: string;
      border: string;
    }
  > = {
    sm: {
      container: 'w-4 h-4',
      outer: 'w-4 h-4',
      inner: 'w-2 h-2',
      border: 'border',
    },
    md: {
      container: 'w-6 h-6',
      outer: 'w-6 h-6',
      inner: 'w-3 h-3',
      border: 'border-2',
    },
    lg: {
      container: 'w-8 h-8',
      outer: 'w-8 h-8',
      inner: 'w-4 h-4',
      border: 'border-2',
    },
    xl: {
      container: 'w-12 h-12',
      outer: 'w-12 h-12',
      inner: 'w-6 h-6',
      border: 'border-2',
    },
  };

  const toneMap: Record<SpinnerTone, { outer: string; inner: string }> = {
    teal: {
      outer: 'border-teal',
      inner: 'bg-teal',
    },
    neutral: {
      outer: 'border-text-secondary',
      inner: 'bg-text-secondary',
    },
    ok: {
      outer: 'border-income',
      inner: 'bg-income',
    },
    err: {
      outer: 'border-expense',
      inner: 'bg-expense',
    },
    warn: {
      outer: 'border-warn',
      inner: 'bg-warn',
    },
    current: {
      outer: 'border-current',
      inner: 'bg-current',
    },
  };
</script>

<div
  role="status"
  aria-label={label}
  class="relative inline-flex shrink-0 items-center justify-center select-none {sizeMap[size].container} {customClass}"
>
  <!-- Kotak fill di dalam (berputar berlawanan arah jarum jam) -->
  <div
    class="spin-inner absolute z-0 rounded-none {sizeMap[size].inner} {toneMap[tone].inner}"
    aria-hidden="true"
  ></div>

  <!-- Kotak border di atas/luar (berputar searah jarum jam) -->
  <div
    class="spin-outer pointer-events-none absolute z-10 rounded-none bg-transparent {sizeMap[size].outer} {sizeMap[size].border} {toneMap[tone].outer}"
    aria-hidden="true"
  ></div>

  <span class="sr-only">{label}</span>
</div>

<style>
  .spin-outer {
    will-change: transform;
    animation: spin-cw 2s linear infinite;
  }

  .spin-inner {
    will-change: transform;
    animation: spin-ccw 2s linear infinite;
  }

  @keyframes spin-cw {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes spin-ccw {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(-360deg);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .spin-outer,
    .spin-inner {
      animation-duration: 8s;
    }
  }
</style>

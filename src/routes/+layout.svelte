<script lang="ts">
  import '../app.css';
  import { APP_NAME } from '$lib/types';
  import { onMount } from 'svelte';

  let { children } = $props();

  onMount(() => {
    const preventWheelZoom = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        e.stopPropagation();
      }
    };

    const preventKeyZoom = (e: KeyboardEvent) => {
      if (
        (e.ctrlKey || e.metaKey) &&
        (e.key === '+' ||
          e.key === '-' ||
          e.key === '=' ||
          e.key === '_' ||
          e.key === '0' ||
          e.code === 'NumpadAdd' ||
          e.code === 'NumpadSubtract')
      ) {
        e.preventDefault();
        e.stopPropagation();
      }
    };

    // 3. Block multi-touch pinch gestures
    const preventTouch = (e: TouchEvent) => {
      if (e.touches && e.touches.length > 1) {
        e.preventDefault();
      }
    };

    // 4. Block WebKit gesture events (pinch-to-zoom on touchpad)
    const preventGesture = (e: Event) => {
      e.preventDefault();
      e.stopPropagation();
    };

    document.addEventListener('wheel', preventWheelZoom, {
      capture: true,
      passive: false,
    });
    window.addEventListener('wheel', preventWheelZoom, {
      capture: true,
      passive: false,
    });
    document.addEventListener('keydown', preventKeyZoom, {
      capture: true,
      passive: false,
    });
    window.addEventListener('keydown', preventKeyZoom, {
      capture: true,
      passive: false,
    });
    document.addEventListener('touchstart', preventTouch, {
      capture: true,
      passive: false,
    });
    document.addEventListener('touchmove', preventTouch, {
      capture: true,
      passive: false,
    });
    document.addEventListener('gesturestart', preventGesture, {
      capture: true,
      passive: false,
    });
    document.addEventListener('gesturechange', preventGesture, {
      capture: true,
      passive: false,
    });
    document.addEventListener('gestureend', preventGesture, {
      capture: true,
      passive: false,
    });

    return () => {
      document.removeEventListener('wheel', preventWheelZoom);
      window.removeEventListener('wheel', preventWheelZoom);
      document.removeEventListener('keydown', preventKeyZoom);
      window.removeEventListener('keydown', preventKeyZoom);
      document.removeEventListener('touchstart', preventTouch);
      document.removeEventListener('touchmove', preventTouch);
      document.removeEventListener('gesturestart', preventGesture);
      document.removeEventListener('gesturechange', preventGesture);
      document.removeEventListener('gestureend', preventGesture);
    };
  });
</script>

<svelte:head>
  <title>{APP_NAME}</title>
</svelte:head>

{@render children()}

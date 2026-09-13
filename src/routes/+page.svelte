<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { APP_NAME } from '$lib/types';
  import { store } from '$lib/stores/app-store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { Splash } from '$lib/components/ui';

  let bootError = $state('');

  onMount(async () => {
    try {
      await store.refresh();
    } catch (e) {
      bootError = String(e).replace('Error: ', '');
      return;
    }
    if (!store.appState) return;
    if (!store.appState.configured) {
      goto(resolve('/setup'), { replaceState: true });
    } else if (!store.appState.unlocked) {
      goto(resolve('/login'), { replaceState: true });
    } else {
      goto(resolve('/app'), { replaceState: true });
    }
  });
</script>

<svelte:head>
  <title>{APP_NAME}</title>
</svelte:head>

<Splash label={i18n.t.loadingFinnca} error={bootError} onRetry={() => location.reload()} />

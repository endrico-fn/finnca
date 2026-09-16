<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { APP_NAME } from '$lib/core/types';
  import { session } from '$lib/core/state/session.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Splash } from '$lib/components/ui';

  let bootError = $state('');

  onMount(async () => {
    try {
      await session.refresh();
    } catch (e) {
      bootError = String(e).replace('Error: ', '');
      return;
    }
    if (!session.isConfigured) {
      goto(resolve('/setup'), { replaceState: true });
    } else if (!session.isUnlocked) {
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

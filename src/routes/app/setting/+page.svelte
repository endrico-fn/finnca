<script lang="ts">
  import { store } from '$lib/stores/app-store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { APP_NAME } from '$lib/types';
  import { Splash, ErrorState, PageLayout, Tabs } from '$lib/components/ui';

  import GeneralSettings from './_views/GeneralSettings.svelte';
  import FinanceSettings from './_views/FinanceSettings.svelte';
  import SecuritySettings from './_views/SecuritySettings.svelte';
  import DataSettings from './_views/DataSettings.svelte';

  type Tab = 'general' | 'finance' | 'security' | 'data';
  let activeTab = $state<Tab>('general');

  const tabs = $derived<Array<{ id: Tab; label: string }>>([
    { id: 'general', label: i18n.t.tabGeneral },
    { id: 'finance', label: i18n.t.tabFinance },
    { id: 'security', label: i18n.t.tabSecurity },
    { id: 'data', label: i18n.t.tabData },
  ]);
</script>

<svelte:head>
  <title>{i18n.t.settings} — {APP_NAME}</title>
</svelte:head>

{#if !store.appState}
  <Splash />
{:else if !store.appState.configured}
  <ErrorState
    message={i18n.t.notConfiguredError ?? 'NOT CONFIGURED: Please complete vault setup.'}
  />
{:else}
  <PageLayout title={i18n.t.settings}>
    <div class="border-line mb-2 flex shrink-0 items-center justify-between gap-2 border-b pb-2">
      <div class="flex min-w-0 items-center gap-1">
        <Tabs {tabs} active={activeTab} onSelect={(t) => (activeTab = t as Tab)} />
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto pr-1">
      {#if activeTab === 'general'}
        <GeneralSettings />
      {:else if activeTab === 'finance'}
        <FinanceSettings />
      {:else if activeTab === 'security'}
        <SecuritySettings />
      {:else if activeTab === 'data'}
        <DataSettings />
      {/if}
    </div>
  </PageLayout>
{/if}

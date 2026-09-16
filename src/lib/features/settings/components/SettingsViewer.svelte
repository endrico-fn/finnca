<script lang="ts">
  import { session } from '$lib/core/state/session.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { APP_NAME } from '$lib/core/types';
  import { Splash, ErrorState, PageLayout, Tabs } from '$lib/components/ui';
  import { createTabRouter } from '$lib/core/router/tabRouter.svelte';

  import GeneralSettings from './GeneralSettings.svelte';
  import FinanceSettings from './FinanceSettings.svelte';
  import SecuritySettings from './SecuritySettings.svelte';
  import DataSettings from './DataSettings.svelte';

  const validTabs = ['general', 'finance', 'security', 'data'] as const;
  type SettingsTab = (typeof validTabs)[number];

  const tabRouter = createTabRouter<SettingsTab>('general', validTabs, 'tab');

  const tabs = $derived<Array<{ id: SettingsTab; label: string }>>([
    { id: 'general', label: i18n.t.tabGeneral },
    { id: 'finance', label: i18n.t.tabFinance },
    { id: 'security', label: i18n.t.tabSecurity },
    { id: 'data', label: i18n.t.tabData },
  ]);

  const currentTabLabel = $derived(
    tabs.find((t) => t.id === tabRouter.current)?.label ?? i18n.t.tabGeneral
  );
</script>

<svelte:head>
  <title>{i18n.t.settings} — {APP_NAME}</title>
</svelte:head>

{#if !session.raw}
  <Splash />
{:else if !session.raw.configured}
  <ErrorState message={i18n.t.notConfiguredError} />
{:else}
  <PageLayout crumb={i18n.t.settings} crumbHref="/app/setting" title={currentTabLabel}>
    <div class="border-line mb-2 flex shrink-0 items-center justify-between gap-2 border-b pb-2">
      <div class="flex min-w-0 items-center gap-1">
        <Tabs
          {tabs}
          active={tabRouter.current}
          onSelect={(t) => tabRouter.setTab(t as SettingsTab)}
        />
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto pr-1">
      {#if tabRouter.current === 'general'}
        <GeneralSettings />
      {:else if tabRouter.current === 'finance'}
        <FinanceSettings />
      {:else if tabRouter.current === 'security'}
        <SecuritySettings />
      {:else if tabRouter.current === 'data'}
        <DataSettings />
      {/if}
    </div>
  </PageLayout>
{/if}

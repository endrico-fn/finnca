<script lang="ts">
  import { session } from '$lib/core/state/session.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Splash, ErrorState, Tabs, Button } from '$lib/components/ui';
  import { createTabRouter } from '$lib/core/router/tabRouter.svelte';
  import { generalSettingsState } from '../state/settings.svelte';

  import GeneralSettings from './GeneralSettings.svelte';
  import FinanceSettings from './FinanceSettings.svelte';
  import SecuritySettings from './SecuritySettings.svelte';
  import DataSettings from './DataSettings.svelte';

  const validTabs = ['general', 'finance', 'security', 'data'] as const;
  type SettingsTab = (typeof validTabs)[number];

  const tabRouter = createTabRouter<SettingsTab>('general', validTabs, 'settingTab');

  const tabs = $derived<Array<{ id: SettingsTab; label: string }>>([
    { id: 'general', label: i18n.t.tabGeneral },
    { id: 'finance', label: i18n.t.tabFinance },
    { id: 'security', label: i18n.t.tabSecurity },
    { id: 'data', label: i18n.t.tabData },
  ]);
</script>

{#if !session.raw}
  <Splash />
{:else if !session.raw.configured}
  <ErrorState message={i18n.t.notConfiguredError} />
{:else}
  <div class="flex flex-col">
    <div class="border-line bg-bg-card z-10 mb-4 sticky top-0 -mx-4 px-4 flex items-center justify-between gap-2 border-b pb-2">
      <div class="flex min-w-0 items-center gap-1">
        <Tabs
          {tabs}
          active={tabRouter.current}
          onSelect={(t) => tabRouter.setTab(t as SettingsTab)}
        />
      </div>
      {#if tabRouter.current === 'general'}
        <Button
          variant="primary"
          class="font-proto text-small h-8 px-3 font-bold tracking-wider uppercase"
          disabled={!generalSettingsState.canSave}
          onclick={() => generalSettingsState.save()}
        >
          {generalSettingsState.saving ? i18n.t.savingBtn : i18n.t.saveChanges}
        </Button>
      {/if}
    </div>

    <div class="flex flex-col">
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
  </div>
{/if}

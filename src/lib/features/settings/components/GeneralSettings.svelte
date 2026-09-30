<script lang="ts">
  import { onMount } from 'svelte';
  import UserProfileCard from './UserProfileCard.svelte';
  import VaultIdentityCard from './VaultIdentityCard.svelte';
  import LocalizationCard from './LocalizationCard.svelte';
  import AppAboutCard from './AppAboutCard.svelte';
  import NotificationSettings from './NotificationSettings.svelte';
  import { generalSettingsState } from '../state/settings.svelte';

  onMount(() => {
    generalSettingsState.init();
  });
</script>

<div class="flex flex-col gap-4 select-none">
  <div class="grid grid-cols-1 gap-2 lg:grid-cols-2">
    <UserProfileCard />
    <VaultIdentityCard />
  </div>

  <LocalizationCard />
  <NotificationSettings />
  <AppAboutCard />
</div>

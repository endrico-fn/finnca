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

<section class="border-line bg-bg-app flex max-w-4xl flex-col border p-4 select-none sm:p-6">
  <UserProfileCard />
  <VaultIdentityCard />

  <LocalizationCard />
  <NotificationSettings />
  <AppAboutCard />
</section>

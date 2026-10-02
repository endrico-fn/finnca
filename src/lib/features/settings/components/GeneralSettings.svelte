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

<section class="border-line bg-bg-app flex flex-col border p-4 sm:p-6 select-none max-w-4xl">
  <UserProfileCard />
  <VaultIdentityCard />

  <LocalizationCard />
  <NotificationSettings />
  <AppAboutCard />
</section>

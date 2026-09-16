<script lang="ts">
  import UserProfileCard from './UserProfileCard.svelte';
  import VaultIdentityCard from './VaultIdentityCard.svelte';
  import LocalizationCard from './LocalizationCard.svelte';
</script>

<div class="flex flex-col gap-2 select-none">
  <div class="grid grid-cols-1 gap-2 lg:grid-cols-2">
    <UserProfileCard />
    <VaultIdentityCard />
  </div>

  <LocalizationCard />
</div>

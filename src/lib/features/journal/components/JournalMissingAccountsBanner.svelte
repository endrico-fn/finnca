<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { Icon, Button } from '$lib/components/ui';
  import { seedStarterAccountsCmd } from '$lib/core/ipc/bindings';
  import { eventBus } from '$lib/core/events/eventBus.svelte';

  let { onCancel }: { onCancel: () => void } = $props();
  let seeding = $state(false);

  async function handleSeed() {
    seeding = true;
    try {
      await seedStarterAccountsCmd(i18n.locale);
      eventBus.emit('accounts:changed', undefined);
    } catch (e) {
      console.error('Failed to seed starter accounts:', e);
    } finally {
      seeding = false;
    }
  }
</script>

<div class="border-warning/50 bg-warning/10 my-2 flex flex-wrap items-center justify-between gap-2 border p-2.5">
  <div class="flex items-center gap-2">
    <Icon name="chart" size={14} />
    <span class="text-warning font-proto text-small">
      {i18n.t.placeholderNotPostable}
    </span>
  </div>
  <div class="flex items-center gap-2">
    <Button variant="primary" size="sm" onclick={handleSeed} disabled={seeding}>
      {seeding ? i18n.t.savingBtn : i18n.t.seedStarterAccountsBtn}
    </Button>
    <Button variant="tactical" size="sm" href="/app/accounts" onclick={onCancel}>
      {i18n.t.chartOfAccounts}
    </Button>
  </div>
</div>

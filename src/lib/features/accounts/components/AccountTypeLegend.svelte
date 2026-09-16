<script lang="ts">
  import {
    ACCOUNT_TYPES,
    accountTypeLabel,
    ACCOUNT_TYPE_BG,
  } from '../state/accounts.svelte';
</script>

<div class="border-line bg-bg-card flex shrink-0 flex-wrap gap-1.5 border-t px-3 py-1.5">
  {#each ACCOUNT_TYPES as k (k)}
    <span
      class="bg-bg-app border-line text-text-muted font-proto text-smaller inline-flex items-center gap-1.5 border px-2 py-0.5 leading-none"
    >
      <span class="size-1.5 shrink-0 {ACCOUNT_TYPE_BG[k]}"></span>
      {accountTypeLabel(k)}
    </span>
  {/each}
</div>

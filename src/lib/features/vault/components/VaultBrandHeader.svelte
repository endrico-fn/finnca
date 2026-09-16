<script lang="ts">
  import { APP_NAME, version_app } from '$lib/core/types';
  import { Icon } from '$lib/components/ui';

  let { tagline = '' }: { tagline?: string } = $props();
</script>

<div class="mb-6 flex items-center gap-3">
  <div class="border-line bg-bg-card grid size-9 place-items-center border">
    <span class="text-teal inline-flex">
      <Icon name="lock" size={16} />
    </span>
  </div>
  <div>
    <div class="flex items-center gap-2">
      <h1 class="text-text-strong text-large font-medium tracking-[0.25em]">
        {APP_NAME}
      </h1>
      <span
        class="font-proto text-text-dim border-line bg-bg-card text-smaller border px-1.5 py-0.5"
      >
        v{version_app}
      </span>
    </div>
    {#if tagline}
      <p class="text-text-dim text-smaller mt-0.5 tracking-wider">
        {tagline}
      </p>
    {/if}
  </div>
</div>

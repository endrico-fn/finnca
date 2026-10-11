<script lang="ts">
  import { AppBrand } from '$lib/components/ui';

  let { tagline = '' }: { tagline?: string } = $props();
</script>

<AppBrand variant="hero" {tagline} />

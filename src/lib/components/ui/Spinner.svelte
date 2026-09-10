<script lang="ts">
  let { size = 'md' }: { size?: 'sm' | 'md' } = $props();
</script>

<span class={size === 'sm' ? 'spinner-sm' : 'spinner'} aria-hidden="true"></span>

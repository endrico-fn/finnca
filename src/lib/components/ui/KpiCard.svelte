<script lang="ts">
  let {
    label,
    value,
    subValue,
    labelClass = 'text-text-muted',
    valueClass = 'text-text-base',
  }: {
    label: string;
    value: string;
    subValue?: string;
    labelClass?: string;
    valueClass?: string;
  } = $props();
</script>

<div class="sharp-card space-y-1 px-3 pt-2.5 pb-2.5">
  <span class="font-proto block text-[9px] tracking-wider uppercase leading-none {labelClass}">
    {label}
  </span>
  <span class="font-proto block text-[18px] font-bold leading-tight {valueClass}">
    {value}
  </span>
  {#if subValue}
    <span class="text-text-dim font-proto block text-[10px] leading-tight">
      {subValue}
    </span>
  {/if}
</div>

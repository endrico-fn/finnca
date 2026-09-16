<script lang="ts">
  import type { Currency } from '$lib/core/types';
  import { i18n } from '$lib/core/i18n.svelte';

  let {
    description = $bindable(''),
    date = $bindable(''),
    num = $bindable(''),
    dueDate = $bindable(''),
    currency = $bindable<Currency>('IDR'),
    settled = $bindable(false),
    notes = $bindable(''),
  }: {
    description: string;
    date: string;
    num?: string;
    dueDate?: string;
    currency: Currency;
    settled?: boolean;
    notes?: string;
  } = $props();
</script>

<div class="mt-2 grid gap-2 md:grid-cols-[1fr_160px]">
  <div>
    <span class="label-xs">{i18n.t.description} *</span>
    <input
      bind:value={description}
      placeholder={i18n.t.txDescPlaceholder}
      class="sharp-input text-small mt-1 w-full"
    />
  </div>
  <div>
    <span class="label-xs">{i18n.t.date}</span>
    <input
      type="date"
      bind:value={date}
      class="sharp-input font-proto text-small mt-1 w-full"
    />
  </div>
</div>

<div class="mt-2 grid gap-2 md:grid-cols-[1fr_150px_130px]">
  <div>
    <span class="label-xs">{i18n.t.txRefInvoice}</span>
    <input
      bind:value={num}
      placeholder={i18n.t.txRefPlaceholder}
      class="sharp-input font-proto text-small mt-1 w-full"
    />
  </div>
  <div>
    <span class="label-xs">{i18n.t.txDueDateOpt}</span>
    <input
      type="date"
      bind:value={dueDate}
      class="sharp-input font-proto text-small mt-1 w-full"
    />
  </div>
  <div>
    <span class="label-xs">{i18n.t.currency}</span>
    <select bind:value={currency} class="sharp-input font-proto text-small mt-1 w-full">
      <option value="IDR">{i18n.t.currencyIdrOpt}</option>
      <option value="USD">{i18n.t.currencyUsdOpt}</option>
    </select>
  </div>
</div>

{#if dueDate}
  <div class="mt-2 flex items-center gap-2">
    <label class="text-text-base font-proto text-small flex cursor-pointer items-center gap-2">
      <input type="checkbox" bind:checked={settled} class="accent-teal" />
      {i18n.t.txMarkSettled}
    </label>
  </div>
{/if}

<div class="mt-2">
  <span class="label-xs">{i18n.t.notesMemo}</span>
  <textarea
    bind:value={notes}
    rows="2"
    class="sharp-input mt-1 w-full resize-none px-3 py-1.5"
    placeholder={i18n.t.txNotesPlaceholder}
  ></textarea>
</div>

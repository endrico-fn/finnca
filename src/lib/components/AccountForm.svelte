<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { ACCOUNT_TYPE_LABEL, ACCOUNT_TYPE_COLOR, type Account } from '$lib/accounting/types';
  import { getAccountPath } from '$lib/accounting/finance';
  import { todayString } from '$lib/accounting/finance';
  import { i18n } from '$lib/i18n.svelte';
  import { Tabs } from '$lib/components/ui';

  let {
    form = $bindable<Partial<Account>>({}),
    isNew = false,
    openingBalance = $bindable(''),
    openingBalanceDate = $bindable(todayString()),
    error = '',
  }: {
    form: Partial<Account>;
    isNew?: boolean;
    openingBalance?: string;
    openingBalanceDate?: string;
    error?: string;
  } = $props();

  let activeTab = $state<'general' | 'more' | 'balance'>('general');

  const parentOptions = $derived(
    ledger.accounts
      .filter((a) => a.placeholder || a.id === form.parentId)
      .map((a) => ({
        id: a.id,
        path: getAccountPath(a, ledger.accountsById),
        label: `${getAccountPath(a, ledger.accountsById)} (${a.code})`,
      }))
      .sort((a, b) => a.path.localeCompare(b.path))
  );

  const colorOptions = Object.values(ACCOUNT_TYPE_COLOR);
  const presetColors = [...colorOptions, '#ffffff', '#000000', ''];

  function setColor(c: string) {
    form.color = c || undefined;
  }

  const smallestFractionText = $derived(
    form.currency === 'USD' ? '1/100 (USD cents)' : '1 (IDR rupiah)'
  );
</script>

{#if error}
  <div class="badge-err px-2.5 py-1.5 text-[11px]">{error}</div>
{/if}

<!-- Tabs — sticky agar tidak ikut scroll -->
<div class="bg-bg-card border-line sticky top-0 z-10 mb-1 flex gap-1 border-b pt-1 pr-2 pb-1">
  <Tabs
    tabs={[
      { id: 'general', label: i18n.t.generalTab },
      { id: 'more', label: i18n.t.morePropertiesTab },
      { id: 'balance', label: i18n.t.openingBalanceTab },
    ]}
    active={activeTab}
    onSelect={(id) => (activeTab = id as typeof activeTab)}
  />
</div>

<div class="py-1 pr-2">
  {#if activeTab === 'general'}
    <div class="space-y-2">
      <p class="label-xs text-text-muted">{i18n.t.identification}</p>
      <div class="grid grid-cols-[110px_1fr] items-center gap-2">
        <span class="label-xs">{i18n.t.name} *</span>
        <input
          bind:value={form.name}
          placeholder={i18n.t.accountNamePlaceholder}
          class="sharp-input w-full px-2.5 py-1.5"
        />
      </div>
      <div class="grid grid-cols-[110px_1fr] items-center gap-2">
        <span class="label-xs">{i18n.t.code} *</span>
        <input bind:value={form.code} placeholder={i18n.t.accountCodePlaceholder} class="sharp-input w-full px-2.5 py-1.5" />
      </div>
      <div class="grid grid-cols-[110px_1fr] items-center gap-2">
        <span class="label-xs">{i18n.t.descriptionField}</span>
        <input
          bind:value={form.description}
          placeholder={i18n.t.accountDescPlaceholder}
          class="sharp-input w-full px-2.5 py-1.5"
        />
      </div>
      <div class="grid grid-cols-[110px_1fr] items-center gap-2">
        <span class="label-xs">{i18n.t.parent}</span>
        <select bind:value={form.parentId} class="sharp-input w-full px-2.5 py-1.5">
          <option value={null}>— ROOT (PRIMARY) —</option>
          {#each parentOptions as opt (opt.id)}
            <option value={opt.id}>{opt.label}</option>
          {/each}
        </select>
      </div>
      <div class="grid grid-cols-[110px_1fr] items-center gap-2">
        <span class="label-xs">{i18n.t.type} *</span>
        <select bind:value={form.type} class="sharp-input w-full px-2.5 py-1.5">
          {#each Object.entries(ACCOUNT_TYPE_LABEL) as [k, v] (k)}
            <option value={k}>{v} ({k})</option>
          {/each}
        </select>
      </div>
      <div class="grid grid-cols-[110px_1fr] items-center gap-2">
        <span class="label-xs">{i18n.t.currency}</span>
        <select bind:value={form.currency} class="sharp-input w-full px-2.5 py-1.5">
          <option value="IDR">IDR — Indonesian Rupiah</option>
          <option value="USD">USD — US Dollar</option>
        </select>
      </div>
      <div class="grid grid-cols-[110px_1fr] items-center gap-2">
        <span class="label-xs">{i18n.t.smallestFraction}</span>
        <div
          class="sharp-input text-text-muted flex w-full items-center justify-between px-2.5 py-1.5"
        >
          <span>{i18n.t.useCommodityValue} — {smallestFractionText}</span>
          <span class="text-text-muted text-[10px]">auto</span>
        </div>
      </div>
      <div class="grid grid-cols-[110px_1fr] gap-2">
        <span class="label-xs pt-2">{i18n.t.accountColor}</span>
        <div class="flex flex-wrap items-center gap-1.5">
          <button
            type="button"
            onclick={() => setColor('')}
            class="border px-2.5 py-1 text-[11px] {!form.color
              ? 'bg-line text-text-white border-line'
              : 'bg-bg-app text-text-muted border-line hover:border-text-muted'}">{i18n.t.defaultColorLabel}</button
          >
          {#each presetColors.filter((c) => c) as c (c)}
            <button
              type="button"
              onclick={() => setColor(c)}
              class="flex size-6 items-center justify-center border transition-all {form.color === c
                ? 'border-teal ring-teal ring-1'
                : 'border-line hover:border-text-muted'}"
              style="background:{c}"
              title={c}
              aria-label="color {c}"
            >
              {#if form.color === c}<span
                  class="text-[10px]"
                  style="color:{c === '#ffffff' ? '#000' : '#fff'}">✓</span
                >{/if}
            </button>
          {/each}
          <label class="ml-1 flex items-center gap-1">
            <input
              type="color"
              value={form.color ?? '#4ea398'}
              oninput={(e) => setColor((e.target as HTMLInputElement).value)}
              class="border-line size-6 cursor-pointer border bg-transparent p-0"
            />
            <span class="text-text-muted text-[10px]">{i18n.t.customColorLabel}</span>
          </label>
        </div>
      </div>
      <div class="grid grid-cols-[110px_1fr] gap-2">
        <span class="label-xs pt-2">{i18n.t.note}</span>
        <textarea
          bind:value={form.note}
          rows="2"
          placeholder={i18n.t.accountNotePlaceholder}
          class="sharp-input w-full resize-none px-2.5 py-1.5"></textarea>
      </div>
    </div>
  {:else if activeTab === 'more'}
    <div class="space-y-4">
      <p class="label-xs text-text-muted">
        {i18n.t.balanceLimitTitle}
      </p>
      <div class="border-line bg-bg-app space-y-2 border p-3">
        <p class="text-text-muted text-[10px] leading-relaxed">
          {i18n.t.placeholderHiddenDesc}
        </p>
        <label
          class="border-line text-text-base flex cursor-pointer items-center gap-2 border-y py-2 text-[11px] uppercase"
        >
          <input type="checkbox" bind:checked={form.placeholder} class="accent-teal" />
          {i18n.t.placeholderGroup}
        </label>
        <label class="text-text-base flex cursor-pointer items-center gap-2 text-[11px] uppercase">
          <input type="checkbox" bind:checked={form.hidden} class="accent-teal" />
          {i18n.t.hiddenAccount}
        </label>
      </div>
    </div>
  {:else}
    <div class="space-y-4">
      <div>
        <p class="label-xs mb-2">{i18n.t.balanceInformation}</p>
        <div class="border-line bg-bg-app space-y-2 border p-3">
          <div class="grid grid-cols-[110px_1fr] items-center gap-2">
            <span class="label-xs">{i18n.t.colBalance}</span>
            <input
              bind:value={openingBalance}
              placeholder="0.00"
              type="text"
              inputmode="decimal"
              class="sharp-input w-full px-2.5 py-1.5 text-right"
            />
          </div>
          <div class="grid grid-cols-[110px_1fr] items-center gap-2">
            <span class="label-xs">{i18n.t.date}</span>
            <input
              type="date"
              bind:value={openingBalanceDate}
              class="sharp-input w-full px-2.5 py-1.5"
            />
          </div>
          <p class="text-text-muted text-[10px] leading-relaxed">
            {#if isNew}
              {i18n.t.openingBalanceHintNew}
            {:else}
              {i18n.t.openingBalanceHintEdit}
            {/if}
          </p>
        </div>
      </div>
      <div>
        <p class="label-xs mb-2">{i18n.t.initialBalanceTransfer}</p>
        <div class="border-line bg-bg-app border p-3">
          <label class="text-text-base flex items-center gap-2 text-[11px]">
            <input type="radio" checked disabled class="accent-teal" />
            {i18n.t.openingBalanceEquityLabel}
          </label>
          <p class="text-text-muted mt-1 ml-6 text-[10px]">
            {i18n.t.openingBalanceEquityHint}
          </p>
        </div>
      </div>
    </div>
  {/if}
</div>

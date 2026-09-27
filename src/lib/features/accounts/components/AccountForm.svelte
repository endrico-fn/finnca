<script lang="ts">
  import {
    accountsState,
    ACCOUNT_TYPES,
    accountTypeLabel,
    ACCOUNT_TYPE_COLOR,
    getAccountPath,
  } from '$lib/features/accounts/state/accounts.svelte';
  import type { Account, AccountType } from '$lib/core/ipc/bindings';
  import { todayString } from '$lib/core/format/date';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Tabs, SelectDropdown, Button } from '$lib/components/ui';
  import { resolveOpeningEquity } from '$lib/features/accounts/state/openingBalance';
  import {
    suggestAccountCode,
    isOutsideParentPrefix,
  } from '$lib/features/accounts/state/accountCode';

  let {
    form = $bindable<Partial<Account>>({}),
    isNew = false,
    openingBalance = $bindable(''),
    openingBalanceDate = $bindable(todayString()),
    error = '',
    activeTab = $bindable<'general' | 'more' | 'balance'>('general'),
  }: {
    form: Partial<Account>;
    isNew?: boolean;
    openingBalance?: string;
    openingBalanceDate?: string;
    error?: string;
    activeTab?: 'general' | 'more' | 'balance';
  } = $props();

  const parentOptions = $derived(
    accountsState.accounts
      .filter((a) => a.placeholder || a.id === form.parent_id)
      .map((a) => ({
        id: a.id,
        path: getAccountPath(a, accountsState.accountsById),
        label: `${getAccountPath(a, accountsState.accountsById)} (${a.code})`,
      }))
      .sort((a, b) => a.path.localeCompare(b.path))
  );

  const colorOptions = Object.values(ACCOUNT_TYPE_COLOR);
  const ACCOUNT_LIGHT_SWATCH = 'var(--color-text-white)';
  const ACCOUNT_DARK_SWATCH = 'var(--color-bg-app)';
  const presetColors = [...colorOptions, ACCOUNT_LIGHT_SWATCH, ACCOUNT_DARK_SWATCH, ''];
  const DEFAULT_SWATCH_HEX = '#4ea398';

  function setColor(c: string) {
    form.color = c || null;
  }

  function normalizeSwatch(c: string): string {
    const n = c.trim().toLowerCase();
    if (n === '#ffffff' || n === '#fff') return ACCOUNT_LIGHT_SWATCH.toLowerCase();
    if (n === '#000000' || n === '#000') return ACCOUNT_DARK_SWATCH.toLowerCase();
    return n;
  }

  function isActiveSwatch(c: string): boolean {
    return normalizeSwatch(form.color ?? '') === normalizeSwatch(c);
  }

  function isLightSwatch(c: string): boolean {
    const n = normalizeSwatch(c);
    if (n === ACCOUNT_LIGHT_SWATCH.toLowerCase()) return true;
    if (n === ACCOUNT_DARK_SWATCH.toLowerCase()) return false;
    const hex = n.match(/^#([0-9a-f]{6})$/);
    if (!hex) return false;
    const v = parseInt(hex[1], 16);
    const luminance =
      (0.2126 * ((v >> 16) & 255) + 0.7152 * ((v >> 8) & 255) + 0.0722 * (v & 255)) / 255;
    return luminance > 0.6;
  }

  const smallestFractionText = $derived(
    form.currency === 'USD' ? i18n.t.smallestFractionUsd : i18n.t.smallestFractionIdr
  );

  const parentAccount = $derived(
    form.parent_id ? (accountsState.accountsById.get(form.parent_id) ?? null) : null
  );
  const codeSuggestion = $derived.by(() => {
    if (!isNew || form.code?.trim()) return '';
    if (!parentAccount) return '';
    const taken = new Set(accountsState.accounts.map((a) => a.code));
    return suggestAccountCode(parentAccount.code, taken);
  });
  const codeOutsidePrefix = $derived.by(() => {
    if (!form.code?.trim() || !parentAccount) return false;
    return isOutsideParentPrefix(form.code.trim(), parentAccount.code);
  });
  const equityPreview = $derived.by(() => {
    if (!openingBalance.trim() || form.placeholder) return null;
    const cur = form.currency ?? 'IDR';
    const found = resolveOpeningEquity(accountsState.accounts, cur, form.id);
    if (found) {
      return i18n.t.openingBalanceWillUse
        .replace('{code}', found.code)
        .replace('{name}', found.name);
    }
    return i18n.t.openingBalanceWillCreate
      .replace('{code}', '3110')
      .replace('{name}', i18n.t.openingBalanceAutoName.replace('{cur}', cur));
  });
</script>

{#if error}
  <div class="badge-err font-proto text-small px-2.5 py-1.5">{error}</div>
{/if}

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
        <div class="w-full">
          <input
            bind:value={form.code}
            placeholder={i18n.t.accountCodePlaceholder}
            class="sharp-input font-proto w-full px-2.5 py-1.5 tabular-nums"
          />
          {#if codeSuggestion}
            <button
              type="button"
              onclick={() => (form.code = codeSuggestion)}
              class="text-text-muted font-proto text-smaller hover:text-teal mt-1 cursor-pointer underline underline-offset-2"
            >
              {i18n.t.codeSuggestUse.replace('{code}', codeSuggestion)}
            </button>
          {/if}
          {#if codeOutsidePrefix && parentAccount}
            <p class="text-warning font-proto text-smaller mt-1 leading-relaxed">
              {i18n.t.codePrefixWarning
                .replace('{code}', form.code?.trim() ?? '')
                .replace('{prefix}', parentAccount.code)}
            </p>
          {/if}
        </div>
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
        <SelectDropdown
          value={form.parent_id ?? ''}
          onSelect={(v) => (form.parent_id = v || null)}
          placeholder={i18n.t.accountRootOpt}
          options={parentOptions.map((opt) => ({ value: opt.id, label: opt.label }))}
          class="w-full"
        />
      </div>
      <div class="grid grid-cols-[110px_1fr] items-center gap-2">
        <span class="label-xs">{i18n.t.type} *</span>
        <SelectDropdown
          value={form.account_type ?? ''}
          onSelect={(v) => (form.account_type = v as AccountType)}
          placeholder={i18n.t.accountTypeRequired}
          options={ACCOUNT_TYPES.map((k) => ({
            value: k,
            label: `${accountTypeLabel(k)} (${k})`,
          }))}
          class="w-full"
        />
      </div>
      <div class="grid grid-cols-[110px_1fr] items-center gap-2">
        <span class="label-xs">{i18n.t.currency}</span>
        <SelectDropdown
          value={form.currency ?? ''}
          onSelect={(v) => (form.currency = v || 'IDR')}
          options={[
            { value: 'IDR', label: i18n.t.currencyIdrOpt },
            { value: 'USD', label: i18n.t.currencyUsdOpt },
          ]}
          class="w-full"
        />
      </div>
      <div class="grid grid-cols-[110px_1fr] items-center gap-2">
        <span class="label-xs">{i18n.t.smallestFraction}</span>
        <div
          class="sharp-input text-text-muted flex w-full items-center justify-between px-2.5 py-1.5"
        >
          <span>{i18n.t.useCommodityValue} — {smallestFractionText}</span>
          <span class="text-text-muted text-smaller">{i18n.t.commonAuto}</span>
        </div>
      </div>
      <div class="grid grid-cols-[110px_1fr] gap-2">
        <span class="label-xs pt-2">{i18n.t.accountColor}</span>
        <div class="flex flex-wrap items-center gap-1.5">
          <Button
            variant={!form.color ? 'primary' : 'ghost'}
            size="sm"
            pressed={!form.color}
            onclick={() => setColor('')}>{i18n.t.defaultColorLabel}</Button
          >
          {#each presetColors.filter((c) => c) as c (c)}
            <button
              type="button"
              onclick={() => setColor(c)}
              aria-pressed={isActiveSwatch(c)}
              class="flex size-6 items-center justify-center border transition-colors {isActiveSwatch(
                c
              )
                ? 'border-teal'
                : 'border-line hover:border-text-muted'}"
              style="background:{c}"
              title={c}
              aria-label={i18n.t.accountColorAria.replace('{c}', c)}
            >
              {#if isActiveSwatch(c)}<span
                  class="font-proto text-smaller font-bold"
                  style="color:{isLightSwatch(c)
                    ? 'var(--color-bg-app)'
                    : 'var(--color-text-white)'}">✓</span
                >{/if}
            </button>
          {/each}
          <label class="ml-1 flex items-center gap-1">
            <input
              type="color"
              value={form.color?.startsWith('#') ? form.color : DEFAULT_SWATCH_HEX}
              oninput={(e) => setColor((e.target as HTMLInputElement).value)}
              class="border-line size-6 cursor-pointer border bg-transparent p-0"
            />
            <span class="text-text-muted text-smaller">{i18n.t.customColorLabel}</span>
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
        <p class="text-text-muted text-smaller leading-relaxed">
          {i18n.t.placeholderHiddenDesc}
        </p>
        <label
          class="border-line text-text-base text-small flex cursor-pointer items-center gap-2 border-y py-2 uppercase"
        >
          <input type="checkbox" bind:checked={form.placeholder} class="accent-teal" />
          {i18n.t.placeholderGroup}
        </label>
        <label class="text-text-base text-small flex cursor-pointer items-center gap-2 uppercase">
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
              placeholder={i18n.t.commonZeroPlaceholder}
              type="text"
              inputmode="decimal"
              class="sharp-input font-proto w-full px-2.5 py-1.5 text-right tabular-nums"
            />
          </div>
          <div class="grid grid-cols-[110px_1fr] items-center gap-2">
            <span class="label-xs">{i18n.t.date}</span>
            <input
              type="date"
              bind:value={openingBalanceDate}
              class="sharp-input font-proto w-full px-2.5 py-1.5 tabular-nums"
            />
          </div>
          <p class="text-text-muted text-smaller leading-relaxed">
            {#if isNew}
              {i18n.t.openingBalanceHintNew}
            {:else}
              {i18n.t.openingBalanceHintEdit}
            {/if}
          </p>
          {#if equityPreview}
            <p class="font-proto text-smaller text-teal mt-1">{equityPreview}</p>
          {/if}
        </div>
      </div>
      <div>
        <p class="label-xs mb-2">{i18n.t.initialBalanceTransfer}</p>
        <div class="border-line bg-bg-app border p-3">
          <label class="text-text-base text-small flex items-center gap-2">
            <input type="radio" checked disabled class="accent-teal" />
            {i18n.t.openingBalanceEquityLabel}
          </label>
          <p class="text-text-muted text-smaller mt-1 ml-6">
            {i18n.t.openingBalanceEquityHint}
          </p>
        </div>
      </div>
    </div>
  {/if}
</div>

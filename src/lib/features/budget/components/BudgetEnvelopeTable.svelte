<script lang="ts">
  import { formatIDR, fromMinor, toMinor } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Badge } from '$lib/components/ui';
  import type { MonthCalculation } from '../state/budget.svelte';

  let {
    budgetData,
    onAssignBudget,
  }: {
    budgetData: MonthCalculation;
    onAssignBudget: (accountId: string, amountIdrMinor: number) => Promise<void>;
  } = $props();
</script>

<div class="sharp-card flex w-full flex-1 flex-col overflow-y-auto">
  <!-- Table Header -->
  <div
    class="border-line bg-line/20 font-proto text-text-muted text-smaller sticky top-0 z-10 grid grid-cols-12 gap-2 border-b px-3 py-2 tracking-widest uppercase"
  >
    <div class="col-span-4">{i18n.t.budgetCategoryEnvelope}</div>
    <div class="col-span-3 text-right">{i18n.t.budgetAssignedThisMonth}</div>
    <div class="col-span-2 text-right">{i18n.t.budgetActivity}</div>
    <div class="col-span-3 text-right">{i18n.t.budgetAvailable}</div>
  </div>

  <!-- Rows -->
  <div class="divide-line/40 flex-1 divide-y">
    {#each budgetData.envelopes as env (env.accountId)}
      {@const usagePercent =
        env.assigned > 0
          ? Math.min(100, Math.round((env.activity / env.assigned) * 100))
          : env.activity > 0
            ? 100
            : 0}
      <div
        class="font-proto hover:bg-bg-btn text-small grid grid-cols-12 items-center gap-2 px-3 py-2 transition-colors"
      >
        <!-- Category Envelope Info & Micro Progress -->
        <div class="col-span-4 flex flex-col gap-1 pr-2">
          <div class="flex items-center gap-2">
            <span class="text-text-muted font-proto text-smaller">{env.accountCode || '—'}</span>
            <span class="text-text-strong truncate font-medium">{env.accountName}</span>
          </div>
          <div class="bg-line/40 h-1 w-full overflow-hidden">
            <div
              class="h-full transition-all duration-300 {env.available < 0
                ? 'bg-expense'
                : usagePercent >= 90
                  ? 'bg-warning'
                  : 'bg-teal'}"
              style="width: {usagePercent}%"
            ></div>
          </div>
        </div>

        <!-- Assigned Input -->
        <div class="col-span-3">
          <div
            class="border-line bg-bg-app focus-within:border-teal relative w-full border transition-colors"
          >
            <span class="text-text-dim text-smaller absolute top-1/2 left-2 -translate-y-1/2">
              Rp
            </span>
            <input
              type="number"
              value={fromMinor('IDR', env.assigned)}
              onchange={(e) =>
                onAssignBudget(env.accountId, toMinor('IDR', Number(e.currentTarget.value)))}
              class="text-text-strong font-proto text-small w-full bg-transparent py-1 pr-2 pl-6 text-right font-bold tabular-nums focus:outline-none"
            />
          </div>
        </div>

        <!-- Actual Activity -->
        <div class="text-text-muted font-proto col-span-2 text-right tabular-nums">
          {formatIDR(env.activity)}
        </div>

        <!-- Available Balance Pill -->
        <div class="font-proto col-span-3 text-right font-bold tabular-nums">
          <Badge size="m" tone={env.available > 0 ? 'ok' : env.available === 0 ? 'neutral' : 'err'}>
            {formatIDR(env.available)}
          </Badge>
        </div>
      </div>
    {/each}
  </div>

  <!-- Total Footer -->
  <div
    class="font-proto text-text-strong bg-line/20 border-line text-small mt-auto grid shrink-0 grid-cols-12 gap-2 border-t px-3 py-2.5 tracking-widest uppercase"
  >
    <div class="col-span-4 pt-0.5 text-right">{i18n.t.totals}</div>
    <div class="text-teal font-proto col-span-3 pt-0.5 text-right font-bold tabular-nums">
      {formatIDR(budgetData.totalAssigned)}
    </div>
    <div class="text-expense font-proto col-span-2 pt-0.5 text-right font-bold tabular-nums">
      {formatIDR(budgetData.totalActivity)}
    </div>
    <div class="col-span-3"></div>
  </div>
</div>

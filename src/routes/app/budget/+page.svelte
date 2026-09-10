<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { calculateBudgetMonth, formatIDR, fromMinor, toMinor } from '$lib/accounting/finance';

  import { PageLayout, Icon, Button, Card } from '$lib/components/ui';
  import { i18n } from '$lib/i18n.svelte';
  import { notifStore } from '$lib/notifications/store.svelte';

  let dateObj = $state(new Date());

  const currentMonth = $derived(
    dateObj.getFullYear() + '-' + String(dateObj.getMonth() + 1).padStart(2, '0')
  );

  const monthName = $derived(
    dateObj
      .toLocaleDateString(i18n.locale === 'id' ? 'id-ID' : 'en-US', {
        month: 'long',
        year: 'numeric',
      })
      .toUpperCase()
  );

  function prevMonth() {
    const d = new Date(dateObj);
    d.setMonth(d.getMonth() - 1);
    dateObj = d;
  }

  function nextMonth() {
    const d = new Date(dateObj);
    d.setMonth(d.getMonth() + 1);
    dateObj = d;
  }

  const budgetData = $derived(
    ledger.data
      ? calculateBudgetMonth(ledger.data, currentMonth)
      : { envelopes: [], totalAssigned: 0, totalActivity: 0, toBeBudgeted: 0 }
  );

  async function assignBudget(accountId: string, amountIdrMinor: number) {
    if (!ledger.data) return;

    let b = ledger.data.budgets?.find((b) => b.accountId === accountId && b.month === currentMonth);

    if (b) {
      if (amountIdrMinor === 0) {
        await ledger.deleteBudget(b.id);
      } else {
        b.amount = amountIdrMinor;
        await ledger.upsertBudget(b);
      }
    } else if (amountIdrMinor !== 0) {
      const { uid } = await import('$lib/accounting/finance');
      await ledger.upsertBudget({
        id: uid(),
        month: currentMonth,
        accountId,
        amount: amountIdrMinor,
      });
    }
  }

  async function copyPreviousMonth() {
    if (!ledger.data) return;
    const prevD = new Date(dateObj);
    prevD.setMonth(prevD.getMonth() - 1);
    const prevMonthStr = `${prevD.getFullYear()}-${String(prevD.getMonth() + 1).padStart(2, '0')}`;

    const prevBudgets =
      ledger.data.budgets?.filter((b) => b.month === prevMonthStr && b.amount > 0) ?? [];

    if (prevBudgets.length === 0) {
      notifStore.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.budgetEnvelopeTitle,
        message: i18n.t.budgetRolloverEmpty,
      });
      return;
    }

    for (const b of prevBudgets) {
      await assignBudget(b.accountId, b.amount);
    }

    notifStore.addNotification({
      type: 'LEDGER_INTEGRITY',
      priority: 'low',
      title: i18n.t.budgetEnvelopeTitle,
      message: i18n.t.budgetRolloverSuccess,
    });
  }
</script>

<PageLayout title={i18n.t.budget}>
  {#snippet actions()}
    <div class="flex items-center gap-2">
      <Button
        variant="ghost"
        size="sm"
        onclick={copyPreviousMonth}
        title={i18n.t.budgetRolloverBtn}
      >
        <span class="font-proto inline-flex items-center gap-1.5 text-[10px]">
          <Icon name="refresh" size={11} />
          {i18n.t.budgetRolloverBtn}
        </span>
      </Button>

      <div class="border-line bg-bg-card flex h-7 items-center gap-1 border px-1">
        <Button
          variant="pager"
          size="icon"
          onclick={prevMonth}
          ariaLabel={i18n.t.prevMonth}
          title={i18n.t.prevMonth}><Icon name="chev-left" size={12} /></Button
        >
        <span
          class="font-proto text-text-strong min-w-28 text-center text-[11px] font-bold tracking-widest"
          >{monthName}</span
        >
        <Button
          variant="pager"
          size="icon"
          onclick={nextMonth}
          ariaLabel={i18n.t.nextMonth}
          title={i18n.t.nextMonth}><Icon name="chev-right" size={12} /></Button
        >
      </div>
    </div>
  {/snippet}

  <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
    <!-- Telemetry HUD Summary Strip -->
    <div class="grid shrink-0 grid-cols-1 gap-2 md:grid-cols-3">
      <!-- To Be Budgeted Card -->
      <Card
        title={i18n.t.budgetReadyToAssign}
        class={budgetData.toBeBudgeted === 0
          ? ''
          : budgetData.toBeBudgeted > 0
            ? 'bg-income/5 border-income/30'
            : 'bg-expense/5 border-expense/30'}
      >
        {#snippet header()}
            <span
              class="font-proto border px-1.5 py-0.5 text-[9px] {budgetData.toBeBudgeted === 0
                ? 'border-income/40 text-income bg-income/10'
                : budgetData.toBeBudgeted > 0
                  ? 'border-line text-text-dim'
                  : 'border-expense/40 text-expense bg-expense/10'}"
            >
              {budgetData.toBeBudgeted === 0
                ? i18n.t.budgetStatusBalanced
                : budgetData.toBeBudgeted > 0
                  ? i18n.t.budgetStatusUnassigned
                  : i18n.t.budgetStatusDeficit}
            </span>
        {/snippet}

        <div class="mt-2 flex flex-col pt-1">
          <span
            class="font-proto truncate text-[20px] font-bold tabular-nums {budgetData.toBeBudgeted ===
            0
              ? 'text-text-strong'
              : budgetData.toBeBudgeted > 0
                ? 'text-income'
                : 'text-expense'}"
            title={formatIDR(fromMinor('IDR', budgetData.toBeBudgeted))}
          >
            {formatIDR(fromMinor('IDR', budgetData.toBeBudgeted))}
          </span>
          <span class="text-text-dim font-proto mt-1 text-[10px]">
            {#if budgetData.toBeBudgeted > 0}
              {i18n.t.budgetGiveDollarJob}
            {:else if budgetData.toBeBudgeted < 0}
              {i18n.t.budgetAssignedMore}
            {:else}
              {i18n.t.budgetAllFundsAssigned}
            {/if}
          </span>
        </div>
      </Card>

      <!-- Total Assigned Card -->
      <Card title={i18n.t.budgetAssignedThisMonth}>
        {#snippet header()}
          <span class="font-proto text-text-dim ml-auto text-[9px] tracking-widest">{i18n.t.budgetAllocatedLabel}</span>
        {/snippet}
        <div class="mt-2 flex flex-col pt-1">
          <span
            class="font-proto text-text-strong truncate text-[20px] font-bold tabular-nums"
            title={formatIDR(fromMinor('IDR', budgetData.totalAssigned))}
          >
            {formatIDR(fromMinor('IDR', budgetData.totalAssigned))}
          </span>
          <span class="text-text-dim font-proto mt-1 text-[10px]">
            {budgetData.envelopes.filter((e) => e.assigned > 0).length} {i18n.t.envelopesLabel}
          </span>
        </div>
      </Card>

      <!-- Total Activity Card -->
      <Card title={i18n.t.budgetActivity}>
        {#snippet header()}
          <span class="font-proto text-text-dim ml-auto text-[9px] tracking-widest">{i18n.t.budgetOutflowLabel}</span>
        {/snippet}
        <div class="mt-2 flex flex-col pt-1">
          <span
            class="font-proto text-expense truncate text-[20px] font-bold tabular-nums"
            title={formatIDR(fromMinor('IDR', budgetData.totalActivity))}
          >
            {formatIDR(fromMinor('IDR', budgetData.totalActivity))}
          </span>
          <span class="text-text-dim font-proto mt-1 text-[10px]">
            {budgetData.totalAssigned > 0
              ? i18n.t.percentSpent.replace('{pct}', String(Math.round((budgetData.totalActivity / budgetData.totalAssigned) * 100)))
              : '0%'}
          </span>
        </div>
      </Card>
    </div>

    <!-- Envelopes Table Grid -->
    <div class="sharp-card flex w-full flex-1 flex-col overflow-y-auto">
      <!-- Table Header -->
      <div
        class="border-line bg-line/20 font-proto text-text-muted sticky top-0 z-10 grid grid-cols-12 gap-2 border-b px-3 py-2 text-[10px] tracking-widest uppercase"
      >
        <div class="col-span-4">{i18n.t.budgetCategoryEnvelope}</div>
        <div class="col-span-3 text-right">{i18n.t.budgetAssignedThisMonth}</div>
        <div class="col-span-2 text-right">{i18n.t.budgetActivity}</div>
        <div class="col-span-3 text-right">{i18n.t.budgetAvailable}</div>
      </div>

      <!-- Rows -->
      <div class="divide-line/40 flex-1 divide-y">
        {#each budgetData.envelopes as env (env.accountId)}
          {@const acc = ledger.accounts.find((a) => a.id === env.accountId)}
          {@const usagePercent =
            env.assigned > 0
              ? Math.min(100, Math.round((env.activity / env.assigned) * 100))
              : env.activity > 0
                ? 100
                : 0}
          <div
            class="font-proto hover:bg-bg-row-hover grid grid-cols-12 items-center gap-2 px-3 py-2 text-xs transition-colors"
          >
            <!-- Category Envelope Info & Micro Progress -->
            <div class="col-span-4 flex flex-col gap-1 pr-2">
              <div class="flex items-center gap-2">
                <span class="text-text-muted font-proto text-[10px]">{acc?.code ?? '—'}</span>
                <span class="text-text-strong truncate font-medium">{acc?.name}</span>
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
                <span class="text-text-dim absolute top-1/2 left-2 -translate-y-1/2 text-[9px]"
                  >Rp</span
                >
                <input
                  type="number"
                  value={fromMinor('IDR', env.assigned)}
                  onchange={(e) =>
                    assignBudget(env.accountId, toMinor('IDR', Number(e.currentTarget.value)))}
                  class="text-text-strong w-full bg-transparent py-1 pr-2 pl-6 text-right text-xs font-bold tabular-nums font-proto focus:outline-none"
                />
              </div>
            </div>

            <!-- Actual Activity -->
            <div class="text-text-muted col-span-2 text-right tabular-nums font-proto">
              {formatIDR(fromMinor('IDR', env.activity))}
            </div>

            <!-- Available Balance Pill -->
            <div class="col-span-3 text-right font-bold tabular-nums font-proto">
              <span
                class="font-proto inline-block border px-2 py-0.5 text-[11px] {env.available >= 0
                  ? env.available === 0
                    ? 'text-text-dim border-line bg-line/10'
                    : 'bg-income/10 text-income border-income/30'
                  : 'bg-expense/10 text-expense border-expense/30'}"
              >
                {formatIDR(fromMinor('IDR', env.available))}
              </span>
            </div>
          </div>
        {/each}
      </div>

      <!-- Total Footer -->
      <div
        class="font-proto text-text-strong bg-line/20 border-line mt-auto grid shrink-0 grid-cols-12 gap-2 border-t px-3 py-2.5 text-[11px] tracking-widest uppercase"
      >
        <div class="col-span-4 pt-0.5 text-right">{i18n.t.totals}</div>
        <div class="text-teal col-span-3 pt-0.5 text-right font-bold tabular-nums font-proto">
          {formatIDR(fromMinor('IDR', budgetData.totalAssigned))}
        </div>
        <div class="text-expense col-span-2 pt-0.5 text-right font-bold tabular-nums font-proto">
          {formatIDR(fromMinor('IDR', budgetData.totalActivity))}
        </div>
        <div class="col-span-3"></div>
      </div>
    </div>
  </div>
</PageLayout>

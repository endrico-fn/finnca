<script lang="ts">
  import { page } from '$app/state';
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import {
    trialBalance,
    totalDebitsCredits,
    incomeStatement,
    balanceSheet,
    fromMinor,
    convertMinor,
    accountBalanceMinor,
    accountBalanceMinorFiltered,
    buildChildrenMap,
    historicalDailyBalances,
    todayString,
  } from '$lib/accounting/finance';
  import { save } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { notifStore } from '$lib/notifications/store.svelte';
  import { onMount } from 'svelte';
  import { PageLayout, Tabs, Button, DateRangeDropdown } from '$lib/components/ui';
  import TrialBalance from './_views/TrialBalance.svelte';
  import DebtReport from './_views/DebtReport.svelte';
  import BalanceSheet from './_views/BalanceSheet.svelte';
  import ProfitLoss from './_views/ProfitLoss.svelte';
  import Trends from './_views/Trends.svelte';
  import Cashflow from './_views/Cashflow.svelte';
  import FxReport from './_views/FxReport.svelte';

  import ExportOverlay from '$lib/components/ExportOverlay.svelte';

  let reportTab = $state<'pnl' | 'bs' | 'tb' | 'trends' | 'debt' | 'cashflow' | 'fx'>('pnl');
  let from = $state('');
  let to = $state('');
  let exportOpen = $state(false);

  const currentTabLabel = $derived.by(() => {
    switch (reportTab) {
      case 'pnl':
        return i18n.t.pnlTitle;
      case 'bs':
        return i18n.t.balanceSheetTitle;
      case 'tb':
        return i18n.t.trialBalanceTitle;
      case 'trends':
        return i18n.t.trendsTitle;
      case 'debt':
        return i18n.t.debtReportTitle;
      case 'cashflow':
        return i18n.t.cashflowSankeyTitle;
      case 'fx':
        return i18n.t.fxRevaluationTitle;
      default:
        return i18n.t.report;
    }
  });

  function setReportTab(t: 'pnl' | 'bs' | 'tb' | 'trends' | 'debt' | 'cashflow' | 'fx') {
    reportTab = t;
    if (typeof window !== 'undefined') {
      const url = new URL(window.location.href);
      url.searchParams.set('tab', t);
      window.history.replaceState({}, '', url.toString());
    }
  }

  onMount(() => {
    if (!ledger.data) ledger.load();
    const p = page.url.searchParams.get('tab');
    if (p && ['pnl', 'bs', 'tb', 'trends', 'debt', 'cashflow', 'fx'].includes(p)) {
      reportTab = p as typeof reportTab;
    }
  });

  const tb = $derived(ledger.data ? trialBalance(ledger.data) : []);
  const totals = $derived(totalDebitsCredits(tb));

  const pnl = $derived(
    ledger.data
      ? incomeStatement(ledger.data, from || undefined, to || undefined)
      : { income: 0, expense: 0, net: 0 }
  );

  const bs = $derived(
    ledger.data
      ? balanceSheet(ledger.data, undefined, ledger.childrenMap)
      : { assets: 0, liabilities: 0, equity: 0, netIncome: 0, balanced: false }
  );

  const childrenMap = $derived(ledger.data ? buildChildrenMap(ledger.data.accounts) : new Map());
  const incomeAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'INCOME' && !a.placeholder)
  );
  const expenseAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'EXPENSE' && !a.placeholder)
  );
  const assetAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'ASSET' && !a.placeholder)
  );
  const liabilityAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'LIABILITY' && !a.placeholder)
  );
  const equityAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'EQUITY' && !a.placeholder)
  );

  // Unified Date Range Dropdown manages its own quick filters now.

  let trendsPeriod = $state<'1W' | '1M' | '3M' | 'YTD' | '1Y' | 'ALL'>('1M');
  let trendsMetric = $state<'netWorth' | 'assets' | 'liabilities' | 'liquidCash'>('netWorth');

  const trendsDateRange = $derived.by(() => {
    const today = new Date();
    const todayStr = todayString(today);
    let startStr = todayStr;

    if (trendsPeriod === '1W') {
      const d = new Date(today);
      d.setDate(d.getDate() - 6);
      startStr = todayString(d);
    } else if (trendsPeriod === '1M') {
      const d = new Date(today);
      d.setDate(d.getDate() - 29);
      startStr = todayString(d);
    } else if (trendsPeriod === '3M') {
      const d = new Date(today);
      d.setDate(d.getDate() - 89);
      startStr = todayString(d);
    } else if (trendsPeriod === 'YTD') {
      startStr = `${today.getFullYear()}-01-01`;
    } else if (trendsPeriod === '1Y') {
      const d = new Date(today);
      d.setDate(d.getDate() - 364);
      startStr = todayString(d);
    } else if (trendsPeriod === 'ALL') {
      if (ledger.transactions.length > 0) {
        const sorted = [...ledger.transactions].sort((a, b) => a.date.localeCompare(b.date));
        startStr = sorted[0].date;
      } else {
        const d = new Date(today);
        d.setDate(d.getDate() - 29);
        startStr = todayString(d);
      }
    }

    return { startStr, endStr: todayStr };
  });

  const historicalPoints = $derived.by(() => {
    if (!ledger.data) return [];
    return historicalDailyBalances(ledger.data, trendsDateRange.startStr, trendsDateRange.endStr);
  });

  async function exportCSV(rangeFrom: string = from, rangeTo: string = to) {
    if (!ledger.data) return;
    let csvRows: string[] = [];
    const activeFrom = rangeFrom;
    const activeTo = rangeTo;

    if (reportTab === 'pnl') {
      csvRows.push('TYPE,CODE,ACCOUNT_NAME,AMOUNT_IDR');
      for (const a of incomeAccounts) {
        const bal = accountBalanceMinorFiltered(
          a.id,
          ledger.data,
          activeFrom || undefined,
          activeTo || undefined,
          childrenMap
        );
        const idrMinor = convertMinor(Math.abs(bal), a.currency, 'IDR', ledger.fxRate);
        csvRows.push(`INCOME,"${a.code}","${a.name}",${fromMinor('IDR', idrMinor)}`);
      }
      for (const a of expenseAccounts) {
        const bal = accountBalanceMinorFiltered(
          a.id,
          ledger.data,
          activeFrom || undefined,
          activeTo || undefined,
          childrenMap
        );
        const idrMinor = convertMinor(Math.abs(bal), a.currency, 'IDR', ledger.fxRate);
        csvRows.push(`EXPENSE,"${a.code}","${a.name}",${fromMinor('IDR', idrMinor)}`);
      }
      csvRows.push(`,,,TOTAL_NET_INCOME,${fromMinor('IDR', pnl.net)}`);
    } else if (reportTab === 'bs') {
      csvRows.push('SECTION,CODE,ACCOUNT_NAME,AMOUNT_IDR');
      for (const a of assetAccounts) {
        const bal = accountBalanceMinor(a.id, ledger.data, childrenMap);
        const idrMinor = convertMinor(Math.abs(bal), a.currency, 'IDR', ledger.fxRate);
        csvRows.push(`ASSET,"${a.code}","${a.name}",${fromMinor('IDR', idrMinor)}`);
      }
      for (const a of liabilityAccounts) {
        const bal = accountBalanceMinor(a.id, ledger.data, childrenMap);
        const idrMinor = convertMinor(Math.abs(bal), a.currency, 'IDR', ledger.fxRate);
        csvRows.push(`LIABILITY,"${a.code}","${a.name}",${fromMinor('IDR', idrMinor)}`);
      }
      for (const a of equityAccounts) {
        const bal = accountBalanceMinor(a.id, ledger.data, childrenMap);
        const idrMinor = convertMinor(Math.abs(bal), a.currency, 'IDR', ledger.fxRate);
        csvRows.push(`EQUITY,"${a.code}","${a.name}",${fromMinor('IDR', idrMinor)}`);
      }
    } else if (reportTab === 'trends') {
      csvRows.push('DATE,NET_WORTH_IDR,ASSETS_IDR,LIABILITIES_IDR,LIQUID_CASH_IDR');
      for (const pt of historicalPoints) {
        csvRows.push(
          `${pt.date},${fromMinor('IDR', pt.netWorth)},${fromMinor('IDR', pt.assets)},${fromMinor('IDR', pt.liabilities)},${fromMinor('IDR', pt.liquidCash)}`
        );
      }
    } else {
      csvRows.push('CODE,ACCOUNT_NAME,TYPE,DEBIT_IDR,CREDIT_IDR');
      for (const row of tb) {
        csvRows.push(
          `"${row.account.code}","${row.account.name}","${row.account.type}",${fromMinor('IDR', row.debit)},${fromMinor('IDR', row.credit)}`
        );
      }
      csvRows.push(
        `,,,TOTAL_DEBIT,${fromMinor('IDR', totals.debit)},TOTAL_CREDIT,${fromMinor('IDR', totals.credit)}`
      );
    }

    const csvContent = csvRows.join('\n');
    const defaultFilename = `finnca_${reportTab}_${todayString()}.csv`;

    try {
      // 1. Try native desktop save dialog (Tauri)
      const selectedPath = await save({
        defaultPath: defaultFilename,
        filters: [{ name: 'CSV Spreadsheet', extensions: ['csv'] }],
      });

      if (selectedPath) {
        await invoke('export_text_file', {
          path: selectedPath,
          contents: csvContent,
        });
        notifStore.addNotification({
          type: 'LEDGER_INTEGRITY',
          priority: 'low',
          title: i18n.t.exportSuccess,
          message: selectedPath,
        });
      }
    } catch {
      // 2. Fallback to browser blob download if running in standard browser/dev
      const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.setAttribute('href', url);
      link.setAttribute('download', defaultFilename);
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    }
  }

  async function handleExport(format: 'pdf' | 'csv', rangeFrom: string, rangeTo: string) {
    if (format === 'csv') {
      await exportCSV(rangeFrom, rangeTo);
    } else {
      await exportPDF(rangeFrom, rangeTo);
    }
  }

  import { store as appStore } from '$lib/stores/app-store.svelte';
  import { APP_NAME } from '$lib/types';

  async function exportPDF(rangeFrom: string, rangeTo: string) {
    if (!ledger.data) return;
    const { exportReportPDF } = await import('$lib/accounting/reports/export');
    const vaultName = appStore.appState?.vault_name || APP_NAME;
    const periodLabel = rangeFrom && rangeTo ? `${rangeFrom} → ${rangeTo}` : i18n.t.allTime;
    await exportReportPDF({
      vaultName,
      reportTitle: currentTabLabel,
      period: periodLabel,
      tab: reportTab,
      vault: ledger.data,
      from: rangeFrom,
      to: rangeTo,
      pnl,
      bs,
      tb,
      totals,
      incomeAccounts,
      expenseAccounts,
      assetAccounts,
      liabilityAccounts,
      equityAccounts,
      childrenMap,
      fxRate: ledger.fxRate,
    });
    notifStore.addNotification({
      type: 'LEDGER_INTEGRITY',
      priority: 'low',
      title: i18n.t.exportSuccess,
      message: `PDF — ${currentTabLabel}`,
    });
  }
</script>

<PageLayout crumb={i18n.t.report} crumbHref="/app/reports" title={currentTabLabel}>
  {#snippet actions()}
    <Button variant="ghost" onclick={() => (exportOpen = true)}>
      {i18n.t.exportCsv}
    </Button>
  {/snippet}

  <ExportOverlay
    bind:open={exportOpen}
    reportTitle={currentTabLabel}
    supportsPDF={reportTab !== 'trends'}
    onExport={handleExport}
  />

  <!-- Row 1: Unified Report Tab Navigation & Period Filters -->
  <div class="border-line mb-2 flex shrink-0 items-center justify-between gap-2 border-b pb-2">
    <div class="flex min-w-0 items-center gap-1">
      <Tabs
        tabs={[
          { id: 'pnl', label: i18n.t.pnlTitle },
          { id: 'bs', label: i18n.t.balanceSheetTitle },
          { id: 'tb', label: i18n.t.trialBalanceTitle },
          { id: 'trends', label: i18n.t.trendsTitle },
          { id: 'debt', label: i18n.t.debtReportTitle },
          { id: 'cashflow', label: i18n.t.cashflowTab },
          { id: 'fx', label: i18n.t.fxRevalTab },
        ]}
        active={reportTab}
        onSelect={(id) =>
          setReportTab(id as 'pnl' | 'bs' | 'tb' | 'trends' | 'debt' | 'cashflow' | 'fx')}
      />
    </div>

    {#if reportTab === 'pnl' || reportTab === 'cashflow' || reportTab === 'fx'}
      <div class="flex shrink-0 items-center">
        <DateRangeDropdown bind:from bind:to />
      </div>
    {:else if reportTab !== 'trends'}
      <div class="flex h-6 shrink-0 items-center gap-2">
        <label for="as-of-date" class="font-proto text-text-dim text-[10px] tracking-wider uppercase">
          {i18n.t.asOfTodayLabel}:
        </label>
        <input 
          id="as-of-date"
          type="date" 
          bind:value={to} 
          class="bg-bg-input border-line text-text-base focus:border-teal h-6 rounded-none border px-2 py-0.5 text-[11px] font-mono outline-none"
        />
        {#if to && to > todayString()}
           <span class="text-teal font-proto text-[10px] font-bold animate-pulse">[FORECAST]</span>
        {/if}
      </div>
    {/if}
  </div>

  <!-- MAIN REPORT CONTENT AREA — 100% FIT SCREEN -->
  <div class="flex min-h-0 flex-1 overflow-hidden">
    <!-- 1. PROFIT & LOSS TAB -->
    {#if reportTab === 'pnl'}
      <ProfitLoss {from} {to} />
      <!-- 2. BALANCE SHEET TAB -->
    {:else if reportTab === 'bs'}
      <BalanceSheet asOf={to} />
      <!-- 3. TRIAL BALANCE TAB -->
    {:else if reportTab === 'tb'}
      <TrialBalance asOf={to} />
      <!-- 4. TRENDS TAB -->
    {:else if reportTab === 'trends'}
      <Trends
        {bs}
        monthlyExpense={pnl.expense > 0 ? pnl.expense : 1}
        {childrenMap}
        bind:trendsPeriod
        bind:trendsMetric
        {trendsDateRange}
        {historicalPoints}
      />
      <!-- 5. DEBT REPORT -->
    {:else if reportTab === 'debt'}
      <DebtReport asOf={to} />
      <!-- 6. CASHFLOW SANKEY -->
    {:else if reportTab === 'cashflow'}
      <Cashflow {from} {to} />
      <!-- 7. FX REVALUATION -->
    {:else if reportTab === 'fx'}
      <FxReport />
    {/if}
  </div>
</PageLayout>

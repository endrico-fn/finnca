<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity';
  import { ModalShell, Button, Badge, EmptyState } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import type { MatchStatementOutput, StatementRow } from '../state/reconcile.svelte';
  import { reconcileState } from '../state/reconcile.svelte';
  import { detectBankAdjustment } from '../state/statementParser';
  import type { Account } from '$lib/core/ipc/bindings';
  import ReconcileRulesModal from './ReconcileRulesModal.svelte';

  let {
    open = $bindable(false),
    matchResult,
    currency = 'IDR',
    accounts = [],
    onApply,
    onClose,
    onQuickAdd,
    onQuickBankFee,
  }: {
    open: boolean;
    matchResult: MatchStatementOutput | null;
    currency?: string;
    accounts?: Account[];
    onApply: (selectedPostingIds: string[]) => Promise<void>;
    onClose: () => void;
    onQuickAdd: (row: StatementRow, suggestedAccountId?: string, overrideDescription?: string) => void;
    onQuickBankFee?: (row: StatementRow, adjustmentType: 'FEE' | 'INTEREST') => void;
  } = $props();

  let activeTab = $state<'matched' | 'unmatched'>('matched');
  let selectedPostingIds = new SvelteSet<string>();
  let showRulesModal = $state(false);
  let ruleModalPattern = $state('');
  let ruleModalMatchType = $state('ANY');

  function openCreateRuleForRow(row: StatementRow) {
    ruleModalPattern = row.description || '';
    ruleModalMatchType = row.amount > 0 ? 'INFLOW' : 'OUTFLOW';
    showRulesModal = true;
  }

  async function handleRuleCreated() {
    if (matchResult && matchResult.unmatched_statement_rows.length > 0) {
      matchResult.unmatched_statement_rows = await reconcileState.evaluateRules(
        matchResult.unmatched_statement_rows
      );
    }
  }

  $effect(() => {
    if (matchResult && open) {
      selectedPostingIds.clear();
      for (const m of matchResult.matches) {
        selectedPostingIds.add(m.posting_id);
      }
      activeTab = matchResult.matches.length > 0 ? 'matched' : 'unmatched';
    }
  });

  function togglePosting(id: string) {
    if (selectedPostingIds.has(id)) {
      selectedPostingIds.delete(id);
    } else {
      selectedPostingIds.add(id);
    }
  }

  function selectAll() {
    if (!matchResult) return;
    for (const m of matchResult.matches) {
      selectedPostingIds.add(m.posting_id);
    }
  }

  function deselectAll() {
    selectedPostingIds.clear();
  }

  let applying = $state(false);

  async function handleApply() {
    applying = true;
    try {
      await onApply(Array.from(selectedPostingIds));
      open = false;
    } finally {
      applying = false;
    }
  }
</script>

<ModalShell bind:open title={i18n.t.reconcileReviewTitle} size="wide" tone="teal" {onClose}>
  <div class="flex flex-col gap-3">
    <!-- Tab Navigation -->
    <div class="border-line bg-bg-card font-proto text-smaller flex items-center gap-2 border p-1">
      <button
        type="button"
        class="px-3 py-1.5 font-bold tracking-wider transition-colors {activeTab === 'matched'
          ? 'bg-teal text-bg-app'
          : 'text-text-muted hover:text-text-base'}"
        onclick={() => (activeTab = 'matched')}
      >
        {i18n.t.reconcileTabMatched} ({matchResult?.matches.length ?? 0})
      </button>
      <button
        type="button"
        class="px-3 py-1.5 font-bold tracking-wider transition-colors {activeTab === 'unmatched'
          ? 'bg-teal text-bg-app'
          : 'text-text-muted hover:text-text-base'}"
        onclick={() => (activeTab = 'unmatched')}
      >
        {i18n.t.reconcileTabUnmatchedStmt} ({matchResult?.unmatched_statement_rows.length ?? 0})
      </button>

      <button
        type="button"
        class="font-proto text-smaller text-text-dim hover:text-teal ml-auto px-2 py-1 transition-colors"
        onclick={() => {
          ruleModalPattern = '';
          ruleModalMatchType = 'ANY';
          showRulesModal = true;
        }}
      >
        ⚙ {i18n.t.reconcileRulesManageBtn}
      </button>
    </div>

    <!-- Tab 1: Matched Rows -->
    {#if activeTab === 'matched'}
      {#if !matchResult || matchResult.matches.length === 0}
        <div class="py-12">
          <EmptyState title={i18n.t.reconcileNoMatchesFound} hint={i18n.t.reconcileEmptyHint} />
        </div>
      {:else}
        <div class="font-proto text-smaller flex items-center justify-between">
          <span class="text-text-dim">
            {selectedPostingIds.size} / {matchResult.matches.length}
            {i18n.t.clearedStatus}
          </span>
          <div class="flex items-center gap-2">
            <button type="button" class="text-teal hover:underline" onclick={selectAll}>
              {i18n.t.reconcileSelectAll}
            </button>
            <span class="text-text-muted">|</span>
            <button type="button" class="text-text-muted hover:underline" onclick={deselectAll}>
              {i18n.t.reconcileDeselectAll}
            </button>
          </div>
        </div>

        <div class="border-line font-proto text-smaller max-h-80 overflow-y-auto border">
          <table class="sharp-table w-full">
            <thead class="bg-bg-card sticky top-0 z-10">
              <tr>
                <th class="w-10 px-2 py-1.5 text-center">✓</th>
                <th class="px-2 py-1.5 text-left">{i18n.t.reconcileColDate}</th>
                <th class="px-2 py-1.5 text-left">{i18n.t.reconcileColDesc}</th>
                <th class="px-2 py-1.5 text-right">{i18n.t.reconcileColAmount}</th>
                <th class="w-24 px-2 py-1.5 text-center">{i18n.t.reconcileConfidence}</th>
              </tr>
            </thead>
            <tbody>
              {#each matchResult.matches as m (m.posting_id)}
                {@const isChecked = selectedPostingIds.has(m.posting_id)}
                <tr
                  class="hover:bg-bg-card/40 cursor-pointer transition-colors {isChecked
                    ? 'bg-bg-card/20'
                    : 'opacity-60'}"
                  onclick={() => togglePosting(m.posting_id)}
                >
                  <td class="px-2 py-1.5 text-center">
                    <input
                      type="checkbox"
                      checked={isChecked}
                      class="accent-teal cursor-pointer rounded-none"
                      onclick={(e) => e.stopPropagation()}
                      onchange={() => togglePosting(m.posting_id)}
                    />
                  </td>
                  <td class="text-text-dim px-2 py-1.5 tabular-nums">
                    {m.days_diff === 0 ? '±0d' : `±${m.days_diff}d`}
                  </td>
                  <td class="text-text-base px-2 py-1.5">
                    <div class="truncate font-medium">{m.matched_description || '—'}</div>
                  </td>
                  <td class="px-2 py-1.5 text-right font-bold tabular-nums">
                    <span class={m.matched_amount >= 0 ? 'text-income' : 'text-expense'}>
                      {formatMinorToDisplay(m.matched_amount, currency)}
                    </span>
                  </td>
                  <td class="px-2 py-1.5 text-center">
                    <Badge
                      size="s"
                      tone={m.confidence >= 90 ? 'ok' : m.confidence >= 75 ? 'teal' : 'warn'}
                    >
                      {m.confidence}%
                    </Badge>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    {:else}
      <!-- Tab 2: Unmatched Statement Rows (in Bank, NOT in Ledger) -->
      {#if !matchResult || matchResult.unmatched_statement_rows.length === 0}
        <div class="py-12">
          <EmptyState title={i18n.t.reconcileNoUnmatchedStmt} hint={i18n.t.reconcileReadyMsg} />
        </div>
      {:else}
        <p class="text-text-dim font-aux text-small">
          {i18n.t.reconcileTabUnmatchedStmt}:
        </p>

        <div class="border-line font-proto text-smaller max-h-80 overflow-y-auto border">
          <table class="sharp-table w-full">
            <thead class="bg-bg-card sticky top-0 z-10">
              <tr>
                <th class="px-3 py-1.5 text-left">{i18n.t.reconcileColDate}</th>
                <th class="px-3 py-1.5 text-left">{i18n.t.reconcileColDesc}</th>
                <th class="px-3 py-1.5 text-right">{i18n.t.reconcileColAmount}</th>
                <th class="w-48 px-3 py-1.5 text-center">{i18n.t.auditActionCol}</th>
              </tr>
            </thead>
            <tbody>
              {#each matchResult.unmatched_statement_rows as row, idx (idx)}
                {@const adj = detectBankAdjustment(row.description ?? undefined, row.amount)}
                <tr class="hover:bg-bg-card/40 transition-colors">
                  <td class="text-text-dim px-3 py-1.5 whitespace-nowrap tabular-nums">
                    {row.date}
                  </td>
                  <td class="text-text-base px-3 py-1.5">
                    <div class="flex flex-col gap-0.5">
                      <div class="truncate font-medium">{row.description || '—'}</div>
                      {#if row.rule_match}
                        <div class="flex items-center gap-1">
                          <Badge size="s" tone="ok">
                            {i18n.t.reconcileRuleBadgeMatch}: [{row.rule_match.account_code}] {row.rule_match.account_name}
                          </Badge>
                        </div>
                      {/if}
                    </div>
                  </td>
                  <td class="px-3 py-1.5 text-right font-bold whitespace-nowrap tabular-nums">
                    <span class={row.amount >= 0 ? 'text-income' : 'text-expense'}>
                      {formatMinorToDisplay(row.amount, currency)}
                    </span>
                  </td>
                  <td class="px-3 py-1.5 text-center whitespace-nowrap">
                    <div class="flex items-center justify-center gap-1.5">
                      {#if row.rule_match}
                        <Button
                          variant="tactical"
                          size="sm"
                          class="font-proto text-smaller px-2 font-bold tracking-wider"
                          onclick={() => {
                            onQuickAdd(row, row.rule_match?.account_id, row.rule_match?.description_override ?? undefined);
                            open = false;
                          }}
                        >
                          {i18n.t.reconcileQuickAddWithRuleBtn}
                        </Button>
                      {:else if adj.isFeeOrInterest && onQuickBankFee}
                        <Button
                          variant="tactical"
                          size="sm"
                          class="font-proto text-smaller px-2 font-bold tracking-wider"
                          onclick={() => {
                            onQuickBankFee(row, adj.type || 'FEE');
                            open = false;
                          }}
                        >
                          {adj.type === 'INTEREST'
                            ? i18n.t.reconcileRecordBankInterestBtn
                            : i18n.t.reconcileRecordBankFeeBtn}
                        </Button>
                      {/if}
                      <Button
                        variant={row.rule_match ? 'ghost' : 'primary'}
                        size="sm"
                        class="font-proto text-smaller px-2 font-bold tracking-wider"
                        onclick={() => {
                          onQuickAdd(row);
                          open = false;
                        }}
                      >
                        {i18n.t.reconcileQuickAddBtn}
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        class="font-proto text-smaller px-1.5 font-bold tracking-wider"
                        title={i18n.t.reconcileRuleSaveRuleBtn}
                        onclick={() => openCreateRuleForRow(row)}
                      >
                        {i18n.t.reconcileRuleSaveRuleBtn}
                      </Button>
                    </div>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    {/if}

    <!-- Modal Footer Actions -->
    <div
      class="border-line font-proto text-smaller flex items-center justify-between border-t pt-3"
    >
      <Button variant="ghost" size="sm" onclick={() => (open = false)}>
        {i18n.t.cancelBtn}
      </Button>

      {#if activeTab === 'matched' && (matchResult?.matches.length ?? 0) > 0}
        <Button
          variant="tactical"
          size="sm"
          disabled={selectedPostingIds.size === 0 || applying}
          onclick={handleApply}
        >
          {i18n.t.reconcileApplyMatchesBtn.replace('{count}', String(selectedPostingIds.size))}
        </Button>
      {/if}
    </div>
  </div>
</ModalShell>

<ReconcileRulesModal
  bind:open={showRulesModal}
  {accounts}
  initialPattern={ruleModalPattern}
  initialMatchType={ruleModalMatchType}
  onRuleCreated={handleRuleCreated}
/>

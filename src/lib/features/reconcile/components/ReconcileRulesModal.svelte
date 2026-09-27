<script lang="ts">
  import { ModalShell, Button, Badge, EmptyState } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import { reconcileState } from '../state/reconcile.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import type { Account } from '$lib/core/ipc/bindings';

  let {
    open = $bindable(false),
    accounts = [],
    initialPattern = '',
    initialMatchType = 'ANY',
    onRuleCreated,
    onClose,
  }: {
    open: boolean;
    accounts: Account[];
    initialPattern?: string;
    initialMatchType?: string;
    onRuleCreated?: () => void;
    onClose?: () => void;
  } = $props();

  let pattern = $state('');
  let isRegex = $state(false);
  let matchType = $state('ANY');
  let selectedAccountId = $state('');
  let priority = $state(0);
  let descriptionOverride = $state('');
  let saving = $state(false);
  let deletingId = $state<string | null>(null);

  $effect(() => {
    if (open) {
      pattern = initialPattern || '';
      matchType = initialMatchType || 'ANY';
      isRegex = false;
      descriptionOverride = '';
      priority = 0;
      reconcileState.loadRules();
    }
  });

  const postableAccounts = $derived(
    accounts.filter((a) => !a.placeholder && (a.account_type === 'EXPENSE' || a.account_type === 'INCOME' || a.account_type === 'LIABILITY' || a.account_type === 'ASSET'))
  );

  async function handleCreateRule() {
    const trimmed = pattern.trim();
    if (!trimmed) return;
    if (!selectedAccountId) return;

    saving = true;
    try {
      await reconcileState.createRule({
        pattern: trimmed,
        is_regex: isRegex,
        match_type: matchType,
        account_id: selectedAccountId,
        priority,
        description_override: descriptionOverride.trim() ? descriptionOverride.trim() : null,
      });

      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.reconcileRulesTitle,
        message: i18n.t.reconcileRuleCreatedSuccess,
      });

      pattern = '';
      descriptionOverride = '';
      priority = 0;
      onRuleCreated?.();
    } catch (e) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.reconcileRulesTitle,
        message: String(e),
      });
    } finally {
      saving = false;
    }
  }

  async function handleDeleteRule(id: string) {
    deletingId = id;
    try {
      await reconcileState.deleteRule(id);
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.reconcileRulesTitle,
        message: i18n.t.reconcileRuleDeletedSuccess,
      });
      onRuleCreated?.();
    } catch (e) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.reconcileRulesTitle,
        message: String(e),
      });
    } finally {
      deletingId = null;
    }
  }
</script>

<ModalShell
  bind:open
  title={i18n.t.reconcileRulesTitle}
  size="wide"
  tone="teal"
  onClose={() => {
    onClose?.();
    open = false;
  }}
>
  <div class="flex flex-col gap-4">
    <!-- Form to Create / Add Rule -->
    <div class="border-line bg-bg-card/40 flex flex-col gap-3 border p-3">
      <div class="font-proto text-smaller text-teal flex items-center justify-between font-bold tracking-wider">
        <span>{i18n.t.reconcileRuleCreateBtn}</span>
      </div>

      <div class="grid grid-cols-1 gap-2.5 sm:grid-cols-2">
        <!-- Pattern Input -->
        <div class="flex flex-col gap-1">
          <label for="rule-pattern" class="font-proto text-smaller text-text-dim">
            {i18n.t.reconcileRulePattern} *
          </label>
          <input
            id="rule-pattern"
            type="text"
            bind:value={pattern}
            placeholder={i18n.t.reconcileRulePatternPlaceholder}
            class="border-line bg-bg-app font-aux text-small focus:border-teal h-8 border px-2.5 outline-none transition-colors"
          />
        </div>

        <!-- Target Account Select -->
        <div class="flex flex-col gap-1">
          <label for="rule-account" class="font-proto text-smaller text-text-dim">
            {i18n.t.reconcileRuleTargetAccount} *
          </label>
          <select
            id="rule-account"
            bind:value={selectedAccountId}
            class="border-line bg-bg-app font-aux text-small focus:border-teal h-8 border px-2 outline-none transition-colors"
          >
            <option value="" disabled>{i18n.t.reconcileRuleSelectAccount}</option>
            {#each postableAccounts as acc (acc.id)}
              <option value={acc.id}>
                [{acc.code}] {acc.name} ({acc.account_type})
              </option>
            {/each}
          </select>
        </div>

        <!-- Match Type (Direction) -->
        <div class="flex flex-col gap-1">
          <label for="rule-direction" class="font-proto text-smaller text-text-dim">
            {i18n.t.reconcileRuleMatchType}
          </label>
          <select
            id="rule-direction"
            bind:value={matchType}
            class="border-line bg-bg-app font-proto text-smaller focus:border-teal h-8 border px-2 outline-none transition-colors"
          >
            <option value="ANY">{i18n.t.reconcileRuleMatchAny}</option>
            <option value="INFLOW">{i18n.t.reconcileRuleMatchInflow}</option>
            <option value="OUTFLOW">{i18n.t.reconcileRuleMatchOutflow}</option>
          </select>
        </div>

        <!-- Description Override -->
        <div class="flex flex-col gap-1">
          <label for="rule-override" class="font-proto text-smaller text-text-dim">
            {i18n.t.reconcileRuleDescOverride}
          </label>
          <input
            id="rule-override"
            type="text"
            bind:value={descriptionOverride}
            placeholder={i18n.t.reconcileRuleDescOverridePlaceholder}
            class="border-line bg-bg-app font-aux text-small focus:border-teal h-8 border px-2.5 outline-none transition-colors"
          />
        </div>
      </div>

      <!-- Checkbox & Priority & Submit -->
      <div class="flex flex-wrap items-center justify-between gap-3 pt-1">
        <label class="flex cursor-pointer items-center gap-2">
          <input
            type="checkbox"
            bind:checked={isRegex}
            class="accent-teal h-3.5 w-3.5 rounded-none"
          />
          <span class="font-proto text-smaller text-text-dim">
            {i18n.t.reconcileRuleIsRegex}
          </span>
        </label>

        <div class="flex items-center gap-2">
          <Button
            variant="tactical"
            size="sm"
            class="font-proto text-smaller h-8 px-3 font-bold tracking-wider"
            disabled={!pattern.trim() || !selectedAccountId || saving}
            onclick={handleCreateRule}
          >
            {i18n.t.saveRuleBtn}
          </Button>
        </div>
      </div>
    </div>

    <!-- Existing Rules List -->
    <div class="flex flex-col gap-2">
      <div class="font-proto text-smaller text-text-dim flex items-center justify-between">
        <span>{i18n.t.reconcileRulesTitle} ({reconcileState.rules.length})</span>
      </div>

      {#if reconcileState.rules.length === 0}
        <div class="py-6">
          <EmptyState
            title={i18n.t.reconcileRuleEmpty}
            hint={i18n.t.reconcileRuleEmptyHint}
          />
        </div>
      {:else}
        <div class="border-line font-proto text-smaller max-h-64 overflow-y-auto border">
          <table class="sharp-table w-full">
            <thead class="bg-bg-card sticky top-0 z-10">
              <tr>
                <th class="px-2.5 py-1.5 text-left">{i18n.t.reconcileRulePattern}</th>
                <th class="px-2.5 py-1.5 text-left">{i18n.t.reconcileRuleMatchType}</th>
                <th class="px-2.5 py-1.5 text-left">{i18n.t.reconcileRuleTargetAccount}</th>
                <th class="px-2.5 py-1.5 text-left">{i18n.t.reconcileRuleDescOverride}</th>
                <th class="w-16 px-2.5 py-1.5 text-center">{i18n.t.auditActionCol}</th>
              </tr>
            </thead>
            <tbody>
              {#each reconcileState.rules as r (r.id)}
                <tr class="hover:bg-bg-card/40 transition-colors">
                  <td class="text-text-base px-2.5 py-1.5">
                    <div class="flex items-center gap-1.5">
                      <span class="font-bold text-teal">{r.pattern}</span>
                      {#if r.is_regex}
                        <Badge size="s" tone="warn">REGEX</Badge>
                      {/if}
                    </div>
                  </td>
                  <td class="text-text-dim px-2.5 py-1.5 whitespace-nowrap">
                    <Badge
                      size="s"
                      tone={r.match_type === 'INFLOW' ? 'ok' : r.match_type === 'OUTFLOW' ? 'err' : 'neutral'}
                    >
                      {r.match_type}
                    </Badge>
                  </td>
                  <td class="text-text-base px-2.5 py-1.5 whitespace-nowrap">
                    <span class="text-text-dim">[{r.account_code}]</span> {r.account_name}
                  </td>
                  <td class="text-text-dim font-aux px-2.5 py-1.5">
                    {r.description_override || '—'}
                  </td>
                  <td class="px-2.5 py-1.5 text-center">
                    <button
                      type="button"
                      class="text-expense hover:text-expense/80 font-bold transition-colors disabled:opacity-50"
                      disabled={deletingId === r.id}
                      onclick={() => handleDeleteRule(r.id)}
                      title={i18n.t.deleteBtn}
                    >
                      ✕
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

    <!-- Modal Footer -->
    <div class="border-line flex justify-end border-t pt-3">
      <Button
        variant="ghost"
        size="sm"
        onclick={() => {
          onClose?.();
          open = false;
        }}
      >
        {i18n.t.close}
      </Button>
    </div>
  </div>
</ModalShell>

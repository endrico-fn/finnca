<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import {
    getAuditLogCmd,
    verifyAuditLogIntegrityCmd,
    type AuditEntry,
    type AuditIntegrityReport,
  } from '$lib/core/ipc/bindings';
  import {
    PageLayout,
    Card,
    EmptyState,
    Badge,
    Pagination,
    LoadingSpinner,
    Button,
  } from '$lib/components/ui';

  let currentPage = $state(1);
  const perPage = 25;
  let entries = $state<AuditEntry[]>([]);
  let total = $state(0);
  let loading = $state(false);
  let error = $state<string | null>(null);

  let integrityReport = $state<AuditIntegrityReport | null>(null);
  let verifyingIntegrity = $state(false);

  const totalPages = $derived(Math.max(1, Math.ceil(total / perPage)));

  async function checkIntegrity() {
    verifyingIntegrity = true;
    try {
      integrityReport = await verifyAuditLogIntegrityCmd();
    } catch {
      integrityReport = null;
    } finally {
      verifyingIntegrity = false;
    }
  }

  async function loadLogs() {
    loading = true;
    error = null;
    try {
      const res = await getAuditLogCmd(currentPage, perPage);
      entries = res.entries;
      total = Number(res.total);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    checkIntegrity();
  });

  $effect(() => {
    void currentPage;
    loadLogs();
  });

  function formatTimestamp(ts: number): string {
    const d = new Date(ts);
    return d.toLocaleString(i18n.locale === 'id' ? 'id-ID' : 'en-US', {
      year: 'numeric',
      month: 'short',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false,
    });
  }
</script>

<PageLayout title={i18n.t.auditLogTitle}>
  {#snippet actions()}
    <div class="flex items-center gap-2">
      {#if integrityReport}
        <Badge
          size="s"
          tone={integrityReport.is_valid ? 'ok' : 'err'}
          class="font-proto tracking-wider uppercase font-bold"
        >
          {integrityReport.is_valid
            ? i18n.t.auditHashChainValid.replace('{count}', String(integrityReport.total_verified))
            : i18n.t.auditHashChainTampered}
        </Badge>
      {/if}
      <Button
        variant="secondary"
        size="sm"
        loading={verifyingIntegrity}
        onclick={checkIntegrity}
        class="font-proto text-smaller h-8 px-2.5 font-bold uppercase tracking-wider"
      >
        {i18n.t.auditVerifyIntegrityBtn}
      </Button>
    </div>
  {/snippet}

  {#if error}
    <div class="badge-err text-small font-proto mb-2 p-2">{error}</div>
  {/if}

  <Card
    title={i18n.t.auditLogTitle}
    badge={i18n.t.auditAppendOnly}
    class="min-h-0 flex-1"
    padding={false}
  >
    {#snippet header()}
      <Pagination
        bind:currentPage
        {totalPages}
        totalItems={total}
        itemLabel={i18n.t.auditEntityCol}
      />
    {/snippet}

    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      {#if loading && entries.length === 0}
        <div class="flex flex-1 items-center justify-center py-12">
          <LoadingSpinner />
        </div>
      {:else if entries.length === 0}
        <div class="flex flex-1 items-center justify-center p-8">
          <EmptyState title={i18n.t.auditNoLogs} hint={i18n.t.auditLogDesc} />
        </div>
      {:else}
        <div class="font-proto text-small min-h-0 flex-1 overflow-y-auto">
          <table class="sharp-table">
            <thead class="sticky top-0 z-10">
              <tr>
                <th class="w-48 pl-3 text-left">
                  {i18n.t.auditTimeCol}
                </th>
                <th class="w-28 px-2 text-left">
                  HASH
                </th>
                <th class="w-36 px-3 text-left">
                  {i18n.t.auditActorCol}
                </th>
                <th class="w-36 px-3 text-left">
                  {i18n.t.auditActionCol}
                </th>
                <th class="px-3 text-left">
                  {i18n.t.auditEntityCol}
                </th>
              </tr>
            </thead>
            <tbody>
              {#each entries as entry (entry.id)}
                <tr>
                  <td
                    class="text-text-muted font-proto text-smaller py-1 pl-3 whitespace-nowrap tabular-nums"
                  >
                    {formatTimestamp(entry.created_at)}
                  </td>
                  <td class="px-2 py-1 whitespace-nowrap">
                    {#if entry.hash}
                      <span
                        class="font-proto text-smaller bg-bg-app border-line text-text-dim border px-1 select-all"
                        title="prev: {entry.prev_hash || 'GENESIS'}&#10;hash: {entry.hash}"
                      >
                        {entry.hash.slice(0, 8)}…
                      </span>
                    {:else}
                      <span class="text-text-dim font-proto text-smaller">—</span>
                    {/if}
                  </td>
                  <td
                    class="font-proto text-text-strong text-smaller px-3 py-1 font-medium whitespace-nowrap"
                  >
                    {entry.actor}
                  </td>
                  <td class="px-3 py-1 whitespace-nowrap">
                    <Badge size="s" tone="neutral">{entry.action}</Badge>
                  </td>
                  <td class="text-text-base px-3 py-1">
                    <Badge size="s" tone="teal">{entry.entity_type}</Badge>
                    <span class="text-text-dim text-smaller font-proto ml-1"
                      >#{entry.entity_id}</span
                    >
                    {#if entry.detail}
                      <span class="text-text-muted text-smaller font-proto ml-2"
                        >{entry.detail}</span
                      >
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  </Card>
</PageLayout>

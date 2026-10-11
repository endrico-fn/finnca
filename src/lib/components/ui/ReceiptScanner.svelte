<script lang="ts">
  import { Icon } from '$lib/components/ui';
  import { formatMinorGrouping } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';

  let isScanning = $state(false);
  let progress = $state(0);
  let scanResultText = $state('');
  let detectedAmount = $state<number | null>(null);

  let {
    onScanComplete,
    pendingFile = $bindable<File | null>(null),
    compact = false,
  }: {
    onScanComplete?: (data: { text: string; amount: number | null }) => void;
    pendingFile?: File | null;
    compact?: boolean;
  } = $props();

  $effect(() => {
    if (pendingFile) {
      const file = pendingFile;
      pendingFile = null;
      scanReceipt(file);
    }
  });

  async function handleFileSelect(e: Event) {
    const target = e.target as HTMLInputElement;
    if (!target.files || target.files.length === 0) return;
    const file = target.files[0];
    await scanReceipt(file);
  }

  async function scanReceipt(file: File) {
    isScanning = true;
    progress = 0;
    scanResultText = '';
    detectedAmount = null;

    try {
      if (
        file.type.startsWith('text/') ||
        file.name.endsWith('.txt') ||
        file.name.endsWith('.csv') ||
        file.name.endsWith('.tsv')
      ) {
        progress = 50;
        const text = await file.text();
        progress = 100;
        scanResultText = text;

        const lines = scanResultText.split('\n');
        let maxAmount = 0;

        for (const line of lines) {
          const matches = line.match(/\b\d{1,3}(?:[.,]\d{3})*(?:[.,]\d{2})?\b/g);
          if (matches) {
            for (const match of matches) {
              const cleanStr = match.replace(/[^\d]/g, '');
              const val = parseInt(cleanStr, 10);
              if (!isNaN(val) && val > maxAmount && val < 1000000000) {
                maxAmount = val;
              }
            }
          }
        }

        if (maxAmount > 0) {
          detectedAmount = maxAmount;
        }

        onScanComplete?.({
          text: scanResultText,
          amount: detectedAmount,
        });
      } else {
        notificationState.addNotification({
          type: 'LEDGER_INTEGRITY',
          priority: 'medium',
          title: i18n.t.receiptScannerTitle,
          message: i18n.t.receiptOcrFailed,
        });
      }
    } catch {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.receiptScannerTitle,
        message: i18n.t.receiptOcrFailed,
      });
    } finally {
      isScanning = false;
    }
  }
</script>

{#if !compact}
  <div
    class="border-line bg-bg-app hover:border-teal/50 hover:bg-teal/5 relative flex flex-col items-center justify-center border border-dashed p-6 text-center transition-colors"
  >
    <input
      type="file"
      accept="text/*,.txt,.csv,.tsv"
      class="absolute inset-0 h-full w-full cursor-pointer opacity-0"
      onchange={handleFileSelect}
      disabled={isScanning}
    />

    {#if isScanning}
      <div class="flex flex-col items-center gap-3">
        <div class="text-teal animate-spin">
          <Icon name="refresh" size={24} />
        </div>
        <span class="font-proto text-teal text-smaller font-bold tracking-widest uppercase">
          {i18n.t.receiptScanning.replace('{percent}', String(progress))}
        </span>
        <div class="bg-bg-card border-line h-1 w-48 border">
          <div class="bg-teal h-full transition-all" style="width: {progress}%"></div>
        </div>
      </div>
    {:else}
      <div class="text-text-muted mb-2"><Icon name="chart" size={24} /></div>
      <h3 class="font-proto text-text-strong text-medium mb-1 font-bold tracking-widest uppercase">
        {i18n.t.receiptScannerTitle}
      </h3>
      <p class="font-proto text-text-dim text-smaller max-w-xs">
        {i18n.t.receiptScannerDesc}
      </p>
    {/if}
  </div>
{/if}

{#if scanResultText && !isScanning}
  <div class="border-line bg-bg-card mt-4 border p-3">
    <div class="mb-2 flex items-center justify-between">
      <span class="font-proto text-text-muted text-smaller tracking-widest uppercase"
        >{i18n.t.receiptExtractedData}</span
      >
      {#if detectedAmount}
        <span
          class="font-proto text-teal bg-teal/10 border-teal/20 text-small border px-2 py-0.5 font-bold tabular-nums"
        >
          {i18n.t.receiptTotalDetected.replace(
            '{amount}',
            detectedAmount !== null ? formatMinorGrouping(detectedAmount, 'IDR') : ''
          )}
        </span>
      {/if}
    </div>

    <div
      class="text-text-dim bg-bg-app border-line text-smaller font-aux max-h-32 overflow-y-auto border p-2 whitespace-pre-wrap"
    >
      {scanResultText}
    </div>
  </div>
{/if}

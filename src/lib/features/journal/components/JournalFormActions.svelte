<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { Button, Icon } from '$lib/components/ui';
  import ConfirmDialog from '$lib/components/feedback/ConfirmDialog.svelte';

  let {
    saving = false,
    isEdit = false,
    isValid = true,
    showScanner = $bindable(false),
    hasDelete = false,
    onSave,
    onCancel,
    onDelete,
  }: {
    saving?: boolean;
    isEdit?: boolean;
    isValid?: boolean;
    showScanner: boolean;
    hasDelete?: boolean;
    onSave: () => void;
    onCancel: () => void;
    onDelete?: () => Promise<void> | void;
  } = $props();

  let confirmDeleteOpen = $state(false);
</script>

<div class="border-line mt-3 flex gap-2 border-t pt-3">
  <Button
    variant={showScanner ? 'primary' : 'tactical'}
    pressed={showScanner}
    onclick={() => (showScanner = !showScanner)}
    class="flex-1"
  >
    <span class="flex items-center justify-center gap-2">
      <Icon name="chart" size={12} />
      <span>{i18n.t.txScanBtn}</span>
    </span>
  </Button>
  <Button
    variant="primary"
    onclick={onSave}
    disabled={saving || !isValid}
    class="flex-1"
  >
    {saving ? i18n.t.savingBtn : isEdit ? i18n.t.saveTransaction : i18n.t.createTransaction}
  </Button>
  <Button variant="ghost" onclick={onCancel}>{i18n.t.cancelBtn}</Button>
  {#if hasDelete && onDelete}
    <Button variant="danger" onclick={() => (confirmDeleteOpen = true)}>
      {i18n.t.deleteAccountBtn}
    </Button>
  {/if}
</div>

{#if hasDelete && onDelete}
  <ConfirmDialog
    bind:open={confirmDeleteOpen}
    title={i18n.t.confirmTitle}
    message={i18n.t.confirmDeleteTxMsg}
    confirmLabel={i18n.t.confirmBtn}
    onConfirm={async () => {
      await onDelete();
    }}
  />
{/if}

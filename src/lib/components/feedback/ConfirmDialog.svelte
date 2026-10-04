<script lang="ts">
  import { ModalShell, Button } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';

  let {
    open = $bindable(false),
    title = i18n.t.confirmDeleteDefaultTitle,
    message = i18n.t.confirmDeleteDefaultMsg,
    confirmLabel = i18n.t.deleteBtn,
    cancelLabel = i18n.t.cancelBtn,
    danger = true,
    onConfirm,
    onCancel,
  }: {
    open: boolean;
    title?: string;
    message?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    danger?: boolean;
    onConfirm: () => void | Promise<void>;
    onCancel?: () => void;
  } = $props();

  let busy = $state(false);

  async function executeConfirmation() {
    busy = true;
    try {
      await onConfirm();
      open = false;
    } finally {
      busy = false;
    }
  }

  function dismissConfirmation() {
    if (busy) return;
    open = false;
    onCancel?.();
  }
</script>

<ModalShell bind:open {title} tone={danger ? 'err' : 'teal'} onClose={onCancel}>
  <p class="text-text-base text-small font-aux py-3 leading-relaxed">
    {message}
  </p>
  <div class="border-line flex justify-end gap-2 border-t pt-3">
    <Button variant="ghost" onclick={dismissConfirmation} disabled={busy}>
      {cancelLabel}
    </Button>
    <Button variant={danger ? 'danger' : 'primary'} onclick={executeConfirmation} disabled={busy}>
      {busy ? i18n.t.planProcessing : confirmLabel}
    </Button>
  </div>
</ModalShell>

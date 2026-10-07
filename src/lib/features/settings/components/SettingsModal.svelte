<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { CloseButton } from '$lib/components/ui';
  import DraggableModal from '$lib/components/layout/DraggableModal.svelte';
  import SettingsViewer from './SettingsViewer.svelte';

  let open = $state(true);
</script>

<DraggableModal
  bind:open
  title={i18n.t.settings}
  positionKey="settings"
  widthClass="w-[880px] max-w-[calc(100vw-2rem)]"
  heightClass="max-h-[90vh]"
  onClose={() => modalState.closeSettings()}
>
  <header
    class="border-line bg-bg-card flex shrink-0 cursor-grab items-center justify-between border-b-2 px-3 py-2 select-none active:cursor-grabbing"
    data-drag-handle
  >
    <div class="flex items-center gap-2.5">
      <span class="text-text-muted text-[10px] tracking-tighter select-none" aria-hidden="true"
        >⠿</span
      >
      <span class="font-proto text-text-strong text-small font-bold tracking-wider uppercase">
        {i18n.t.settings}
      </span>
    </div>
    <CloseButton
      onclick={() => {
        open = false;
        modalState.closeSettings();
      }}
    />
  </header>
  <div class="min-h-0 flex-1 overflow-y-auto p-4">
    <SettingsViewer />
  </div>
</DraggableModal>

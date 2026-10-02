<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { ModalShell } from '$lib/components/ui';
  import SettingsViewer from './SettingsViewer.svelte';
</script>

<ModalShell
  open={true}
  title={i18n.t.settings}
  tone="neutral"
  size="wide"
  onClose={() => modalState.closeSettings()}
>
  <SettingsViewer />
</ModalShell>

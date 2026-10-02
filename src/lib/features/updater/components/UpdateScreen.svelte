<script lang="ts">
  import { updaterState, type UpdaterPhase } from '$lib/core/updater/updaterState.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { appInfo } from '$lib/core/state/appInfo.svelte';
  import { Button, ModalShell } from '$lib/components/ui';
  import { lockPolicy } from '$lib/features/security/state/lockPolicy.svelte';

  let { onClose }: { onClose: () => void } = $props();

  const PHASE_LABELS: Record<UpdaterPhase, string> = $derived({
    idle: i18n.t.updaterPhaseIdle,
    checking: i18n.t.updaterPhaseChecking,
    available: i18n.t.updaterPhaseAvailable,
    downloading: i18n.t.updaterPhaseDownloading,
    verifying: i18n.t.updaterPhaseVerifying,
    ready: i18n.t.updaterPhaseReady,
    restarting: i18n.t.updaterPhaseRestarting,
    up_to_date: i18n.t.updaterPhaseUpToDate,
    error: i18n.t.updaterPhaseError,
  });

  const isSessionExpired = $derived.by(() => {
    if (!updaterState.sessionStartedAt) return false;
    const elapsed = Date.now() - updaterState.sessionStartedAt;
    const timeout = lockPolicy.getInactivityTimeoutMs();
    return Number.isFinite(timeout) && elapsed > timeout;
  });

  function formatReleaseNotes(raw: string): string {
    return raw
      .replace(/^#{1,3}\s/gm, '')
      .replace(/\*\*(.*?)\*\*/g, '$1')
      .trim();
  }

  const progressBarWidth = $derived(
    updaterState.phase === 'verifying' || updaterState.phase === 'ready'
      ? '100%'
      : `${updaterState.progressPercent}%`
  );

  const isProgressActive = $derived(
    updaterState.phase === 'downloading' || updaterState.phase === 'verifying'
  );

  const isReady = $derived(updaterState.phase === 'ready');
  const isRestarting = $derived(updaterState.phase === 'restarting');
  const isError = $derived(updaterState.phase === 'error');
</script>

<ModalShell
  open={true}
  title={i18n.t.updaterTitle}
  tone={isError ? 'err' : isReady ? 'ok' : 'teal'}
  zIndex="z-[100]"
  {onClose}
  size="wide"
>
  <div class="grid grid-cols-[1fr_auto] gap-0">
    <!-- left: status + progress -->
    <div class="border-line flex flex-col gap-4 border-r pr-5">
      <!-- Current Version -->
      <div class="text-text-dim font-proto text-[10px]">
        {i18n.t.updaterCurrentVersion}: v{appInfo.version}
      </div>

      <!-- phase indicator -->
      <div class="flex flex-col gap-1.5">
        <div class="flex items-center justify-between">
          <span class="text-text-dim font-proto text-[10px] tracking-widest">
            {i18n.t.updaterStatus}
          </span>
          {#if updaterState.latestVersion}
            <span class="text-teal font-proto text-[10px]">
              v{updaterState.latestVersion}
            </span>
          {/if}
        </div>
        <div class="border-line-subtle bg-bg-btn border px-3 py-2">
          <span
            class="font-proto text-xs tracking-wider {isError
              ? 'text-expense'
              : isReady
                ? 'text-income'
                : 'text-text-strong'}"
          >
            {PHASE_LABELS[updaterState.phase]}
          </span>
        </div>
      </div>

      <!-- progress bar -->
      {#if isProgressActive || isReady || isRestarting}
        <div class="flex flex-col gap-1.5">
          <div class="flex items-center justify-between">
            <span class="text-text-dim font-proto text-[10px] tracking-widest">
              {i18n.t.updaterProgress}
            </span>
            {#if updaterState.phase === 'downloading'}
              <span class="text-text-base font-proto text-[10px]">
                {updaterState.downloadedMb} / {updaterState.totalMb} MB · {updaterState.progressPercent}%
              </span>
            {/if}
          </div>
          <div class="bg-line h-1.5 w-full">
            <div
              class="h-full transition-all duration-300 {isReady || isRestarting
                ? 'bg-income'
                : 'bg-teal'}"
              style="width: {progressBarWidth}"
            ></div>
          </div>
        </div>
      {/if}

      <!-- error detail -->
      {#if isError && updaterState.errorMessage}
        <div
          class="font-aux border-danger-border bg-danger-bg text-expense border px-3 py-2 text-xs"
        >
          {updaterState.errorMessage}
        </div>
      {/if}

      <!-- session timeout warning -->
      {#if isSessionExpired && isReady}
        <div class="border-warning-border bg-warning-bg border px-3 py-2.5">
          <span class="text-warning font-proto text-[10px] tracking-widest">
            {i18n.t.updaterSessionExpiredWarning}
          </span>
        </div>
      {/if}

      <!-- action buttons -->
      <div class="mt-auto flex gap-2">
        {#if updaterState.phase === 'available'}
          <Button onclick={() => updaterState.downloadAndInstall()} variant="primary" size="sm">
            {i18n.t.updaterDownloadBtn}
          </Button>
          <Button onclick={onClose} variant="ghost" size="sm">
            {i18n.t.updaterLaterBtn}
          </Button>
        {:else if isReady}
          <Button onclick={() => updaterState.applyAndRelaunch()} variant="primary" size="sm">
            {isSessionExpired ? i18n.t.updaterRelockAndApplyBtn : i18n.t.updaterApplyBtn}
          </Button>
        {:else if isError}
          <Button onclick={() => updaterState.checkForUpdate()} variant="ghost" size="sm">
            {i18n.t.updaterRetryBtn}
          </Button>
          <Button onclick={onClose} variant="ghost" size="sm">
            {i18n.t.updaterCancelBtn}
          </Button>
        {:else if isRestarting}
          <span class="text-teal font-proto animate-pulse text-[10px] tracking-widest">
            {i18n.t.updaterRestartingMsg}
          </span>
        {/if}
      </div>
    </div>

    <!-- right: release notes -->
    <div class="flex w-72 flex-col gap-2 pl-5">
      <span class="text-text-dim font-proto text-[10px] tracking-widest">
        {i18n.t.updaterReleaseNotesTitle}
      </span>
      {#if updaterState.releaseDate}
        <span class="text-text-muted font-proto text-[10px]">
          {new Date(updaterState.releaseDate).toLocaleDateString()}
        </span>
      {/if}
      {#if updaterState.releaseNotes}
        <div class="font-aux text-text-base mt-1 max-h-72 overflow-y-auto text-xs leading-relaxed">
          {#each formatReleaseNotes(updaterState.releaseNotes).split('\n') as line, idx (idx)}
            {#if line.startsWith('- ') || line.startsWith('• ')}
              <p class="border-line mb-1 border-l-2 pl-2">
                {line.replace(/^[-•]\s/, '')}
              </p>
            {:else if line.trim()}
              <p class="text-text-strong font-proto mb-1.5 text-[10px] tracking-wider">
                {line}
              </p>
            {/if}
          {/each}
        </div>
      {:else}
        <span class="font-aux text-text-muted text-xs">
          {i18n.t.updaterNoReleaseNotes}
        </span>
      {/if}
    </div>
  </div>
</ModalShell>

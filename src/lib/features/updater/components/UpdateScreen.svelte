<script lang="ts">
  import { updaterState, type UpdaterPhase } from '$lib/core/updater/updaterState.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { appInfo } from '$lib/core/state/appInfo.svelte';
  import { Button } from '$lib/components/ui';
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

<div
  class="fixed inset-0 z-[100] flex items-center justify-center"
  style="background: var(--color-overlay)"
>
  <div
    class="flex w-full max-w-2xl flex-col gap-0 border"
    style="border-color: var(--color-line); background: var(--color-bg-card)"
  >
    <!-- header bar -->
    <div
      class="flex h-10 shrink-0 items-center justify-between border-b px-4"
      style="border-color: var(--color-line)"
    >
      <div class="flex items-center gap-2.5">
        <span
          class="size-2 shrink-0"
          style="background: {isError
            ? 'var(--color-expense)'
            : isReady
              ? 'var(--color-income)'
              : 'var(--color-teal)'}"
        ></span>
        <span class="font-proto text-xs tracking-widest" style="color: var(--color-text-base)">
          {i18n.t.updaterTitle}
        </span>
      </div>
      <div class="flex items-center gap-3">
        <span class="font-proto text-[10px]" style="color: var(--color-text-dim)">
          {i18n.t.updaterCurrentVersion}: v{appInfo.version}
        </span>
        {#if updaterState.phase === 'idle' || updaterState.phase === 'up_to_date' || isError}
          <button
            class="font-proto text-xs transition-colors"
            style="color: var(--color-text-dim)"
            onclick={onClose}
          >
            ✕
          </button>
        {/if}
      </div>
    </div>

    <!-- main content -->
    <div class="grid grid-cols-[1fr_auto] gap-0">
      <!-- left: status + progress -->
      <div class="flex flex-col gap-4 border-r p-5" style="border-color: var(--color-line)">
        <!-- phase indicator -->
        <div class="flex flex-col gap-1.5">
          <div class="flex items-center justify-between">
            <span
              class="font-proto text-[10px] tracking-widest"
              style="color: var(--color-text-dim)"
            >
              {i18n.t.updaterStatus}
            </span>
            {#if updaterState.latestVersion}
              <span class="font-proto text-[10px]" style="color: var(--color-teal)">
                v{updaterState.latestVersion}
              </span>
            {/if}
          </div>
          <div
            class="border px-3 py-2"
            style="border-color: var(--color-line-subtle); background: var(--color-bg-btn)"
          >
            <span
              class="font-proto text-xs tracking-wider"
              style="color: {isError
                ? 'var(--color-expense)'
                : isReady
                  ? 'var(--color-income)'
                  : 'var(--color-text-strong)'}"
            >
              {PHASE_LABELS[updaterState.phase]}
            </span>
          </div>
        </div>

        <!-- progress bar -->
        {#if isProgressActive || isReady || isRestarting}
          <div class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between">
              <span
                class="font-proto text-[10px] tracking-widest"
                style="color: var(--color-text-dim)"
              >
                {i18n.t.updaterProgress}
              </span>
              {#if updaterState.phase === 'downloading'}
                <span class="font-proto text-[10px]" style="color: var(--color-text-base)">
                  {updaterState.downloadedMb} / {updaterState.totalMb} MB · {updaterState.progressPercent}%
                </span>
              {/if}
            </div>
            <div class="h-1.5 w-full" style="background: var(--color-line)">
              <div
                class="h-full transition-all duration-300"
                style="width: {progressBarWidth}; background: {isReady || isRestarting
                  ? 'var(--color-income)'
                  : 'var(--color-teal)'}"
              ></div>
            </div>
          </div>
        {/if}

        <!-- error detail -->
        {#if isError && updaterState.errorMessage}
          <div
            class="font-aux border px-3 py-2 text-xs"
            style="border-color: var(--color-danger-border); background: var(--color-danger-bg); color: var(--color-expense)"
          >
            {updaterState.errorMessage}
          </div>
        {/if}

        <!-- session timeout warning -->
        {#if isSessionExpired && isReady}
          <div
            class="border px-3 py-2.5"
            style="border-color: var(--color-warning-border); background: var(--color-warning-bg)"
          >
            <span
              class="font-proto text-[10px] tracking-widest"
              style="color: var(--color-warning)"
            >
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
            <span
              class="font-proto animate-pulse text-[10px] tracking-widest"
              style="color: var(--color-teal)"
            >
              {i18n.t.updaterRestartingMsg}
            </span>
          {/if}
        </div>
      </div>

      <!-- right: release notes -->
      <div class="flex w-72 flex-col gap-2 p-5">
        <span class="font-proto text-[10px] tracking-widest" style="color: var(--color-text-dim)">
          {i18n.t.updaterReleaseNotesTitle}
        </span>
        {#if updaterState.releaseDate}
          <span class="font-proto text-[10px]" style="color: var(--color-text-muted)">
            {new Date(updaterState.releaseDate).toLocaleDateString()}
          </span>
        {/if}
        {#if updaterState.releaseNotes}
          <div
            class="font-aux mt-1 max-h-72 overflow-y-auto text-xs leading-relaxed"
            style="color: var(--color-text-base)"
          >
            {#each formatReleaseNotes(updaterState.releaseNotes).split('\n') as line, idx (idx)}
              {#if line.startsWith('- ') || line.startsWith('• ')}
                <p class="mb-1 pl-2" style="border-left: 2px solid var(--color-line)">
                  {line.replace(/^[-•]\s/, '')}
                </p>
              {:else if line.trim()}
                <p
                  class="font-proto mb-1.5 text-[10px] tracking-wider"
                  style="color: var(--color-text-strong)"
                >
                  {line}
                </p>
              {/if}
            {/each}
          </div>
        {:else}
          <span class="font-aux text-xs" style="color: var(--color-text-muted)">
            {i18n.t.updaterNoReleaseNotes}
          </span>
        {/if}
      </div>
    </div>
  </div>
</div>

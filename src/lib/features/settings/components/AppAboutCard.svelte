<script lang="ts">
  import { APP_NAME } from '$lib/core/types';
  import { appInfo } from '$lib/core/state/appInfo.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Badge, AppBrand, Button, Icon } from '$lib/components/ui';
  import { updaterState } from '$lib/core/updater/updaterState.svelte';
</script>

<div class="border-line mb-6 flex flex-col border-b pb-6 last:mb-0 last:border-0 last:pb-0">
  <div class="mb-4 flex items-start justify-between">
    <div class="flex flex-col gap-1">
      <h3 class="font-proto text-text-strong text-small tracking-widest uppercase">
        {i18n.t.aboutAppTitle}
      </h3>
    </div>
    <Badge size="m" tone="neutral">v{appInfo.version}</Badge>
  </div>
  <div class="flex flex-col gap-2 md:flex-row md:items-start md:justify-between">
    <div class="flex items-center gap-2">
      <AppBrand variant="hero" />
    </div>

    <div
      class="divide-line/40 grid flex-1 grid-cols-1 gap-2 divide-y sm:grid-cols-2 sm:divide-x sm:divide-y-0 md:max-w-xl"
    >
      <div class="flex flex-col gap-1 sm:pr-4">
        <div class="flex items-center justify-between">
          <span class="font-proto text-text-muted text-smaller tracking-widest uppercase"
            >{i18n.t.aboutLocalDb}</span
          >
          <Badge size="s" tone="ok">{i18n.t.aboutEngineBadge}</Badge>
        </div>
        <p class="font-aux text-text-dim text-smaller leading-relaxed">
          {i18n.t.aboutEngineDesc}
        </p>
      </div>

      <div class="flex flex-col gap-1 pt-2 sm:pt-0 sm:pl-4">
        <div class="flex items-center justify-between">
          <span class="font-proto text-text-muted text-smaller tracking-widest uppercase"
            >{i18n.t.aboutSandboxedIpc}</span
          >
          <Badge size="s" tone="teal">TAURI v2</Badge>
        </div>
        <p class="font-aux text-text-dim text-smaller leading-relaxed">
          {i18n.t.aboutSandboxedDesc}
        </p>
      </div>
    </div>
  </div>

  <div class="border-line mt-4 border-t pt-4">
    <div
      class="font-proto text-smaller text-text-muted flex flex-wrap items-center justify-between gap-2"
    >
      <div class="flex items-center gap-2">
        <span>{APP_NAME} &copy; {new Date().getFullYear()}</span>
        <span class="tracking-widest uppercase">{i18n.t.aboutLicense}: MIT</span>
      </div>

      <Button
        size="sm"
        variant="secondary"
        onclick={() => {
          updaterState.openScreen();
          updaterState.checkForUpdate();
        }}
      >
        <span class="inline-flex items-center gap-1.5 tracking-wider uppercase">
          <Icon name="refresh" size={12} />
          {i18n.t.checkForUpdates}
        </span>
      </Button>
    </div>
  </div>
</div>

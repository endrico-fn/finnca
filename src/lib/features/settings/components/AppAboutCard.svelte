<script lang="ts">
  import { APP_NAME, APP_VERSION } from '$lib/core/types';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Card, Badge, AppBrand } from '$lib/components/ui';
</script>

<Card title={i18n.t.aboutAppTitle} badge="v{APP_VERSION}">
  <div class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
    <div class="flex items-center gap-3">
      <AppBrand variant="hero" />
    </div>

    <div
      class="divide-line/40 grid flex-1 grid-cols-1 gap-2 divide-y sm:grid-cols-2 sm:divide-x sm:divide-y-0 md:max-w-xl"
    >
      <div class="flex flex-col gap-1 sm:pr-4">
        <div class="flex items-center justify-between">
          <span class="label-xs text-text-muted">{i18n.t.aboutLocalDb}</span>
          <Badge size="s" tone="ok">{i18n.t.aboutEngineBadge}</Badge>
        </div>
        <p class="font-aux text-text-dim text-smaller leading-relaxed">
          {i18n.t.aboutEngineDesc}
        </p>
      </div>

      <div class="flex flex-col gap-1 pt-2 sm:pt-0 sm:pl-4">
        <div class="flex items-center justify-between">
          <span class="label-xs text-text-muted">{i18n.t.aboutSandboxedIpc}</span>
          <Badge size="s" tone="teal">TAURI v2</Badge>
        </div>
        <p class="font-aux text-text-dim text-smaller leading-relaxed">
          {i18n.t.aboutSandboxedDesc}
        </p>
      </div>
    </div>
  </div>

  <div
    class="border-line/40 font-proto text-smaller text-text-muted mt-4 flex items-center justify-between border-t pt-2.5"
  >
    <span>{APP_NAME} &copy; {new Date().getFullYear()}</span>
    <span class="tracking-widest uppercase">{i18n.t.aboutLicense}: MIT</span>
  </div>
</Card>

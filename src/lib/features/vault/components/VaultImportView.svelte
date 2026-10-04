<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import {
    importVault,
    inspectVaultFolder,
    type VaultInspectionResult,
  } from '$lib/core/ipc/bindings';
  import { extractErrorMessage } from '$lib/core/ipc/errors';
  import { pickDirectory } from '$lib/core/dialog';
  import { i18n } from '$lib/core/i18n.svelte';
  import { onMount } from 'svelte';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { Button, Icon } from '$lib/components/ui';
  import { session } from '$lib/core/state/session.svelte';
  import { rememberVault } from '$lib/features/vault/state/vaultList.svelte';
  import type { AppStateView } from '$lib/core/types';

  let {
    initialPath = '',
    onSuccess,
    onCancel,
  }: {
    initialPath?: string;
    onSuccess?: (st: AppStateView) => void;
    onCancel: () => void;
  } = $props();

  let importPath = $state('');
  let importPass = $state('');
  let importInspection = $state<VaultInspectionResult | null>(null);
  let busy = $state(false);
  let error = $state('');
  let isCapsLock = $state(false);

  async function inspectPath(path: string) {
    if (!path) return;
    try {
      importInspection = await inspectVaultFolder(path);
    } catch (e) {
      console.error('Failed to inspect vault directory:', e);
    }
  }

  $effect(() => {
    const target = initialPath || session.pendingImportPath || '';
    if (target && !importPath) {
      importPath = target;
      void inspectPath(target);
    }
  });

  onMount(() => {
    const target = initialPath || session.pendingImportPath || '';
    if (target) {
      importPath = target;
      void inspectPath(target);
    }

    const unsub = eventBus.on('vault:import_file', ({ path }) => {
      importPath = path;
      error = '';
      void inspectPath(path);
    });

    return () => {
      unsub();
    };
  });

  function checkCapsLock(e: KeyboardEvent) {
    if (typeof e.getModifierState === 'function') {
      isCapsLock = e.getModifierState('CapsLock');
    }
  }

  async function pickImportFolder() {
    const dir = await pickDirectory();
    if (dir) {
      importPath = dir;
      error = '';
      await inspectPath(dir);
    }
  }

  async function executeVaultImport() {
    error = '';
    if (!importPath) {
      error = i18n.t.selectVaultFolderRequired;
      return;
    }
    if (importInspection && !importInspection.is_valid) {
      error = importInspection.message;
      return;
    }
    if (importPass.length < 8) {
      error = i18n.t.passwordMin8;
      return;
    }
    busy = true;
    try {
      const st = await importVault(importPath, importPass);
      session.raw = st;
      void session.clearPendingImport();
      if (st.vault_name) {
        const name = st.vault_name;
        const actualPath = st.vault_path ?? importPath;
        rememberVault({
          id: `${name}@${actualPath}`,
          name,
          path: actualPath,
          username: st.username ?? '',
        });
      }
      if (onSuccess) {
        onSuccess(st);
      } else {
        goto(resolve('/app'));
      }
    } catch (e) {
      error = extractErrorMessage(e);
    } finally {
      busy = false;
    }
  }

  function handleCancel() {
    void session.clearPendingImport();
    onCancel();
  }
</script>

<div class="anim-fade-fast flex flex-1 flex-col justify-between">
  <div class="space-y-4">
    <div class="border-line border-b pb-2">
      <h2 class="text-text-strong font-proto text-medium font-medium tracking-wide uppercase">
        {i18n.t.importVaultTitle}
      </h2>
    </div>

    <div>
      <span class="label-xs text-text-base font-proto mb-1.5 block">
        {i18n.t.oldVaultFolder}
      </span>
      <button
        type="button"
        onclick={pickImportFolder}
        class="sharp-input hover:border-teal/50 text-small flex h-8 w-full cursor-pointer items-center justify-between px-3 transition-colors"
      >
        <span class="truncate {importPath ? 'text-text-strong' : 'text-text-dim'}">
          {importPath || i18n.t.chooseDirectory}
        </span>
        <span class="text-teal ml-2 inline-flex shrink-0">
          <Icon name="folder" size={13} />
        </span>
      </button>
    </div>

    {#if importInspection}
      {#if importInspection.status === 'valid_sqlite'}
        <div
          class="badge-ok font-proto text-smaller border-line flex items-center justify-between border px-3 py-2"
        >
          <span class="inline-flex items-center gap-1.5">
            <Icon name="check" size={11} />
            <span>{i18n.t.vaultDetectedSqlite}</span>
          </span>
          {#if importInspection.vault_name}
            <span class="text-text-strong truncate font-medium">
              {importInspection.vault_name}
              {#if importInspection.username}
                • {importInspection.username}
              {/if}
            </span>
          {/if}
        </div>
      {:else if importInspection.status === 'valid_legacy'}
        <div
          class="border-teal/40 bg-teal/10 text-teal font-proto text-smaller flex items-center justify-between border px-3 py-2"
        >
          <span class="inline-flex items-center gap-1.5">
            <Icon name="check" size={11} />
            <span>{i18n.t.vaultDetectedLegacy}</span>
          </span>
          {#if importInspection.vault_name}
            <span class="text-text-strong truncate font-medium">
              {importInspection.vault_name}
            </span>
          {/if}
        </div>
      {:else}
        <div
          class="badge-err font-proto text-smaller border-line flex items-center justify-between border px-3 py-2"
        >
          <span class="inline-flex items-center gap-1.5">
            <Icon name="close" size={11} />
            <span>{importInspection.message}</span>
          </span>
        </div>
      {/if}
    {/if}

    <div>
      <span class="label-xs text-text-base font-proto mb-1.5 block">
        {i18n.t.legacyVaultPassword}
      </span>
      <input
        bind:value={importPass}
        type="password"
        onkeydown={checkCapsLock}
        onkeyup={checkCapsLock}
        class="sharp-input text-small h-8 w-full px-3"
        placeholder={i18n.t.passwordPlaceholder}
      />
      {#if isCapsLock}
        <span class="text-warning font-proto text-smaller mt-1 block font-bold tracking-wider">
          {i18n.t.capsLockActive}
        </span>
      {/if}
      <p class="text-text-muted font-proto text-smaller mt-1 leading-relaxed">
        {i18n.t.legacyVaultPasswordHint}
      </p>
    </div>

    {#if error}
      <div class="badge-err font-proto text-small px-3 py-2 tracking-wide">
        {error}
      </div>
    {/if}
  </div>

  <div class="border-line mt-6 flex items-center justify-between border-t pt-4">
    <Button type="button" variant="ghost" onclick={handleCancel}>
      <span class="font-proto">{i18n.t.cancelBtn}</span>
    </Button>
    <Button
      variant="primary"
      onclick={executeVaultImport}
      disabled={busy || (importInspection != null && !importInspection.is_valid)}
    >
      <span class="font-proto text-small font-bold tracking-wider">
        {busy ? i18n.t.importingBtn : i18n.t.importVaultBtn}
      </span>
    </Button>
  </div>
</div>

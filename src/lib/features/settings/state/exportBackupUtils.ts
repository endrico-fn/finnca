import { todayString } from '$lib/core/format/date';
import { fromMinor } from '$lib/core/format/currency';
import {
  exportVaultBackupFolder,
  listAccountsCmd,
  listJournalEntriesCmd,
} from '$lib/core/ipc/bindings';
import { invokeIpc } from '$lib/core/ipc/client';
import { pickDirectory, pickSaveFile } from '$lib/core/dialog';
import { APP_NAME } from '$lib/core/types';

export async function exportVaultBackupJson(currentVault: string | null): Promise<string | null> {
  const dest = await pickSaveFile(
    `${APP_NAME.toLowerCase()}-vault-backup-${todayString()}.json`,
    [{ name: 'JSON Backup', extensions: ['json'] }]
  );
  if (!dest) return null;

  const [accountsView, entries] = await Promise.all([
    listAccountsCmd().catch(() => []),
    listJournalEntriesCmd().catch(() => []),
  ]);

  const backupData = {
    exportedAt: new Date().toISOString(),
    vault: currentVault,
    accounts: accountsView.map((v) => v.account),
    entries,
  };
  const jsonStr = JSON.stringify(backupData, null, 2);
  await invokeIpc<void>('export_text_file', { path: dest, contents: jsonStr });
  return dest;
}

export async function exportCsvTransactions(): Promise<string | null> {
  const dest = await pickSaveFile(
    `${APP_NAME.toLowerCase()}-transactions-${todayString()}.csv`,
    [{ name: 'CSV Transactions', extensions: ['csv'] }]
  );
  if (!dest) return null;

  const entries = await listJournalEntriesCmd().catch(() => []);
  const lines = ['Date,Account,Description,Amount'];
  for (const entry of entries) {
    for (const posting of entry.postings) {
      lines.push(
        `"${entry.date}","${posting.account_name}","${entry.description}",${fromMinor(entry.currency, posting.amount)}`
      );
    }
  }
  const csvContent = lines.join('\n');
  await invokeIpc<void>('export_text_file', { path: dest, contents: csvContent });
  return dest;
}

export async function exportEncryptedVault(): Promise<string | null> {
  const dest = await pickDirectory();
  if (!dest) return null;
  await exportVaultBackupFolder(dest);
  return dest;
}


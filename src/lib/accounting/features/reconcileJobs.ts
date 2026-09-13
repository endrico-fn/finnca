import { jobManager } from '../../infrastructure/jobs.svelte';
import { ledger } from '../store.svelte';
import { toMinor, parseBankAmountToMinor, classifyDcType } from '../finance';
import type { Currency, Transaction } from '../types';
import Papa from 'papaparse';
import { notifStore } from '../../notifications/store.svelte';
import { i18n } from '../../i18n.svelte';

// Mendaftarkan Handler untuk 'SYNC_CSV'
jobManager.registerHandler('SYNC_CSV', async (job, updateProgress) => {
  const accountId = job.payload.accountId as string;
  const fileContent = job.payload.fileContent as string;
  const bankPreset = job.payload.bankPreset as string;
  const currency = (job.payload.currency as Currency) || 'IDR';

  updateProgress(10); // Mulai parsing

  return new Promise((resolve, reject) => {
    Papa.parse(fileContent, {
      header: true,
      skipEmptyLines: true,
      complete: async (results) => {
        try {
          updateProgress(30); // Parsing selesai

          let matchedCount = 0;
          let unmatchedCount = 0;
          const rows = results.data as Record<string, string>[];
          const totalRows = rows.length;

          const modifiedTxs = new Set<Transaction>(); // Simpan referensi TX yang berubah

          // Tarik transaksi yang belum direkonsiliasi untuk akun ini
          const unclearedTransactions = ledger.transactions.filter((t) =>
            t.splits.some((s) => s.accountId === accountId && s.reconcile !== 'y')
          );

          for (let i = 0; i < totalRows; i++) {
            const row = rows[i];

            // ... (Logika penentuan kolom CSV identik dengan sebelumnya)
            const keys = Object.keys(row);
            let dateKey = '',
              amountKey = '',
              typeKey = '',
              debitKey = '',
              creditKey = '';

            if (bankPreset === 'BCA') {
              dateKey =
                keys.find(
                  (k) => k.toLowerCase().includes('tgl') || k.toLowerCase().includes('date')
                ) || '';
              amountKey =
                keys.find(
                  (k) => k.toLowerCase().includes('mutasi') || k.toLowerCase().includes('amount')
                ) || '';
              typeKey =
                keys.find(
                  (k) => k.toLowerCase().includes('type') || k.toLowerCase().includes('cr/db')
                ) || '';
            } else if (['MANDIRI', 'BRI', 'BNI'].includes(bankPreset)) {
              dateKey =
                keys.find(
                  (k) =>
                    k.toLowerCase().includes('tanggal') ||
                    k.toLowerCase().includes('tgl') ||
                    k.toLowerCase().includes('date')
                ) || '';
              debitKey =
                keys.find(
                  (k) => k.toLowerCase().includes('debet') || k.toLowerCase().includes('debit')
                ) || '';
              creditKey =
                keys.find(
                  (k) => k.toLowerCase().includes('kredit') || k.toLowerCase().includes('credit')
                ) || '';
            }

            if (!dateKey)
              dateKey =
                keys.find(
                  (k) =>
                    k.toLowerCase().includes('date') ||
                    k.toLowerCase().includes('tanggal') ||
                    k.toLowerCase().includes('tgl')
                ) || '';
            if (!amountKey && !debitKey && !creditKey) {
              amountKey =
                keys.find(
                  (k) =>
                    k.toLowerCase().includes('amount') ||
                    k.toLowerCase().includes('nominal') ||
                    k.toLowerCase().includes('jumlah') ||
                    k.toLowerCase().includes('mutasi')
                ) || '';
              typeKey =
                keys.find(
                  (k) =>
                    k.toLowerCase().includes('type') ||
                    k.toLowerCase().includes('jenis') ||
                    k.toLowerCase().includes('status')
                ) || '';
            }

            const rowDateStr = row[dateKey];
            if (!rowDateStr) continue;

            let dateObj = new Date(rowDateStr);
            if (isNaN(dateObj.getTime())) {
              const parts = String(rowDateStr).trim().split(/[/-]/);
              if (parts.length === 3) {
                const y = parts[2].length === 2 ? `20${parts[2]}` : parts[2];
                dateObj = new Date(`${y}-${parts[1]}-${parts[0]}`);
              }
            }
            if (isNaN(dateObj.getTime())) continue;
            const time = dateObj.getTime();

            let amtRaw = 0;
            if (debitKey && row[debitKey]) {
              const v = parseBankAmountToMinor(String(row[debitKey]), currency || 'IDR');
              if (v > 0) amtRaw = -v;
            }
            if (creditKey && row[creditKey] && amtRaw === 0) {
              const v = parseBankAmountToMinor(String(row[creditKey]), currency || 'IDR');
              if (v > 0) amtRaw = v;
            }
            if (amtRaw === 0 && amountKey && row[amountKey]) {
              amtRaw = parseBankAmountToMinor(String(row[amountKey]), currency || 'IDR');
              const dc = classifyDcType(String(typeKey ? row[typeKey] : row[amountKey]));
              if (dc === 'db') amtRaw = -Math.abs(amtRaw);
              else if (dc === 'cr') amtRaw = Math.abs(amtRaw);
            }

            if (amtRaw === 0) continue;

            let matched = false;
            for (const tx of unclearedTransactions) {
              // Cari split yang belum di-cek sama sekali ('n' atau kosong)
              const splits = tx.splits.filter(
                (sp) => sp.accountId === accountId && sp.reconcile !== 'y' && sp.reconcile !== 'c'
              );
              for (const s of splits) {
                const txTime = new Date(tx.date).getTime();
                const diffDays = Math.abs((txTime - time) / 86400000);
                const minorAmtRaw = toMinor(currency || 'IDR', amtRaw);
                const isAmtMatch = Math.abs(s.amount) === Math.abs(minorAmtRaw);

                if (diffDays <= 4 && isAmtMatch) {
                  s.reconcile = 'c'; // Tandai sebagai cleared (tapi belum finished 'y')
                  modifiedTxs.add(tx);
                  matched = true;
                  matchedCount++;
                  break;
                }
              }
              if (matched) break;
            }
            if (!matched) unmatchedCount++;

            // Update progress setiap 10% row processing
            if (i % Math.ceil(totalRows / 10) === 0) {
              updateProgress(30 + Math.floor((i / totalRows) * 60));
            }
          }

          updateProgress(90); // Processing selesai, menyimpan ke db

          // Simpan permanen ke backend secara bulk
          const txArray = Array.from(modifiedTxs);
          if (txArray.length > 0) {
            await Promise.all(txArray.map((tx) => ledger.upsertTransaction(tx)));
          }

          updateProgress(100);

          notifStore.addNotification({
            type: 'LEDGER_INTEGRITY',
            title: i18n.t.reconcileCsvSyncCompleteTitle,
            message: i18n.t.reconcileCsvSyncCompleteMsg
              .replace('{matched}', String(matchedCount))
              .replace('{unmatched}', String(unmatchedCount)),
            priority: 'high',
          });

          resolve();
        } catch (error) {
          reject(error);
        }
      },
      error: (error: unknown) => reject(error),
    });
  });
});

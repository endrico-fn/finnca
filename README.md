# finnca

Aplikasi catatan keuangan pribadi. Bukan sekadar pencatat pemasukan-pengeluaran — finnca memakai pembukuan **double-entry** beneran, jadi setiap transaksi selalu seimbang antara debit dan kredit, seperti software akuntansi kantoran. Tapi dibungkus antarmuka yang ramping dan jalan sepenuhnya offline di komputermu.

## Kenapa finnca?

- **Data milikmu, titik.** Semua tersimpan lokal dan terenkripsi (AGE • X25519). Tidak ada cloud, tidak ada telemetri, tidak ada akun yang perlu didaftar.
- **Satu vault, satu folder.** Tiap vault hidup di foldernya sendiri (`vault.age` + `vault.key`), gampang di-backup atau dipindah-pindah.
- **Dua mata uang.** IDR dan USD jalan berdampingan, lengkap dengan kurs live dan laporan revaluasi FX.
- **Jujur soal angka.** Nilai uang disimpan sebagai integer minor-units, bukan float — jadi tidak ada selisih misterius Rp 0,01.

## Fitur

- **Dashboard** — total saldo, kesehatan ledger, arus kas, rasio hutang, transaksi terkini.
- **Chart of Accounts** — bagan akun bertingkat dengan kode, tipe, dan mata uang per akun.
- **Journal** — entri jurnal double-entry dengan filter, pencarian, dan status rekonsiliasi.
- **Reports** — laba rugi, neraca, neraca saldo, tren & analitik, diagram Sankey arus kas, revaluasi FX, plus simulator pelunasan hutang (avalanche vs snowball).
- **Plan & Calendar** — pelacak hutang/piutang dan cicilan dengan kalender jatuh tempo.
- **Budget** — anggaran model envelope: beri setiap rupiah pekerjaan.
- **Reconcile** — cocokkan catatan dengan mutasi bank via impor CSV (ada preset bank lokal).
- **Pelengkap** — pindai struk pakai OCR, command palette (`Ctrl+K`), notifikasi lokal, ekspor CSV/PDF/JSON.

## Cara menjalankan

Butuh [Node.js](https://nodejs.org), [pnpm](https://pnpm.io), dan [Rust](https://www.rust-lang.org) (untuk backend Tauri).

```bash
pnpm install
pnpm dev        # mode pengembangan (web)
pnpm tauri dev  # mode pengembangan (aplikasi desktop)
```

Perintah lain yang berguna:

```bash
pnpm check      # type-check Svelte + TypeScript
pnpm lint       # eslint
pnpm format     # cek format prettier
pnpm check:all  # check + lint + format + check Rust
pnpm build      # build produksi
```

## Sekilas arsitektur

```
src/                  # frontend (SvelteKit + Svelte 5 + Tailwind)
  routes/app/         # halaman: dashboard, journal, accounts, reports,
                      #   plan, budget, reconcile, setting
  lib/accounting/     # inti akuntansi murni (tanpa UI, tanpa i18n)
  lib/components/     # komponen UI + komponen domain
  lib/i18n.svelte.ts  # kamus bahasa Inggris / Indonesia
src-tauri/            # backend Rust (vault terenkripsi, file I/O atomik)
```

Aturan main yang dijaga di codebase ini: semua teks UI wajib lewat `i18n` (tidak ada string hardcoded), token desain terpusat di `app.css`, dan state penting dikendalikan backend — bukan `localStorage`.

## Status

Proyek pribadi dalam pengembangan aktif. Struktur dan API masih bisa berubah sewaktu-waktu.

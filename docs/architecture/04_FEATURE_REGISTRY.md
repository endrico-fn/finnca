# 📋 Finnca Feature Registry & Lifecycle Matrix

Dokumen ini memetakan seluruh kapabilitas sistem, mencatat status operasional, serta menegakkan batasan tegas antara **Kode Usang (Dead Code)** dan **Fitur Tertunda (Unwired Feature Stubs)** agar tidak terjadi penghapusan kode yang tidak disengaja (*architectural amnesia*).

---

## 1. Definisi Status Fitur

- `[LIVE]`: Fitur beroperasi penuh, terhubung dari UI Svelte 5 hingga Rust backend, dan terverifikasi oleh automated tests.
- `[IN-PROGRESS]`: Sedang dalam tahap penyempurnaan UI/UX atau modularisasi state.
- `[STUB/PLANNED]`: Fitur/komponen yang sengaja dipersiapkan (algoritma, tipe data, atau prototipe UI) tetapi belum tersambung secara utuh ke alur kerja pengguna. **Dilarang dihapus tanpa persetujuan arsitektur.**
- `[PURGED]`: Kode peninggalan masa lalu yang telah dimusnahkan secara aman dan terdokumentasi di ADR.

---

## 2. Matriks Inventaris Fitur

| Domain | Fitur / Komponen | Status | Backend Engine | Frontend Interface | Catatan Arsitektural |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **Vault** | Create & Register Vault | `[LIVE]` | `vault::manager` | `RegisterForm.svelte` | Argon2id + envelope master key |
| **Vault** | Login & Unlock Vault | `[LIVE]` | `auth::login` | `LoginForm.svelte` | SQLCipher key unwrapping |
| **Vault** | Multi-Vault Switcher | `[LIVE]` | `vault::manager` | `VaultPicker.svelte` | Format path `finnca-<nama>` |
| **Vault** | Delete Vault with Password | `[LIVE]` | `vault::manager` | `ConfirmDialog.svelte` | Wajib verifikasi password sebelum unlink |
| **Vault** | Legacy Migration (Age -> SQLite) | `[LIVE]` | `vault::legacy_migration` | Otomatis saat unlock | Verifikasi checksum & oracle saldo |
| **Vault** | Automated Backup / Export Zip | `[STUB/PLANNED]` | `export_text_file` siap | UI belum dipasang | Ekspor arsip terenkripsi untuk backup |
| **Ledger** | Double-Entry Transaction Posting | `[LIVE]` | `ledger::service` | `JournalEntryForm.svelte` | Invarian sum(dr) == sum(cr) dikunci di Rust |
| **Ledger** | Multi-Split Splits Table | `[LIVE]` | `ledger::service` | `JournalSplitsTable.svelte` | Multi akun debit & kredit |
| **Ledger** | Simple Fast Transfer | `[LIVE]` | `ledger::service` | `JournalSimpleTransfer.svelte`| Form transfer cepat 2 baris |
| **Ledger** | Receipt Scanner (Struk Belanja) | `[STUB/PLANNED]` | Belum ada | `ReceiptScanner.svelte` | UI widget ada, parser OCR direncanakan |
| **Accounts** | Flat Path Chart of Accounts | `[LIVE]` | `accounts::service` | `AccountList` / Flat Row | Tampilan `Root > Sub > Leaf`, bukan deep tree |
| **Accounts** | Balance Rollup Calculation | `[LIVE]` | `accounts::service` | `AccountDetail.svelte` | Rollup rekursif aman siklus di Rust |
| **Security** | Auto-Lock Countdown Timer | `[LIVE]` | `security::activity_tracker`| `security.svelte.ts` | Reactive countdown dari event Rust |
| **Security** | Window Close / Blur Lock Policy | `[LIVE]` | `security::lock_policy` | `AutoLockSettings.svelte` | Pilihan: on-close, on-timeout |
| **Budget** | Monthly Budget Allocation | `[LIVE]` | `budget::service` | `budget/+page.svelte` | Perbandingan budget vs realisasi |
| **Plan** | Installment & Payoff Planner | `[LIVE]` | `plan::service` | `PlansOverview.svelte` | Simulasi amortisasi utang |
| **Plan** | Calendar Agenda Financial View | `[LIVE]` | `plan::service` | `CalendarAgenda.svelte` | Kalender jatuh tempo pembayaran |
| **Reconcile** | Statement Reader (CSV/OFX/QIF) | `[LIVE]` | `reconcile::commands` | `ReconcileWizard.svelte` | Session-gated IPC, clamped 5 MB |
| **Reconcile** | Rule-Based Transaction Matcher | `[LIVE]` | `reconcile::matcher` | `ReconcileWizard.svelte` | Skor kecocokan tanggal, nominal & deskripsi |
| **Reports** | Balance Sheet & Profit Loss | `[LIVE]` | `report::generators` | `ReportViewer.svelte` | Query SQL murni di SQLite |
| **Reports** | Native FX Revaluation Report | `[LIVE]` | `report::generators::fx` | `FxReport.svelte` | Laba/rugi selisih kurs belum terealisasi |
| **Reports** | Native Historical Trends Report | `[LIVE]` | `report::generators::trends`| `Trends.svelte` | Agregasi bulanan saldo akun di Rust |
| **Reports** | PDF & CSV Export Overlay | `[LIVE]` | `export_text_file` | `ExportOverlay.svelte` | Menggunakan pdfmake vfsFonts terisolasi |
| **Audit** | Append-Only Audit Trail | `[LIVE]` | `audit::repository` | `audit/+page.svelte` | Read-only paginated log viewer |

---

## 3. Log Pembersihan Kode Usang (*Purged Dead Code*)

Daftar kode yang telah dimusnahkan karena digantikan oleh mesin Rust baru:
1. `src/lib/accounting/core/math.ts`: Dihapus (digantikan oleh aritmatika `i128` di Rust).
2. `src/lib/accounting/ledger/transactions.ts`: Dihapus (logika double-entry dan balance validation dipindah ke `src-tauri/src/ledger/validation.rs`).
3. `src/lib/accounting/reports/*.ts`: Dihapus (digantikan oleh native report generators di Rust).
4. `src/lib/core/state/briefing.ts`: Dihapus (dead code tanpa referensi atau rencana arsitektur).
5. Monolithic `capabilities/default.json`: Dihapus (diganti dengan capability granular per-domain).

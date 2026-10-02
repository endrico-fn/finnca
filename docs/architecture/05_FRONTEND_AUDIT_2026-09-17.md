# 🔍 Frontend Design Audit — finnca (2026-09-17)

> Status dokumen: **catatan temuan + hasil re-audit**. Bukan ADR (tidak ada keputusan arsitektur baru),
> bukan perintah eksekusi. Setiap ID punya verdict re-audit: `VALID` / `FALSE-POSITIVE` /
> `NEEDS-RENDER` (butuh screenshot Tauri) / `NEEDS-VERIFY` (butuh baca Rust/test).
> Evaluasi memakai dua sumbu berbeda: **(A) validitas temuan** vs **(B) status tindak lanjut**
> (`DIPERBAIKI` = kembali ke spec vs `DITINGKATKAN` = melampaui spec lama).

- Penulis: Fyodor (audit statis read-only, tanpa render browser, tanpa `svelte-check` run)
- Scope: `src/` frontend — dashboard, accounts, journal, budget, report, plan, reconcile,
  settings, vault, shell (Sidebar/TopBar), komponen UI generik, `app.css`, i18n, `APP_NAME/VERSION`
- Bukti: `find src`, baca ~30 file `*.svelte/*.ts`, grep (`#__`, arbitrary, `rounded`, `shadow`,
  `style=`, `variant=`, `z-`, `APP_NAME`, `toLocaleString`, `parseFloat`, `OFX|QIF`, `trap`,
  hotkey), `git diff --stat`, `docs/architecture/00/01/02/04`
- Kondisi repo saat audit: `main...origin/main [ahead 9]`, 13 file dirty (lihat §8)

## Legenda prioritas

- **P0** — build-breaking / risiko finansial (float-leak, TS error, default tone berbahaya)
- **P1** — kontrak desain dilanggar (drift sistemik, duplikasi validasi, a11y palsu)
- **P2** — inkonsistensi halaman / polish (gap, responsif, hotkey, save UX)
- **P3** — aksesibilitas & perf lanjutan (kontras AA, reduced-motion, print CSS)

---

## 1. APP_NAME / APP_VERSION (sistemik)

Standar rapi: `package.json` → `vite.define.__APP_*__` → satu modul `appInfo.ts`
(`APP_NAME` mentah + `APP_SLUG` + `APP_VERSION`) → satu komponen `AppBrand`
(`showVersion`, `size`) dipakai TopBar/splash/about/settings/export.

| ID     | Bukti                                                                                | Temuan                                                                                                                                                        | Verdict                                                    |
| ------ | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| APP-01 | `src/vite-env.d.ts:5` → `declare const __APP_NAME__: strin;`                         | Typo fatal, `svelte-check` error / jatuh ke `any`                                                                                                             | VALID (P0) → DIPERBAIKI                                    |
| APP-02 | `src/app.d.ts:1-12` hanya deklarasikan `__APP_VERSION__`                             | Dua sumber deklarasi tak sinkron; kanonik SvelteKit = `app.d.ts` global, hapus duplikat `vite-env.d.ts`                                                       | VALID (P0) → DIPERBAIKI                                    |
| APP-03 | `src/lib/core/types.ts:4` → `export const version_app` vs `APP_NAME`                 | snake_case vs SCREAMING dalam satu modul; import `VaultBrandHeader:2` ikut snake                                                                              | VALID (P0) → DIPERBAIKI (`APP_VERSION` + codemod 1 import) |
| APP-04 | `types.ts:3` → `(__APP_NAME__ \|\| 'finnca').toUpperCase()`                          | Casing display dikunci di data layer; `VaultIdentityCard:12-13` harus `toLowerCase()` untuk path, judul PDF ikut UPPERCASE. Casing = urusan CSS (`uppercase`) | VALID (P1) → DITINGKATKAN (`APP_SLUG` turunan)             |
| APP-05 | Fallback `'finnca'` / `'0.1.0'` di `types.ts:3-4`                                    | Duplikat `package.json#name/version`; bump version lupa edit = drift                                                                                          | VALID (P0) → DIPERBAIKI                                    |
| APP-06 | `APP_NAME` diimport 8 tempat; versi `v{version_app}` **hanya** `VaultBrandHeader:22` | TopBar tanpa versi, splash tanpa versi, format brand beda (`tracking-[0.25em]` vs `tracking-widest`). Tidak ada `AppBrand.svelte`                             | VALID (P1) → DITINGKATKAN                                  |
| APP-07 | `ReportViewer:115` → `` `finnca_${tab}_...csv` `` literal                            | Ganti nama app = filename tetap `finnca_`                                                                                                                     | VALID (P1) → DIPERBAIKI (`APP_SLUG`)                       |

---

## 2. Hardcode ilegal (Rule 5 i18n + Rule 13 zero-drift)

Patuh: **0 `rounded-*`, 0 `shadow-*`** di seluruh `src/**/*.svelte`. Dilanggar:

| ID    | Bukti                                                                                                                                                                     | Temuan                                                                                                      | Verdict                                                    |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| HC-01 | `core/format/account.ts:7-11` fallback `, #eab308` dkk; `AccountForm:50,58-59` `DEFAULT_SWATCH_HEX` + cek `#fff/#000`                                                     | Hex di TS bypass `@theme`                                                                                   | VALID (P1) → DIPERBAIKI                                    |
| HC-02 | `report/export.ts:148,178,227,243-248` → `#333/#222/#111/#eeeeee` (7 titik)                                                                                               | Tema PDF di luar token                                                                                      | VALID (P1) → DITINGKATKAN (`pdfTheme` mapping token)       |
| HC-03 | Arbitrary (12 titik): `w-[330px]` ×2, `max-h-[90vh]`, `text-[10px]` ×4, `tracking-[0.08em]` ×2, `h-[500px]`, `min-h-[540px]`, `tracking-[0.25em]`, `top-[calc(100%+4px)]` | Langgar Rule 13 eksplisit                                                                                   | VALID (P1) → DIPERBAIKI (token)                            |
| HC-04 | `style=` inline (14 titik): Tabs dot ×2, FilterOption dot, progress width ×5, notif `border-left` ×2, swatch ×2, calendar rows, DebtReport width                          | `dot/accent` terima CSS mentah dari caller = lubang drift; progress width legit tapi tanpa komponen bersama | VALID (P1) → DITINGKATKAN (allowlist tone + `ProgressBar`) |
| HC-05 | Literal non-i18n: `ARGON2ID • SQLCIPHER` ×2, `A = L + E`, spesimen `1,234,567.89 [COMMA]` dkk                                                                             | Perlu klausul pengecualian Rule 5, bukan pelanggaran diam-diam                                              | VALID (P2-dok) → DITINGKATKAN (dok)                        |
| HC-06 | `JournalTable:91-95` `{n} SPLITS`; `commandPaletteSearch:170,218` `toLocaleString` manual; `Rp {amount}` di dalam string i18n                                             | Bypass `formatMinorToDisplay`; double-format `Rp`                                                           | VALID (P1) → DIPERBAIKI                                    |
| HC-07 | `bg-line/90`, `hover:bg-line/20`, `bg-line/20` header (Tabs, DateDropdown ×3, ReconcileTable, BudgetEnvelopeTable)                                                        | Border token dipakai sebagai background                                                                     | VALID (P1) → DIPERBAIKI                                    |

---

## 3. Inkonsistensi per page

### 3.1 Dashboard — VALID semua

- **D-01 (P1):** dua sistem KPI — `SummaryCards(BalanceCard+DebtCard+HealthCard)` custom `dl`
  vs `KpiCard` generik (dipakai Plan/Report, tidak di dashboard). `BalanceCard:26`
  `text-income` untuk total balance = semantik warna bocor.
- **D-02 (P0-finansial):** `BalanceCard` pakai `formatMinorToDisplay` (benar) vs
  `CashflowCard:70,78,90` pakai `formatIDR` (IDR-only; akun USD salah tampil).
- **D-03 (P2):** `grid-cols-12` 8+4 / 3×4 tanpa breakpoint; hanya `CashflowCard:64` responsif (`sm:`).
- **D-04 (P1):** logika bisnis di Viewer (`netWorth`, `monthlyBurn` loop, `runwayMonths`,
  `quickRatio`, klasifikasi via `name.includes('uncategorized'/'imbalance'/'unknown')`
  — rapuh terhadap ganti bahasa). Pindah ke `state/` + unit test.
- **D-05 (P2):** search scope ALL/ACCOUNT/RECENT = dua filter independen, count tak konsisten.

### 3.2 Accounts — VALID semua

- **A-01 (P1):** toggle hide `◉/◎` unicode (`AccountTable:93`), bukan `<Icon>`.
- **A-02 (P2):** 4 lapis header dalam 1 Card (title + `header` slot + FilterBar + Legend).
- **A-03 (P1):** `ACCOUNT_TYPE_BG` ganda — `state/accounts.svelte` vs `core/format/account.ts:14-20`.
- **A-04 (P2):** select-klik vs goto-dblklik tanpa affordance; NEEDS-RENDER untuk vonis mobile.

### 3.3 Journal (15 file, 2691 baris — paling parah) — VALID semua

- **J-01 (P1):** tiga implementasi transfer (`JournalSimpleTransfer` 244 + `TransferModal` 314
  - mode-simple `JournalEntryForm`) — validasi same-account/currency/fee diduplikasi
    (`EntryForm:199-209` vs `TransferModal:69-122`). Target: 1 `TransferForm` + 2 shell.
- **J-02 (P1):** empat renderer baris (`LineRow`/`SplitsTable`/`SplitsDetail`/`JournalTable`).
  Target: 1 `LedgerTable` terkonfigurasi.
- **J-03 (P1):** `JournalEntryForm` 408 baris god-form; dua sumber kebenaran
  (simple-fields vs `draft.splits`) disinkron manual.
- **J-04 (P2):** shortcut `j/k/x/e/Enter/Escape` tanpa hint UI; NEEDS-RENDER (tabrakan input?).
- **J-05 (P1):** HC-06. **J-06 (P1):** dua jalur save — `journalState.saveTransaction`
  vs `PlanViewer:227-242` `postJournalEntryCmd` manual + emit manual.

### 3.4 Budget

- **B-01 (P1):** VALID — nav bulan duplikat (`BudgetViewer:31-39` vs `PlanViewer:68-85`).
  Target: 1 `MonthPager.svelte`.
- **B-02 (P1):** VALID — `copyPreviousMonth:47-86` loop N×IPC tanpa atomisitas; gagal di tengah
  = copy sebagian. Butuh `copy_budget_month` backend (NEEDS-VERIFY sisi Rust).
- **B-03:** KOREKSI re-audit — klaim "envelope tak di-clamp" **FALSE**;
  `BudgetEnvelopeTable:32` sudah `Math.min(100, ...)`. Sisa VALID yang dipertahankan:
  4 progress-bar ad-hoc (`Envelope:52`, `PlansOverview:187`, `DebtReport:177`,
  `BudgetVsActual:92`) tanpa komponen bersama; `progress_percent` Plan/Debt datang dari
  backend tanpa clamp frontend yang terbukti (NEEDS-VERIFY Rust).

### 3.5 Report — VALID semua

- **R-01 (P1):** 7× wrapper `<div class="flex min-h-0 flex-1 flex-col w-full">` identik
  (`ReportViewer:188-225`). Target: snippet/outlet.
- **R-02 (P1):** dua gaya tab — group buttons custom `border-b-2` (`ReportTabBar:75-83`)
  - `<Tabs>`. Target: `Tabs variant="segmented"`.
- **R-03 (P1):** KPI strip diduplikasi 3× (BS/PnL/BvA). Target: `ReportKpiStrip`.
- **R-04 (P2):** `th/td.px-3` ganda di atas `.sharp-table` padding; nama akun
  `font-aux text-text-white` overemphasis vs angka.
- **R-05 (P2):** `TrendsHealthCards` vs `dashboard/HealthCard` — tabrakan nama, metrik beda.
  Target rename: `LedgerHealthCard` vs `TrendHoldingsCard`.
- **R-06 (P1):** APP-07 + HC-02 + skala overlay S/M/L tak terdokumentasi.

### 3.6 Plan

- **P-01 (P1):** VALID (J-06). Plus `postInstallment:118-157` rakit splits manual
  (tanda negatif implisit), bypass `buildSimpleSplitsWithFee`.
- **P-02 (P2):** VALID — format ringkas `toFixed` di `planCalendarUtils:23-31` duplikat
  konsep `formatMinorGrouping(compact?)` yang belum ada.
- **P-03 (P1):** VALID — tombol teal-outline custom (`CalendarAgenda:132,189`) bypass `<Button>`.

### 3.7 Reconcile

- **RC-01 (P0-finansial):** VALID — `ReconcileWizard:37-42` `parseFloat` → `toMinor`
  (float-leak); wajib `parseStringAmountToMinor` (sudah dipakai `TransferModal:81`).
- **RC-02 (P1):** VALID (HC-07, header `bg-line/20`).
- **RC-03 (P1-dok):** VALID — registry (`04_FEATURE_REGISTRY`) klaim reader CSV/OFX/QIF,
  kode hanya CSV (`open` filter `csv`, `parseCsvStatement`); grep `OFX|QIF` di `src/` = **nol**.
  Antara registry over-claim atau OFX/QIF STUB tak terdaftar — putuskan salah satu.

### 3.8 Settings

- **S-01 (P2):** VALID — param query `tab` (settings/report) vs `view` (plan).
- **S-02 (P2):** VALID — tombol Save hanya tab general; dirty-state tak terindikasi di tab lain.
- **S-03 (P1):** VALID — `h-[500px]`/`min-h-[540px]` + `grid-cols-12 col-span-5/7`
  tanpa scroll guard; NEEDS-RENDER di viewport Tauri kecil.

---

## 4. UI/UX per lapis

- **Layout/scroll (P1):** dua pola — Dashboard `overflow-hidden` + inner `overflow-y-auto`
  (benar) vs Report `overflow-y-auto` ganda (page vs inner). Kunci satu pola.
- **Surface (P1):** TopBar+Sidebar+page semua `bg-bg-app` (pemisah hanya border);
  `bg-bg-card/40`, `divide-line/40`, `border-line/60` = warna tak terdefinisi (opacity di atas token).
  `SearchBar bg-bg-app` di atas canvas `bg-bg-app` = kontras struktur nol.
- **Card (P1):** branching 4 arah (`title/header/value/badge+actions`) — kombinasi
  `title+header+actions` buang `badge/value` diam-diam; `Card class="p-3"` pemakai
  menimpa padding internal.
- **Button (P1):** kontrak ok; **89 titik** `sharp-btn/btn-*` langsung (DateDropdown,
  SelectDropdown, RegisterForm, CalendarAgenda, ModalHost:77, VaultIdentityCard).
  Tinggi liar `h-6…h-10`; API kurang (`size:xs`, `variant:outline`, `loading`).
- **Badge (P0):** default `tone='ok'` = status sukses sebagai default — berbahaya;
  ubah ke `neutral`. 10 tone implementasi vs 4 di spec (`02_COMPONENT_SPEC`) = spec basi.
- **Tabs (P1):** dua `<button>` pill/outline diduplikasi (bedakan via data);
  `style=dot` terima CSS mentah.
- **Nav/Sidebar/TopBar (P2):** industrial kuat; breadcrumb potong 2 segmen akhir;
  bell count `text-expense` = alarm palsu untuk notif netral; `resolve(... as '/app')`
  bungkam type-safety route.
- **Overlay/z-index (P1):** 5 komponen berebut `z-50` tanpa urutan
  (Modal/CommandPalette/Drawer/Toast/dropdown/VaultPicker); Toast (50) di atas Drawer (50);
  dropdown `z-50` di dalam `PageHeader z-30` = stacking-context trap.
  **Klaim focus-trap di spec (`02_COMPONENT_SPEC:101-104`) PALSU** — kode hanya
  Escape + click-outside (grep `trap` = nol; `autofocus` hanya CommandPalette + LoginForm).
- **Gap (P2):** token `--layout-gap/page-pad` ada tapi pemakaian `gap-1.5…4`,
  `p-2.5…p-7` liar. Kunci skala S/M/L + screenshot before/after.
- **Warna (P1/P3):** `text-muted #4e5454` (~2.6:1) dan `text-dim #585f5f` (~3.1:1) di atas
  `#161717` gagal WCAG AA untuk `text-smaller 10px` (dipakai 330×) — hitungan manual,
  NEEDS-RENDER dengan alat ukur. `Tabs:pill` aktif abu vs `Tabs:outline` aktif teal =
  "aktif" dua warna. `success/warning` tanpa `hover` (timpang vs `danger`).
  Teal (sistem) dipakai untuk angka (`savingsRate`, forecast) — angka wajib putih/kuat + tone tipe.
- **Animasi (P2):** 8 gerak, tanpa token durasi/easing, tanpa `prefers-reduced-motion`;
  `transition-all` (mahal di N4120) campur `transition-colors`;
  `btn:active translateY` vs `anim-modal scale` bersaing di modal.
- **Label (P2):** 4 gaya label kecil (`label-title`/`label-xs`/Tabs/FilterOption,
  `0.05em` vs `0.08em` vs `widest` vs `wider`). Unifikasi.
- **Search/Input (P2):** 4 pola cari (SearchBar scope, Select searchable, FilterMenu,
  DateDropdown) — keyboard nav tak seragam; 3 sistem hotkey (`/`, `j/k`, `Ctrl+K`)
  tanpa registry. Pertahankan yang sudah benar: date/number/select `font-proto`,
  teks bebas `font-aux`. `accent-teal` checkbox belum di-token-kan.

---

## 5. Dead vs unused vs stub (hapus vs perbaiki)

| Simbol                                            | Bukti pakai                                            | Vonis                                                                                      | Aksi                                 |
| ------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------ | ------------------------------------ |
| `BalanceCard/DebtCard/HealthCard`                 | via `SummaryCards`                                     | USED (bukan dead)                                                                          | Migrasi ke `KpiCard` bertahap        |
| `KpiCard`                                         | Plan/Report                                            | USED; props ≠ spec                                                                         | Perbaiki **spec**, bukan kode        |
| `Pagination`                                      | AccountList/JournalTotalsBar/AuditLogViewer            | USED                                                                                       | Pertahankan                          |
| `DateRangeCalendarPicker`                         | internal `DateRangeDropdown`                           | USED                                                                                       | Pertahankan                          |
| `ReceiptScanner`                                  | dynamic-import `JournalEntryForm:60`, `[STUB/PLANNED]` | STUB — larang hapus (§14)                                                                  | Tetap stub + tandai, atau OCR parser |
| `JournalQuickPresets`                             | `EntryForm:356` (mode simple baru saja)                | USED tersembunyi                                                                           | Evaluasi visibilitas                 |
| `ACCOUNT_TONE` (dihapus, `git diff badgeTone.ts`) | tak ada referensi                                      | PURGED tanpa catat registry §3                                                             | Catat atau kembalikan (amnesia)      |
| `journalEntryToTransaction` di `types.ts`         | `DashboardViewer:12`                                   | Smell: mapper IPC→UI di `core/` — pindah `core/ipc/mappers.ts`                             | Pindah                               |
| `AccountLedger*`                                  | hanya `AccountDetail`                                  | USED terisolasi — stack ledger kedua                                                       | Kandidat unifikasi jangka panjang    |
| Klaim "Transaction duplikat" (audit chat)         | `journalDraft:22,26` hanya **re-export** `core/types`  | FALSE-POSITIVE → turun ke: inkonsistensi **jalur import** (`core/types` vs `journalDraft`) | P1-ringan: unifikasi import          |

---

## 6. Redundansi → tindak lanjut

1. Transfer 3× → 1 `TransferForm` + 2 shell (DRY + validasi divergen).
2. Tabel 4× → 1 `LedgerTable` terkonfigurasi.
3. KPI 2 sistem → 1 `KpiCard` + `KpiStrip`.
4. MonthPager 2× → 1; tab 3 gaya → 1 (+`segmented`); overlay size → token S/M/L.
5. Larang `formatIDR` untuk nominal campuran (CI grep); `formatIDR` hanya label IDR-murni.
6. Larang `parseFloat→toMinor` (RC-01); wajib `parseStringAmountToMinor`.

---

## 7. Expert Svelte/TS/Tailwind/CSS

- Svelte 5 runes dipakai benar; smell: `$effect` reset `accPage` (`AccountList:46-51`)
  dan sync `targetBalanceMinor` (`ReconcileWizard:44-46`) seharusnya derived/handler;
  `untrack` (`ReportViewer:94`) tepat; `svelte:boundary` fallback `JournalTable:146-150`
  tanpa `reset`.
- TS: cast `resolve(href as '/app')` (Sidebar/PageHeader/Button) bungkam cek route;
  `BadgeTone` 10 vs spec 4.
- Tailwind v4: `@theme` benar; langgar: arbitrary, `bg-line/*`, opacity `/40-60`,
  `w-(--layout-*)` shorthand (jalan tapi rapuh → `w-[var(--layout-sidebar)]`).
- CSS: radius/shadow dikunci (taat brutalist); kurang: `prefers-reduced-motion`,
  `:focus-visible` non-button, `::selection`, print stylesheet.

---

## 8. Re-audit log (koreksi terhadap audit chat 2026-09-17)

| #   | Verifikasi                             | Hasil                                                                                                                                                                                    |
| --- | -------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T1  | `variant="pill/outline"` = bug Button? | FALSE-POSITIVE — milik `<Tabs>` (`Tabs:6,13`); `inline` milik `<Splash:15>`. Konteks DebtSimulator:50, AccountList:62, CashflowCard:54, NotificationDrawer:111 terkonfirmasi Tabs/Splash |
| T2  | "Transaction didefinisikan 2 tempat"?  | FALSE-POSITIVE — `journalDraft:22,26` re-export; jadi inkonsistensi jalur import saja                                                                                                    |
| T3  | "BudgetEnvelope tak di-clamp"?         | FALSE (klaim spesifik) — `:32` sudah `Math.min(100,…)`; dipertahankan sebagai kebutuhan `ProgressBar` bersama + clamp backend NEEDS-VERIFY                                               |
| T4  | Arbitrary 12 titik                     | VALID — daftar T5 grep                                                                                                                                                                   |
| T5  | OFX/QIF di registry                    | VALID (dok-vs-kode) — grep `OFX\|QIF` di `src/` = nol                                                                                                                                    |
| T6  | Focus-trap ModalShell                  | VALID (klaim palsu) — grep `trap` = nol                                                                                                                                                  |
| T7  | Hotkey tanpa registry                  | VALID — 3 sistem tersebar                                                                                                                                                                |
| T8  | APP-01/02/03/05                        | VALID — dibaca langsung                                                                                                                                                                  |
| T9  | `rounded/shadow` nol                   | VALID (patuh)                                                                                                                                                                            |
| T10 | 89 bypass Button                       | VALID (hitungan grep)                                                                                                                                                                    |
| T11 | Kontras muted/dim gagal AA             | NEEDS-RENDER — hitungan manual, wajib alat ukur + screenshot                                                                                                                             |

Working-tree saat audit: M ModalHost, Sidebar, TopBar, badgeTone, format/date, i18n/en,
i18n/id, prefs, types, AccountDetail, AccountsViewer, DashboardViewer, TransferModal
(13 file, +83/−105). **Refaktor dilarang di atas tree dirty — commit/stash dulu.**

---

## 9. Rencana refaktor (rekomendasi: fondasi → kontrak → halaman)

- **P0 (1 sesi):** APP-01/02/03/05, Badge default→`neutral`, RC-01, B-03-sisa.
  Hijau: `svelte-check` + `cargo test` tak tersentuh.
- **P1 (2 sesi):** `AppBrand`, `MonthPager`, `TransferForm`, `Tabs+segmented`,
  overlay S/M/L, focus-trap benar, update `02_COMPONENT_SPEC` + `04_FEATURE_REGISTRY`
  (catat `ACCOUNT_TONE`, klaim OFX/QIF, klausul HC-05).
- **P2 (per page):** Journal → Report → Dashboard → Settings (hapus arbitrary, gap/scroll,
  migrasi KpiCard/LedgerTable).
- **P3:** kontras AA, reduced-motion, `transition-colors`-only, hotkey registry, print CSS.

**Pre-mortem:** (1) tree dirty → commit/stash dulu; (2) `bindings.ts` auto-generate —
jangan edit manual; (3) ubah Badge default = 20+ snapshot visual → screenshot
before/after; (4) N4120+zram — `check`/`clippy` sekuensial, bukan paralel.
Rollback: tiap fase = 1 commit atomik.

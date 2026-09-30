# 📐 Finnca Architecture & Autonomous Agent Guidelines

Aplikasi desktop pencatatan keuangan pribadi tingkat lanjut (*advanced personal finance note & double-entry ledger*) dengan pendekatan estetika **utilitarian-brutalist industrial tech**, arsitektur privasi terisolasi (*vault-centric*), dan komputasi presisi tinggi tanpa kompromi (*zero-float invariant*).

---

## 🏛️ 1. Prinsip Fundamental & Filosofi Sistem (*System Axioms*)

1. **Estetika & Kategori Desain**:
   - **Industrial Tech & Utilitarian Brutalist**: Sudut border tajam mutlak (`rounded-none`), border fungsional (`border-line`), palet warna gelap kontras tinggi tanpa gradasi dekoratif berlebihan.
   - **Zero CSS Drift**: Dilarang keras menggunakan warna hex/rgb sembarangan (`#1a1a1a`, `rgb(...)`) atau arbitrary spacing (`px-[13px]`) di komponen Svelte. Wajib menggunakan semantic design tokens Tailwind v4 (`bg-bg-app`, `bg-bg-card`, `bg-bg-btn`, `border-line`, `text-teal`, `text-asset`, `text-expense`, dll).
2. **Tipografi & Hierarki Disiplin**:
   - `Proto Mono` (`font-proto`): Wajib untuk semua *heading*, metrik/angka, kode akun, label aksi/tombol, tab, tanggal, status badge, dan header tabel.
   - `Aux Mono` (`font-aux`): Digunakan khusus untuk teks narasi *body*, deskripsi panjang, catatan transaksi, dan field input data-entry (natural case, tanpa pemaksaan huruf kapital).
3. **Presisi Finansial (*Zero-Float Arithmetic Invariant*)**:
   - Dilarang keras menggunakan tipe data `float` / `f64` / `number` untuk perhitungan saldo atau jurnal!
   - Semua angka moneter dikelola dalam **integer minor units** (misal sen USD atau satuan IDR terkecil) menggunakan tipe `i128` / `i64` di Rust.
   - Frontend **hanya menerima hasil kalkulasi dari Rust** dan memformatnya secara display-only melalui modul tunggal terpusat: `src/lib/core/format/currency.ts`.
4. **Penyajian Struktur Akun (*Flat Path Model*)**:
   - Akun buku besar **tidak ditampilkan sebagai deep nested tree**, melainkan representasi lintasan datar (*breadcrumb path*): `RootAccount > SubAccount > LeafAccount` (misal: `Assets > Banking > BCA`).
5. **Privasi & Keamanan Vault (*Obsidian-Inspired Local Vault*)**:
   - Data finansial terisolasi penuh di folder lokal pilihan pengguna: `finnca-<nama-vault>/`.
   - **Envelope Encryption**: Kata sandi pengguna diturunkan via Argon2id menjadi KEK (*Key Encryption Key*), yang kemudian membuka DEK (*Data Encryption Key*) ChaCha20-Poly1305 untuk membuka SQLite terenkripsi SQLCipher.
   - Pergantian kata sandi hanya me-rewrap DEK tanpa perlu mengenkripsi ulang seluruh database SQLCipher.
   - **Auto-Lock Security Policy**: Mendukung penguncian otomatis berbasis timeout inaktivitas, penutupan jendela (*on-close*), atau reboot sistem.
6. **I18n Strictness & Anti-Hardcoding**:
   - Dilarang menulis teks UI mentah secara *hardcoded*. Semua string teks antarmuka wajib melalui `src/lib/core/i18n.svelte.ts` (kamus `en.ts` dan `id.ts`).
   - Identitas aplikasi seperti nama dan versi tidak boleh ditulis manual sebagai string sembarangan, melainkan diimpor dari modul `src/lib/core/types.ts` (`APP_NAME`) dan `src/lib/core/state/appInfo.svelte.ts` (`appInfo.version`).
7. **Pemusnahan Kode Usang (*Dead Code vs Planned Stubs*)**:
   - **Dead Code**: Kode usang, skrip sementara, atau logika usang yang digantikan wajib langsung dimusnahkan (*purged*), bukan ditinggalkan dalam kondisi rusak.
   - **Unwired Stubs**: Algoritma atau prototipe UI yang sengaja dipersiapkan untuk masa depan wajib dicatat di `docs/architecture/04_FEATURE_REGISTRY.md` dengan status `[STUB/PLANNED]` dan tidak boleh dihapus sembarangan.
8. **Least Privilege & Sandboxed IPC**:
   - Hak akses filesystem langsung dari frontend (`fs:default`) dicabut. Seluruh I/O berkas wajib melewati Rust IPC command yang terotentikasi session aktif, dengan batasan kuota payload (max 5 MB) dan validasi path traversal (`..`).

---

## 🛠️ 2. Aturan Rekayasa Frontend (Svelte 5 + TypeScript)

1. **Svelte 5 Runes Strictness**:
   - Runes (`$state`, `$derived`, `$props`, `$effect`) **hanya boleh digunakan di dalam file `.svelte` atau file berakhiran `.svelte.ts`**!
   - Berkas TypeScript murni (`.ts`) dilarang memanggil rune untuk mencegah `ReferenceError` pada runtime Node.js/Vitest.
2. **Modularitas Feature State**:
   - State reaktif di dalam `src/lib/features/<domain>/state/` bersifat privat untuk domain tersebut.
   - Dilarang mengimpor state internal antar-fitur secara silang. Jika fitur A membutuhkan data fitur B, integrasi wajib melewati Rust IPC atau `eventBus.svelte.ts`.
3. **Contract-First Component Interface**:
   - Komponen atomik dan molekul wajib menggunakan antarmuka standar: Svelte 5 Snippets (`children`, `header`, `actions`).
   - Varian nada warna tombol dan status dibakukan pada token: `neutral`, `ok`, `err`, `warn`, `teal`.
4. **Komentar Kode Minimalis (*Zero Inline Noise*)**:
   - Hindari komentar inline yang hanya menjelaskan apa yang sudah jelas terbaca dari kode (*self-documenting code*).
   - Komentar hanya diizinkan untuk rasionalisasi arsitektural penting atau formula matematis yang tidak intuitif.

---

## 🦀 3. Aturan Rekayasa Backend (Rust + Tauri v2)

1. **Single Source of Truth Double-Entry Ledger**:
   - Invarian fundamental `sum(Debit) == sum(Credit)` divalidasi dan dikunci mutlak di level Rust (`src-tauri/src/ledger/validation.rs`).
   - Frontend dilarang menghitung ulang saldo untuk mengambil keputusan bisnis.
2. **Type-Safe Contract Synchronization (Specta)**:
   - DTO yang diekspos ke frontend wajib mengimplementasikan `#[derive(specta::Type)]`.
   - File `src/lib/core/ipc/bindings.ts` di-generate secara otomatis oleh build pipeline; dilarang mengedit berkas ini secara manual.
3. **Penyimpanan Konfigurasi Non-Sensitif**:
   - Data registri vault global dan konfigurasi aplikasi disimpan di direktori sistem pengguna (`~/.config/finnca/finnca.json` di Linux atau `%APPDATA%\finnca\finnca.json` di Windows).
   - Konfigurasi ini tidak memuat data transaksi keuangan dan dipisahkan dari database vault terenkripsi.

---

## 📦 4. Protokol Rilis, Auto-Updater & Distribusi Multi-Platform

1. **In-App Updater Bersertifikat Kriptografi**:
   - Auto-updater terpasang via `tauri-plugin-updater` menggunakan tanda tangan Minisign.
   - Endpoint rilis membaca aset statis `latest.json` di GitHub Releases, kebal terhadap Webview CSP dan bebas dari limitasi kuota API GitHub.
   - Di level frontend, pengecekan pembaruan dijalankan secara native melalui IPC Rust di `src/lib/core/updater/updateChecker.ts`.
2. **Platform Linux**:
   - Format paket: `.AppImage` (portable), `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL/openSUSE).
   - Skrip instalasi terminal terpadu: `scripts/finnca.sh` (mendukung `install`, `upgrade`, `uninstall`, dan `status`).
   - One-liner: `curl -fsSL https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/install.sh | bash`
3. **Platform Windows**:
   - Format paket: NSIS setup (`_x64-setup.exe`) mode per-user (`currentUser`, rootless tanpa butuh hak Administrator) dan WiX installer (`.msi`).
   - Mesin otomasi terminal PowerShell industrial: `scripts/finnca.ps1` (dengan shim `scripts/install.ps1`).
   - One-liner: `irm https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/install.ps1 | iex`
4. **CI/CD Pipeline (.github/workflows/release.yml)**:
   - Memicu build otomatis saat tag versi didorong (`v*.*.*`).
   - Sinkronisasi versi otomatis dari Git Tag ke `package.json`, `tauri.conf.json`, dan `Cargo.toml`.
   - Menghasilkan biner bertanda tangan untuk Linux dan Windows secara simultan.

---

## 🗂️ 5. Peta Direktori & Batas Subsistem

```
finnca/
├── docs/                                          # Second brain & arsitektur sistem
│   ├── architecture/                              # Manifesto, design system, spesifikasi komponen
│   └── adr/                                       # Architecture Decision Records
│
├── scripts/                                       # Otomasi deployment multi-platform
│   ├── finnca.sh                                  # Engine instalasi Linux (bash)
│   ├── install.sh, upgrade.sh, uninstall.sh       # POSIX shims untuk Linux
│   ├── finnca.ps1                                 # Engine instalasi Windows (PowerShell)
│   └── install.ps1, uninstall.ps1                 # PowerShell shims untuk Windows
│
├── src/                                            # === Frontend (Svelte 5 + TypeScript) ===
│   ├── lib/
│   │   ├── core/                                  # Infrastruktur global & lintas-fitur
│   │   │   ├── ipc/                               # Client IPC, binding Specta, penanganan error
│   │   │   ├── events/                            # Event bus reaktif Svelte 5
│   │   │   ├── format/                            # Format mata uang & ekspresi matematika
│   │   │   ├── updater/                           # Pengecek pembaruan native Tauri & state updater
│   │   │   ├── state/                             # Global runes: appInfo, session, security, modal
│   │   │   └── i18n/                              # Kamus lokalisasi id.ts & en.ts
│   │   ├── components/                            # Komponen UI modular (ui/, layout/, charts/, feedback/)
│   │   └── features/                              # Fitur per-domain (vault, ledger, accounts, report, dll)
│   └── routes/                                    # SvelteKit routing tipis pemanggil fitur
│
└── src-tauri/                                      # === Backend (Rust + Tauri v2) ===
    ├── capabilities/                              # Izin IPC per domain (vault, ledger, security, settings)
    └── src/
        ├── vault/                                 # Manajemen lifecycle vault & migrasi legacy
        ├── crypto/                                # Argon2id KDF, envelope encryption, SQLCipher
        ├── auth/                                  # Logika login & unlock
        ├── security/                              # Kebijakan auto-lock & tracking aktivitas
        ├── ledger/                                # Validasi invarian double-entry & posting jurnal
        ├── accounts/                              # CRUD & rollup saldo Chart of Accounts
        ├── budget/, plan/, reconcile/, report/    # Modul bisnis keuangan terdedikasi
        ├── audit/                                 # Log jejak audit tamper-evident append-only
        └── db/                                    # Koneksi SQLite & skema migrasi bernomor
```

---

## ⚡ 6. Kriteria Kesiapan Sebelum Commit (*Quality Gates*)

Setiap agen pengembang wajib memastikan seluruh perintah ini keluar dengan status hijau (`exit code 0`) sebelum melakukan commit:

1. `pnpm check`: Pemeriksaan tipe TypeScript dan validasi Svelte 5 runes.
2. `pnpm lint`: Standarisasi ESLint tanpa peringatan atau eror.
3. `pnpm test`: Pengujian unit Vitest (seluruh tes matematika minor dan state wajib lulus).
4. `cargo check --manifest-path src-tauri/Cargo.toml`: Kompilasi backend Rust bersih tanpa eror.
5. `cargo test --manifest-path src-tauri/Cargo.toml`: Pengujian unit, golden test fixtures, dan stress test Rust lulus 100%.

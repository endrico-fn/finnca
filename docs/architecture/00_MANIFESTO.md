# 🏛️ Finnca Architecture Manifesto

Aplikasi desktop pencatatan personal note finance double-entry tingkat lanjut yang disesuaikan (*tailored utilitarian-brutalist financial desktop application*).

---

## 1. Core Pillars

### I. Zero-Float Invariant Integrity (Aritmatika Presisi Mutlak)
- **Hukum Utama:** Seluruh kalkulasi finansial di backend menggunakan bilangan bulat `i128` (integer minor units / sen / rupiah dasar) dengan pembulatan *half-up*.
- **Frontend Contract:** Frontend tidak pernah melakukan kalkulasi saldo untuk pengambilan keputusan bisnis. Frontend hanya menerima data terstruktur dan memformatnya untuk visualisasi via `src/lib/core/format/currency.ts`.
- **Double-Entry Invariant:** Setiap transaksi general journal harus memenuhi syarat mutlak:
  $$\sum \text{Debit} = \sum \text{Kredit}$$
  Validasi ini dikunci secara murni di Rust (`ledger/validation.rs`) sebelum persistensi database.

### II. Utilitarian-Brutalist & Industrial Tech UI
- **Estetika:** Monokromatik terstruktur, sudut tajam mutlak (`rounded-none`), border tegas 1px (`border-border`), data density tinggi, dan visual noise minimal.
- **Tipografi Fungsional:**
  - `Proto Mono`: Wajib untuk angka, saldo, heading, label aksi, tabs, kode akun, dan badge status.
  - `Aux Mono`: Khusus untuk teks body naratif, catatan transaksi, dan input teks data-entry natural.
- **Micro-Interactions:** Animasi ringan (durasi < 150ms), transisi instan, navigasi keyboard-first (shortcut badges).

### III. Obsidian-Style Vault Architecture & Privacy-First
- **Vault Directory:** Terisolasi dalam direktori terstruktur `finnca-<vault_name>/`.
- **Envelope Encryption:** Password pengguna diturunkan via Argon2id untuk membuka Master Key database SQLCipher. Mengubah password tidak memerlukan enkripsi ulang seluruh berkas database.
- **Auto-Lock Security:** Kunci vault otomatis berdasarkan kebijakan timeout, idle activity tracker, atau window closing.
- **Air-Gapped & Offline:** Zero analytics, zero cloud phone-home, data 100% milik pengguna lokal.

### IV. Least-Privilege & Capability Sandboxing
- **Tauri Sandbox:** Pencabutan izin monolitik (`fs:default`).
- **Granular Capabilities:** Izin dikotak-kotakkan per domain:
  - `vault.json`: Manajemen pendaftaran dan pembukaan vault.
  - `ledger.json`: Penulisan dan pembacaan jurnal & saldo akun.
  - `security.json`: Pengaturan auto-lock dan deteksi aktivitas.
  - `reports.json`: Akses baca (*read-only*) query analitik.
  - `settings.json`: Preferensi UI lokal.
- **Session-Gated File Operations:** Akses pembacaan berkas rekening koran (reconcile) wajib divalidasi dengan session vault aktif dan dibatasi secara ketat (*5 MB DoS clamp*).

### V. Strict Modular Decoupling
- **Aturan Fitur:** Direktori `src/lib/features/<feature>/state/*.svelte.ts` tidak boleh diimpor lintas-fitur.
- **Jembatan Antar-Domain:** Komunikasi lintas-domain hanya diizinkan melalui:
  1. Typed IPC Call (`core/ipc/client.ts` -> Rust).
  2. Reactive Event Bus (`core/events/eventbus.svelte.ts`).
  3. Prop injection dari *page orchestrator*.

---

## 2. Definisi Kode & Anti-Pattern

| Pola Yang Diwajibkan | Pola Yang Dilarang Keras |
| :--- | :--- |
| Integer minor units (`100000` = Rp 1.000,00) | Tipe `float` atau `number` desimal untuk saldo |
| Semantic design tokens (`bg-surface-1`, `border-border`) | Arbitrary utility classes (`bg-zinc-800`, `p-[13px]`) |
| Sudut tajam mutlak (`rounded-none`) | Sudut membulat (`rounded-md`, `rounded-xl`) |
| Validasi invarian di Rust backend | Menghitung balance posting di Svelte sebelum submit |
| Translasi via `core/i18n.svelte.ts` | Hardcoded string bahasa di komponen UI |
| Penamaan konstanta aplikasi via `APP_NAME`, `APP_VERSION` | Menulis hardcoded teks brand di template UI |

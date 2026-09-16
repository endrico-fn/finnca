# 🧩 Finnca Component Specification (Contract-First UI)

Spesifikasi kontrak standar antarmuka komponen UI di Finnca. Seluruh komponen dibangun menggunakan **Svelte 5 Runes** (`$props`, `$state`, `$derived`) dan **Snippets** (`Snippet`), serta mengadopsi prinsip sudut tajam (`rounded-none`).

---

## 1. Konvensi Standar Komponen

1. **Svelte 5 Snippets:** Menggantikan slots lama. Komponen penampung menerima `children: Snippet`, `actions?: Snippet`, `header?: Snippet`.
2. **Standardized Tones (Warna Fungsional):**
   ```ts
   type BadgeTone = 'neutral' | 'ok' | 'err' | 'warn';
   ```
   - `neutral`: Slate/gray, netral tanpa muatan emosi (info referensi, kode akun biasa).
   - `ok`: Green/teal, status aktif, reconciled, profit, balance cocok.
   - `err`: Red, unreconciled error, defisit, validasi gagal, tombol hapus.
   - `warn`: Amber/yellow, draft belum posting, jatuh tempo mendekat.
3. **Standardized Sizing:**
   ```ts
   type ComponentSize = 's' | 'm' | 'l';
   ```

---

## 2. Katalog Komponen Inti

### I. Layout & Shell Containers

#### `PageLayout.svelte`
Kontainer utama pembungkus halaman di dalam route `/app/*`.
- **Props:**
  - `class?: string`: Custom styling untuk viewport body.
  - `children: Snippet`: Konten halaman yang responsif terhadap scroll.
- **Perilaku:** Menjaga overflow scroll tetap berada di area body, tanpa menggeser Topbar atau Sidebar.

#### `PageHeader.svelte`
Header standar setiap halaman aplikasi.
- **Props:**
  - `title: string`: Judul halaman (`font-proto`, uppercase).
  - `crumb?: string`: Indikator breadcrumb induk (misal: `"DASHBOARD"`).
  - `crumbHref?: string`: Tautan navigasi breadcrumb (default: `"/app"`).
  - `actions?: Snippet`: Tombol aksi utama di sisi kanan header.

---

### II. Data Display & Containers

#### `Card.svelte`
Kontainer utilitarian serbaguna untuk membungkus grup formulir atau widget data.
- **Props:**
  - `title?: string`: Judul kartu di baris atas.
  - `count?: number | string`: Badge angka counter samping judul.
  - `description?: string`: Subtitle penjelas singkat.
  - `header?: Snippet`: Kustomisasi header lengkap.
  - `actions?: Snippet`: Tombol aksi di pojok kanan kartu.
  - `badge?: string`: Teks badge status.
  - `badgeTone?: 'neutral' | 'ok' | 'err' | 'warn'`: Warna badge.
  - `divided?: boolean`: Beri garis pemisah antar-elemen (`divide-line/40`).
  - `padding?: boolean`: Beri padding standar internal (default `true`).
  - `children: Snippet`: Konten utama kartu.

#### `KpiCard.svelte`
Kartu ringkasan metrik finansial cepat (Net Worth, Arus Kas, Total Utang).
- **Props:**
  - `title: string`: Label metrik (uppercase `font-proto`).
  - `value: string`: Nilai uang terformat (misal `Rp 15.000.000`).
  - `delta?: string`: Persentase atau nilai perbandingan periode sebelumnya.
  - `deltaPositive?: boolean`: Apakah kenaikan dianggap positif (hijau) atau negatif (merah).
  - `tone?: 'income' | 'expense' | 'asset' | 'liability' | 'teal'`.

#### `Badge.svelte`
Indikator status atomik ringkas dengan border 1px.
- **Props:**
  - `tone?: 'neutral' | 'ok' | 'err' | 'warn'` (default: `'ok'`).
  - `size?: 's' | 'm' | 'l'` (default: `'m'`).
  - `children: Snippet`: Label teks di dalam badge.

---

### III. Form Controls & Data Entry

#### `SelectDropdown.svelte` & `DateDropdown.svelte`
Input pilihan yang konsisten dengan estetika dark utilitarian, menggantikan `<select>` bawaan browser yang tidak konsisten.
- Menggunakan popup dengan border tajam `border-line` dan latar `bg-bg-btn`.
- Mendukung pencarian instan dan keyboard navigation (Up/Down/Enter/Escape).

#### `FilterMenu.svelte` & `FilterOption.svelte`
Dropdown multi-kriteria untuk tabel transaksi, pencatatan jurnal, dan chart of accounts.

#### `SearchBar.svelte`
Input pencarian real-time dengan hotkey pemicu otomatis (`/` atau `Ctrl+K`).

---

### IV. Feedback & Modals

#### `ConfirmDialog.svelte`
Modal konfirmasi destruktif (misal: Hapus Akun, Hapus Jurnal, Hapus Vault).
- Wajib meminta input password atau pengetikan teks eksplisit untuk aksi dengan risiko tinggi (seperti menghapus vault).

#### `ModalHost.svelte`
Host modal terpusat yang me-render dialog aktif yang dikelola oleh `src/lib/core/state/modal.svelte.ts`.
- Mendukung penutupan dengan tombol Escape.
- Menjaga fokus keyboard (*focus trap*) di dalam jendela modal.

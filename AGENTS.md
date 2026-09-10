# 📐 Finnca Architecture & Development Guidelines

File baru wajib masuk ke direktori yang sesuai kategori di bawah — jangan buat top-level baru di `src/lib/` tanpa alasan struktural yang jelas.

## 🏛️ Prinsip Arsitektur Utama (Non-Negotiable)

1. **Presisi Finansial (Minor-Units)**: Nilai uang WAJIB disimpan sebagai integer minor-units (mis. sen/cent). Dilarang keras menggunakan tipe data `float` atau kalkulasi desimal langsung untuk mencegah _floating-point bias_.
2. **Brutalist / Utilitarian Design System**:
   - Fungsi adalah estetika. Tanpa _drop-shadow_ berlebih, sudut lengkung tajam (0px radius), border 1px solid.
   - Tidak ada elemen dekoratif (contoh: hapus foto profil/avatar, ganti dengan tipografi murni).
3. **Tipografi & Hierarki**:
   - `Proto Mono`: Wajib digunakan untuk _heading_, deskripsi pendek, label aksi, _tabs_, numerik/angka, kode akun, dan _badge_ status.
   - `Aux Mono`: Digunakan khusus untuk teks _body_ atau penjelasan detail yang panjang.
   - Angka harus mematuhi perataan tabular (Tabular Nums) agar sejajar vertikal saat ditampilkan dalam bentuk daftar/jurnal.
4. **Keamanan & Integritas State (Backend-Driven)**:
   - Data konfigurasi dan status _registry_ Vault dikontrol penuh oleh _Backend_ (Rust).
   - Dilarang menduplikasi _state_ vital ke `localStorage` (hanya digunakan untuk preferensi non-sensitif sementara).
   - Penulisan _file_ di Rust harus menggunakan _Atomic Write_ (`sync_all()`) demi mencegah korupsi data saat sistem _crash_.
5. **i18n Strictness**: Dilarang ada _string_ teks UI yang _hardcoded_. Semua wajib melalui `i18n.svelte.ts`.

## 💻 Konvensi Koding (Code Quality)

- **Zero Inline Comment**: Tulis kode yang _self-explanatory_ lewat penamaan variabel/fungsi yang jelas. Komentar hanya diizinkan untuk trik _workaround_ rumit atau gotcha yang sangat spesifik (menjawab "mengapa", bukan "apa").
- **Komponen Reusable**: Komponen _shared_ wajib hidup di `src/lib/components`. Dilarang menduplikasi kode UI antar halaman.
- **CSS Tokens Terpusat**: Definisi spasi, warna, dan perbatasan (_border_) dipusatkan di `app.css`. Jangan timpa ulang (_override_) di level komponen kecuali mendesak.
- **Histori Tugas**: Dilarang keras menghapus daftar tugas (Task/Todo) maupun *Plan* yang sudah berstatus selesai `[x]` di dokumen `tasks/todo.md` maupun `tasks/plan.md`. Biarkan sebagai *history* proyek.

## 🛠️ Backlog Prioritas Selanjutnya

- [ ] **Skalabilitas Data**: Evaluasi strategi _lazy loading_ / paginasi untuk jurnal saat transaksi mencapai puluhan ribu entri.
- [ ] **Standardisasi UI 100%**: Refaktor lanjutan agar _button_, _tab_, warna, form _search_, dan _card_ di seluruh _routes_ konsisten mutlak dengan tema _Brutalist Terminal_.
- [ ] **Keyboard-First Navigation (Power User)**: Implementasi _shortcuts_ dasar (misal `CMD+K` untuk _command palette_ atau navigasi _Vim-like_ di tabel jurnal).
- [ ] **Testing & QA**: Rencana implementasi _unit test_ otomatis untuk memvalidasi algoritma akuntansi dan rekonsiliasi (_double-entry integrity_).

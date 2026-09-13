# 📐 Finnca Architecture & Development Guidelines

aplikasi gui pencatatan personal note finance tingkat lanjut yang di sesuaikan

## halaman:

- dashboard
- account
- journal
- budget
- report
- reconcile
- setting

memiliki konsep vault dimana data di simpan dengan privasi tambahan seperti kunci vault, login, register, multi vault

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
   - **Skala ukuran terkunci** (`@theme` di `app.css`, class `text-*`): `smaller` 10px = info/deskripsi/label/tab/filter/badge · `small` 12px = body/tabel · `medium` 14px = heading/angka figure · `large` 18px = KPI · `largest` 26px = hero. Dilarang `text-[Npx]`/`text-xs/sm/xl` baru — pakai token.
4. **Keamanan & Integritas State (Backend-Driven)**:
   - Data konfigurasi dan status _registry_ Vault dikontrol penuh oleh _Backend_ (Rust).
   - Dilarang menduplikasi _state_ vital ke `localStorage` (hanya digunakan untuk preferensi non-sensitif sementara).
   - Penulisan _file_ di Rust harus menggunakan _Atomic Write_ (`sync_all()`) demi mencegah korupsi data saat sistem _crash_.
5. **i18n Strictness**: Dilarang ada _string_ teks UI yang _hardcoded_. Semua wajib melalui `i18n.svelte.ts`.

## 💻 Konvensi Koding (Code Quality)

- **Zero Inline Comment**: Tulis kode yang _self-explanatory_ lewat penamaan variabel/fungsi yang jelas. Komentar hanya diizinkan untuk trik _workaround_ rumit atau gotcha yang sangat spesifik (menjawab "mengapa", bukan "apa").
- **Komponen Reusable**: Komponen _shared_ wajib hidup di `src/lib/components`. Dilarang menduplikasi kode UI antar halaman.
- **CSS Tokens Terpusat**: Definisi spasi, warna, dan perbatasan (_border_) dipusatkan di `app.css`. Jangan timpa ulang (_override_) di level komponen kecuali mendesak.

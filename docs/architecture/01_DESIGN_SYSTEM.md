# 📐 Finnca Design System: Utilitarian-Brutalist Specification

Sistem desain Finnca dibangun di atas paradigma **Industrial Tech & Utilitarian Brutalism**: sudut tajam absolut, hierarki monospaced ganda, kontras tinggi yang terukur, dan nol dekorasi yang tidak memiliki fungsi informasi.

---

## 1. Typography Hierarchy

Aplikasi ini menggunakan **dua font monospaced** dengan pembagian peran yang ketat:

| Font Family | Token CSS | Target Elemen / Penggunaan | Karakter Visual |
| :--- | :--- | :--- | :--- |
| **Proto Mono** | `font-proto` | Heading (`h1`, `h2`, `h3`), saldo & angka moneter, kode akun (`1001-CASH`), tab navigation, aksi tombol utama, badge status, table headers. | Padat, kokoh, berjarak tetap, visual industrial tegas. |
| **Aux Mono** | `font-aux` / `font-mono` | Deskripsi naratif, memo transaksi, catatan audit, input teks data-entry natural, label form kecil, help text. | Lebih mudah dibaca untuk teks panjang (*high readability*). |

### Aturan Disiplin Tipografi:
- **Dilarang keras** menggunakan font selain `Proto Mono` dan `Aux Mono`.
- Semua angka keuangan **wajib** menggunakan `font-proto`.
- Input teks yang menerima kalimat manusia (misal deskripsi jurnal) menggunakan `font-aux` tanpa pemaksaan huruf kapital (`uppercase`).

---

## 2. Color Tokens & Surface Elevation

Finnca menggunakan skema gelap (*dark mode*) murni dengan palet fungsional yang terdefinisi di `@theme` (`src/app.css`):

### I. Surface Elevation
| Token Semantik | Nilai Hex | Peruntukan / Konteks |
| :--- | :--- | :--- |
| `bg-bg-app` | `#0e0f0f` | Background canvas utama aplikasi / viewport terluar. |
| `bg-bg-card` | `#161717` | Kontainer data, panel formulir, card widget, sidebar background. |
| `bg-bg-btn` | `#1d1e1e` | Permukaan interaktif dasar (tombol netral, dropdown trigger, input fill). |
| `bg-bg-row-active`| `#162725` | Indikator baris aktif atau terpilih pada tabel ledger / tree node. |

### II. Functional Accents & Financial Dimensions
| Kategori Akuntansi | Token Semantik | Nilai Hex | Indikasi Finansial |
| :--- | :--- | :--- | :--- |
| **Asset** | `text-asset` | `#eab308` (Amber) | Kepemilikan, kas, piutang, investasi |
| **Liability** | `text-liability` | `#3b82f6` (Blue) | Utang, kewajiban berjalan |
| **Equity** | `text-equity` | `#a855f7` (Violet) | Modal, ekuitas pemilik |
| **Income** | `text-income` | `#3eb16f` (Green) | Penerimaan kas, revenue, surplus |
| **Expense** | `text-expense` | `#d5305f` (Red) | Beban, pengeluaran, defisit |
| **Brand Accent** | `text-teal` | `#4ea380` (Teal) | Status aktif, highlight sistem, fokus kursor |

### III. Typography Tone
| Token Semantik | Nilai Hex | Kegunaan |
| :--- | :--- | :--- |
| `text-text-white` | `#ffffff` | Judul modal penting, angka saldo utama |
| `text-text-strong` | `#d1d5db` | Teks heading, label tombol aktif |
| `text-text-base` | `#a2a9a9` | Teks body standar, data sel tabel |
| `text-text-dim` | `#585f5f` | Subtitle, placeholder input |
| `text-text-muted` | `#4e5454` | Label nonaktif, kode akun sekunder, separator |

---

## 3. Geometry, Border & Density

### I. Sudut Tajam (Zero-Radius Discipline)
Semua radius dikunci ke `0px` pada level CSS global:
```css
--radius: 0px;
--radius-sm: 0px;
--radius-md: 0px;
--radius-lg: 0px;
```
> **Aturan Wajib:** Dilarang menggunakan class `rounded`, `rounded-md`, `rounded-full`, dsb. Sudut melengkung merusak estetika industrial utilitarian Finnca.

### II. Border & Dividers
- Border standar: `1px solid var(--color-line)` (`#252828`).
- Garis pemisah tegas membatasi setiap modul (Sidebar kanan, Topbar bawah, batas kartu).
- Shadow dinonaktifkan (`--shadow: none;`)—kedalaman hierarki diciptakan melalui kontras permukaan (`surface elevation`), bukan drop-shadow lembut.

---

## 4. Shell Layout Specification

```
+-------------------------------------------------------------------------+
| TOPBAR (h: 41px) - Breadcrumb | Vault Status | Auto-Lock Countdown | UI |
+------------------+------------------------------------------------------+
| SIDEBAR          | MAIN CONTENT VIEWPORT (overflow-y: auto)             |
| (w: 225px)       |                                                      |
|                  | [PageHeader: Title + Actions + KPI Cards]            |
| - Nav Links      |                                                      |
| - Shortcut keys  | [Filter Bar / Search / Controls]                     |
| - Vault Switcher |                                                      |
| - Settings       | [Data Table / Ledger Splits / Grid Widgets]          |
|                  |                                                      |
+------------------+------------------------------------------------------+
```

- **Sidebar Width:** Tetap di `225px` (`--layout-sidebar`).
- **Topbar Height:** Tetap di `41px` (`--layout-topbar`).
- **Inspector Docked Width:** Tetap di `560px` (`--layout-inspector-docked`).
- **Inspector Float Width:** Tetap di `680px` (`--layout-inspector-float`).
- **Inspector Max Width:** Tetap di `740px` (`--layout-inspector-max`).
- **Inspector Float Height:** Tetap di `88vh` (`--layout-inspector-height`).
- **Grid Gap:** Menggunakan token `--layout-gap: 0.5rem` (8px).
- **Page Padding:** Menggunakan token `--layout-page-pad: 0.75rem` (12px).
- **Modal Overlay Blur:** Menggunakan token `--blur-subtle: 2px` (`backdrop-blur-subtle`).

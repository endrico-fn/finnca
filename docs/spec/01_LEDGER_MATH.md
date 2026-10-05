# Spesifikasi Matematis Buku Besar & Invarian Finansial (Ledger Math)

Dokumen ini mendefinisikan landasan formal, invarian matematis, representasi bilangan bulat, dan aturan pengindeksan draf transaksi pada mesin buku besar berpasangan (_double-entry ledger engine_) Finnca.

---

## 1. Zero-Float Arithmetic Invariant

### 1.1 Larangan Bilangan Mengambang (_Float Prohibition_)

Untuk menjamin presisi finansial absolut tanpa akumulasi kesalahan pembulatan IEEE 754 (_floating-point error_ seperti `0.1 + 0.2 = 0.30000000000000004`), seluruh komputasi finansial di Finnca mematuhi invarian:

$$\text{Type}(\text{Amount}) \in \{ \mathbf{i64}, \mathbf{i128} \} \quad \text{dan} \quad \text{Type}(\text{Amount}) \notin \{ \mathbf{f32}, \mathbf{f64}, \mathbf{number} \}$$

Semua angka moneter dikelola dalam **integer minor units** (satuan terkecil yang tidak dapat dibagi lagi untuk mata uang bersangkutan).

### 1.2 Skala Satuan Minor Mata Uang (_Minor Unit Factors_)

Setiap komoditas/mata uang memiliki faktor pengali satuan minor $M(c)$ yang ditentukan secara deterministik (`src-tauri/src/ledger/currency.rs`):

| Kode Komoditas             | Satuan Terkecil | Faktor Minor $M(c)$ | Contoh Nilai Rp/Cents | Representasi Integer |
| -------------------------- | --------------- | ------------------- | --------------------- | -------------------- |
| `IDR`, `JPY`, `KRW`, `VND` | 1 unit          | $10^0 = 1$          | Rp 50.000             | `50000`              |
| `USD`, `EUR`, `SGD`, `GBP` | Cent            | $10^2 = 100$        | $ 12.50               | `1250`               |
| `BHD`, `KWD`, `OMR`        | Fils            | $10^3 = 1000$       | 1.250 KWD             | `1250`               |

Frontend tidak pernah menghitung ulang saldo akun untuk mengambil keputusan bisnis. Frontend hanya menerima integer minor unit dari Rust IPC dan memformatnya secara _display-only_ melalui modul `src/lib/core/format/currency.ts`.

---

## 2. Invarian Keseimbangan Jurnal Berpasangan (_Double-Entry Balance_)

### 2.1 Persamaan Fundamental Keseimbangan

Setiap transaksi finansial $T$ terdiri dari himpunan posting $P = \{ p_1, p_2, \dots, p_n \}$ dengan $n \ge 2$. Keseimbangan transaksi wajib memenuhi:

$$\sum_{i=1}^{n} \text{amount}(p_i) = 0$$

di mana:

- **Debit** didefinisikan sebagai nominal positif: $\text{amount} > 0$ (meningkatkan Akun Aset / Beban).
- **Kredit** didefinisikan sebagai nominal negatif: $\text{amount} < 0$ (meningkatkan Akun Liabilitas / Ekuitas / Pendapatan).

Secara eksplisit:
$$\sum \text{Debit} + \sum \text{Credit} = 0 \iff \sum |\text{Debit}| = \sum |\text{Credit}|$$

### 2.2 Larangan Posting Bernilai Nol (_Non-Zero Invariant_)

Setiap kaki jurnal $p_i$ wajib memiliki nilai bukan nol:
$$\forall p \in P, \quad \text{amount}(p) \ne 0$$
Posting dengan $\text{amount} = 0$ ditolak secara mutlak pada lapisan validasi Rust (`src-tauri/src/ledger/validation.rs`).

---

## 3. Konvensi Draf Transaksi 2-Kaki (_Debit-First Draft Indexing_)

Untuk seluruh alur pencatatan 2-kaki (Transfer Antar-Akun, Pemasukan Sederhana, Pengeluaran Sederhana), urutan indeks kaki draf dibakukan secara tegas:

$$ \begin{aligned}
\text{Index } 0 &\triangleq \mathbf{Debit} \quad (\text{Akun Penerima / Tujuan}, \quad \text{amount} \ge 0) \\
\text{Index } 1 &\triangleq \mathbf{Credit} \quad (\text{Akun Pengirim / Sumber}, \quad \text{amount} \le 0)
\end{aligned}$$

### 3.1 Hubungan Nilai Simetris
$$\text{amount}[0] + \text{amount}[1] = 0 \iff \text{amount}[1] = -\text{amount}[0]$$

### 3.2 Implikasi Perpindahan Antarmuka (*Mode Switching*)
Ketika pengguna beralih mode antarmuka antara **Transfer Sederhana** dan **Multi-Split Entry**:
- Nilai akun pada Index 0 dan Index 1 tidak boleh dihapus atau di-reset meskipun nominal transaksi saat ini masih bernilai 0.
- Transisi status draf mempertahankan integritas urutan debit-kredit tanpa menyebabkan *CSS drift* atau hilangnya konteks input pengguna.

---

## 4. Aritmetika Pembulatan Simetris (*Symmetric Half-Up Rounding*)

Dalam operasi konversi valuta asing (FX) atau alokasi proporsional, pembagian bilangan bulat minor unit wajib menggunakan algoritma pembulatan simetris terhadap nol (*Round Half Up towards $+\infty$ for positive, $-\infty$ for negative*):

$$\text{div\_half\_up}(N, D) = \begin{cases}
0, & \text{jika } D = 0 \\
\left\lfloor \dfrac{N + \lfloor |D| / 2 \rfloor}{D} \right\rfloor, & \text{jika } N \ge 0 \\
\left\lfloor \dfrac{N - \lfloor |D| / 2 \rfloor}{D} \right\rfloor, & \text{jika } N < 0
\end{cases}$$

Implementasi Rust menggunakan tipe komputasi `i128` untuk mencegah *overflow* pada perkalian sebelum pembagian:
```rust
fn div_half_up(num: i128, denom: i128) -> i128 {
    if denom == 0 {
        return 0;
    }
    let half = denom.abs() / 2;
    if num >= 0 {
        (num + half) / denom
    } else {
        (num - half) / denom
    }
}
```

---

## 5. Transaksi Multi-Mata Uang & Keseimbangan Biaya (*Multi-Currency Cost Balance*)

Ketika sebuah transaksi melibatkan mata uang campuran (misalnya pembelian aset berdenominasi `USD` menggunakan rekening `IDR`):

1. **Per-Commodity Balance**: Jika semua posting memiliki komoditas yang sama, per-commodity balance wajib bernilai 0.
2. **Functional Currency Conversion**: Jika posting melibatkan lebih dari satu komoditas:
   - Setiap posting $p_i$ dievaluasi ke dalam mata uang dasar fungsional (`IDR`) menggunakan kurs spesifik transaksi $R_i$ atau biaya eksplisit $\text{cost\_amount}_i$:
   $$\text{base\_amount}(p_i) = \begin{cases}
   \text{sign}(p_i) \cdot |\text{cost\_amount}_i|, & \text{jika } \text{cost\_amount}_i \text{ ditentukan} \\
   \text{amount}(p_i), & \text{jika } \text{curr}(p_i) = \text{IDR} \\
   \text{convert\_minor\_units}(\text{amount}(p_i), \text{curr}(p_i), \text{IDR}, R_i), & \text{lainnya}
   \end{cases}$$
3. **Functional Zero-Sum Invariant**:
   $$\sum_{i=1}^{n} \text{base\_amount}(p_i) = 0$$

Jika $\sum \text{base\_amount}(p_i) \ne 0$, transaksi ditolak dengan eror `Unbalanced multi-currency transaction`.

---

## 6. Validasi Periode Tertutup (*Closing Date Invariant*)

Untuk menjamin catatan audit tamper-evident dan stabilitas laporan keuangan lampau:
- Jika `closing_date` ditetapkan pada tanggal $D_{\text{close}}$:
$$\forall T, \quad \text{date}(T) \le D_{\text{close}} \implies \text{Status}(T) = \mathbf{Locked}$$
- Setiap upaya membuat, mengubah, atau menghapus transaksi dengan tanggal pada atau sebelum $D_{\text{close}}$ ditolak dengan kode eror `PeriodLocked`.
$$

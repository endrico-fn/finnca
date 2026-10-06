# ADR 0007: WebAssembly Plugin Sandboxing via Extism Architecture

## Status

Accepted (Standar Arsitektur Ekstensibilitas Fase 2)

## Context

Aplikasi desktop Finnca dirancang sebagai sistem pencatatan keuangan pribadi tingkat lanjut (*advanced personal finance note & double-entry ledger*) yang mengutamakan privasi terisolasi (*vault-centric*) dan komputasi presisi tinggi tanpa kompromi (*zero-float invariant*). Untuk memperluas kapabilitas ekosistem tanpa membebani repositori inti (*core engine*), Finnca memerlukan arsitektur ekstensibilitas (*plugin system*) pihak ketiga dan komunitas untuk tiga kasus penggunaan utama:

1. **Bank Statement Parsers**: Ekstraksi dan normalisasi mutasi rekening koran dari berbagai institusi perbankan lokal maupun global (misal: BCA, Mandiri, BNI, BRI, Chase, DBS) dengan format dokumen heterogen (CSV, OFX/QFX, SWIFT MT940, CAMT.053 XML, hingga PDF bank statement).
2. **Custom Financial Reports**: Pembuatan laporan dan analisis keuangan kustom sesuai kebutuhan yurisdiksi atau preferensi personal (misal: formulir pajak penghasilan Indonesia SPT Tahunan PPh 21 / PPh Final 0.5% UMKM, simulasi *cash flow runway*, atau model *portfolio rebalancing* multi-mata uang).
3. **External Sync Adapters**: Pengiriman draf cadangan terenkripsi atau sinkronisasi ledger berbasis teks ke penyimpanan terdistribusi (WebDAV, Nextcloud, S3 cold storage, atau Git repository berbasis Beancount/plain-text ledger).

### Tantangan Keamanan & Invarian Finansial

Pengenalan kode pihak ketiga ke dalam aplikasi keuangan menghadirkan risiko keamanan eksistensial:
- **Eksfiltrasi Kunci Kriptografi & Database Decrypted**: Finnca menerapkan *envelope encryption* (Argon2id + ChaCha20-Poly1305) dan basis data terenkripsi SQLCipher. Jika plugin berjalan di dalam ruang memori host (*native host address space*), plugin berbahaya dapat memindai heap memori untuk mengekstrak *Key Encryption Key* (KEK), *Data Encryption Key* (DEK), atau membaca halaman plaintext SQLite langsung dari buffer SQLCipher.
- **Kontaminasi Angka Pecahan (*Zero-Float Violation*)**: Aksioma fundamental Finnca (AGENTS.md § 1.3) melarang keras penggunaan tipe data floating-point (`f32`, `f64`, `number`) dalam komputasi saldo dan jurnal. Seluruh angka moneter wajib beroperasi pada integer minor units (`i64` / `i128`). Plugin yang memancarkan angka desimal IEEE-754 berpotensi merusak integritas pembukuan buku besar.
- **Invarian Jurnal Berpasangan (*Double-Entry & Debit-First*)**: Setiap mutasi wajib mematuhi aturan balance `sum(Debit) == sum(Credit)` dan urutan kaki standar Debit-First (Index 0 = Debit, Index 1 = Kredit). Plugin tidak boleh diizinkan menulis langsung ke SQLite secara sepihak tanpa validasi host dan konfirmasi pengguna.
- **Denial of Service (DoS) & UI Freezing**: Parser bank statement yang memproses dokumen besar (hingga 5 MB) atau ekspresi reguler kompleks berisiko mengalami *infinite loop*, menghabiskan memori RAM, atau membekukan event loop Tauri.

### Evaluasi Kandidat Runtime Ekstensibilitas

Kami mengevaluasi 7 kandidat teknologi eksekusi plugin:

| Teknologi Runtime | Isolasi Memori | Pembatasan Komputasi (Fuel/Epoch) | Latensi Eksekusi | Ekosistem Penulis Plugin | Overhead Biner | Keputusan |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Native DLLs (`libloading`)** | **Zero (0/10)**: Akses penuh ke heap host & pointer SQLCipher. | Tidak ada (dapat membekukan host secara permanen). | < 0.01 ms | Terbatas (C ABI, Rust, Zig). | < 50 KB | **DITOLAK MUTLAK**: Melanggar keamanan vault secara fatal. |
| **Embedded Lua (`mlua`)** | **Rendah (4/10)**: Berbagi heap proses; bergantung pada keamanan C VM. | Hook instruksi sederhana (tidak deterministik). | < 0.2 ms | Sangat sempit untuk komunitas analisis keuangan. | ~ 500 KB | **DITOLAK**: Ekosistem parser perbankan tidak menggunakan Lua. |
| **Embedded JS (`rquickjs`)** | **Sedang (5/10)**: VM sandbox logis, namun memori berada di proses C/Rust. | Hook waktu/tick sederhana. | < 0.5 ms | Hanya JavaScript/TypeScript; tidak mendukung bahasa kompilasi. | ~ 1.5 MB | **DITOLAK**: Kurang deterministik dan isolasi heap rentan eksfiltrasi. |
| **Standalone Wasmtime** | **Maksimum (10/10)**: *Linear memory sandbox* terisolasi perangkat keras. | Metering bahan bakar (*fuel*) & epoch timeout deterministik. | Cold: 2 ms<br>Warm: < 0.1 ms | Polyglot (Rust, Go, C, TS via Component Model/Wasm). | ~ 3.5 MB | **ALTERNATIF LAYAK**: Memerlukan boilerplate ABI scaffolding manual yang sangat besar. |
| **Out-of-Process CLI** | **Tinggi (8/10)**: Proses OS terpisah. | Manajemen via process kill OS (`SIGKILL`). | Cold: 50–200 ms<br>Warm: 5–10 ms | Bergantung pada runtime lokal pengguna (Node/Python). | 0 MB host (tetapi butuh dependensi OS). | **DITOLAK**: Latensi IPC pipa OS terlalu tinggi; instalasi runtime eksternal rapuh. |
| **Extism (Wasmtime Backend)** | **Maksimum (10/10)**: *Linear memory* Wasmtime + sandboxing WASI. | *Fuel budget* per instruksi + *wall-clock epoch interrupts*. | Cold: 2–4 ms<br>Warm: < 0.2 ms | **Sangat Luas (16+ PDK)**: Rust, TypeScript/JS, Go, Python, Zig, C. | ~ 3.8 MB | **TERPILIH (WINNER)**: Tingkat keamanan militer, PDK terstandarisasi, dan kontrol host ketat. |

---

## Decision

Kami menetapkan adopsi **Extism** (berbasis runtime Wasmtime dari Bytecode Alliance) yang diintegrasikan langsung ke dalam backend Rust Finnca (`src-tauri/src/plugins/`) sebagai satu-satunya arsitektur ekstensibilitas aplikasi:

1. **Isolasi Memori Linear Wasm (*Hardware-Enforced Linear Memory*)**:
   - Setiap instansiasi plugin dieksekusi di dalam ruang memori linier 32-bit WebAssembly yang sepenuhnya terisolasi dari heap proses utama Finnca.
   - Diberlakukan batas keras alokasi memori (*hard memory cap*) sebesar **32 MB** per plugin (`max_pages = 512`). Upaya alokasi memori melebihi batas ini memicu *out-of-memory trap* yang ditangani secara aman oleh host tanpa memengaruhi kestabilan aplikasi.
   - Plugin tidak memiliki akses ke pointer host, handle koneksi SQLite, KEK, DEK, maupun buffer SQLCipher.

2. **Pengendalian Eksekusi Deterministik (*Instruction Fuel & Epoch Deadlines*)**:
   - Setiap pemanggilan fungsi plugin dibatasi oleh kuota instruksi (*fuel budget*) maksimal **10.000.000 unit**.
   - Setiap pemanggilan dibatasi oleh batas waktu jam dinding (*wall-clock timeout*) maksimal **5.000 milidetik (5 detik)** menggunakan mekanisme *epoch deadline* Wasmtime.
   - *Infinite loop*, *regex catastrophic backtracking* (ReDoS), atau kalkulasi tak terhingga akan dihentikan paksa (*aborted*) oleh host dengan mengembalikan kode kesalahan `ERR_PLUGIN_TIMEOUT` atau `ERR_PLUGIN_OUT_OF_FUEL`.

3. **Gerbang Validasi Anti-Kontaminasi Pecahan (*Zero-Float Gateway*)**:
   - Komunikasi data melintasi batas host-plugin menggunakan format serialisasi memori terstruktur (JSON / MessagePack).
   - Seluruh payload keluaran dari plugin dipindai oleh deserializer validasi host (`validate_zero_float_integrity`).
   - **Penolakan Mutlak Seluruh Float IEEE-754**: Setiap token angka pada JSON/MessagePack yang berjenis floating-point (`num.is_f64()`) langsung ditolak tanpa syarat dengan galat fatal `ERR_PLUGIN_FLOAT_VIOLATION`. Tidak ada toleransi untuk angka tanpa sisa pecahan (`fract() == 0.0`), sehingga payload seperti `15000.0`, notasi ilmiah `1e2`, angka besar `1e20`, maupun float berisiko pembulatan presisi mantissa 53-bit langsung ditolak keras sebelum mencapai deserializer DTO. Hanya bilangan bulat murni (`i64` / `u64`) dalam rentang minor units yang diizinkan.
   - **Pemindaian String Numerik Terselubung & Nilai Non-Finit**: Deserializer memeriksa seluruh nilai string (`Value::String`). String yang memuat nilai non-finit (`"NaN"`, `"Infinity"`, `"-Infinity"`, `"inf"`, `"-inf"`) atau representasi angka desimal pecahan/eksponensial (misal `"15000.50"`, `"1e3"`) langsung digagalkan untuk mencegah penyelundupan representasi float melalui tipe data teks.
   - Seluruh nominal moneter wajib dinyatakan dalam format bilangan bulat terkecil (*integer minor units*, tipe `i64` / `i128`), dan metrik tren persentase wajib dinyatakan dalam *basis points* (`trend_basis_points: Option<i64>`).

4. **Invarian Double-Entry & Penegakan Standar Debit-First**:
   - Plugin **dilarang keras** memiliki akses tulis langsung (*direct write*) ke berkas SQLite atau SQLCipher.
   - Plugin hanya diizinkan memancarkan objek draf (*draft proposals*): `DraftJournalEntry` atau `ParsedStatementRow`.
   - **Validasi Gerbang Host (*Host Invariant Gateway*)**:
     Sebelum draf diproses atau disajikan ke pengguna, fungsi validasi host `validate_plugin_draft_entry` secara eksplisit menegakkan:
     - **Invarian Debit-First**: Untuk entri 2-kaki (Transfer / Simple Entry), urutan kaki dibakukan mutlak: `postings[0]` adalah Debit (`amount >= 0`, akun tujuan/penerima) dan `postings[1]` adalah Kredit (`amount <= 0`, akun sumber/pengirim). Proposal draf dengan urutan terbalik (`[Credit, Debit]`) langsung ditolak dengan kode `ERR_PLUGIN_INVARIANT_DEBIT_FIRST`.
     - **Nominal Non-Nol**: Memastikan tidak ada kaki transaksi dengan nominal 0 (`amount != 0`).
     - **Keseimbangan Double-Entry**: Memastikan `sum(Debit) == sum(Credit)` per mata uang / komoditas melalui `validate_postings_balance_with_context`.
   - Draf transaksi hanya dapat dimasukkan ke dalam buku besar permanen setelah melalui verifikasi visual dan persetujuan eksplisit (*user confirmation*) dari pengguna pada antarmuka frontend.

5. **Model Hak Akses Berbasis Manifestasi Kapabilitas (*Capability Manifest*)**:
   - Setiap plugin wajib menyertakan berkas konfigurasi deklaratif `plugin.toml` yang mendefinisikan identitas kriptografis dan izin yang dibutuhkan.
   - Hak akses filesystem default adalah `fs:none` (WASI filesystem diblokir). Plugin hanya menerima buffer berkas input yang diinjeksikan secara aman oleh host (maksimal 5 MB).
   - Hak akses jaringan default adalah `net:none` (WASI socket diblokir). Plugin sync adapter yang memerlukan komunikasi internet wajib mendeklarasikan domain HTTPS yang diizinkan (*domain whitelist*), yang harus disetujui pengguna saat pemasangan plugin.
   - Verifikasi integritas biner: Host memvalidasi checksum SHA-256 dan tanda tangan digital Minisign dari berkas `plugin.wasm` sebelum memuat modul ke dalam memori.

---

## Consequences

### Positif
- **Isolasi Keamanan Tingkat Tinggi**: Kunci kriptografi envelope encryption, halaman plaintext SQLite, dan data rahasia pengguna aman 100% dari eksfiltrasi karena isolasi batas memori linier WebAssembly.
- **Ekosistem Penulis Plugin Multibahasa (Polyglot)**: Komunitas dapat menulis plugin dalam bahasa pemrograman favorit mereka (Rust, TypeScript/JavaScript, Go via TinyGo, Python via Componentize-Py, Zig, atau C) menggunakan Extism Plug-in Development Kit (PDK) resmi.
- **Kekebalan terhadap UI Freeze & DoS**: Pembatasan *instruction fuel* dan *epoch timeout* menjamin plugin bermasalah tidak akan pernah menggantung antarmuka desktop Finnca.
- **Konsistensi Invarian Akuntansi Mutlak**: Gerbang Zero-Float dan validasi double-entry di sisi Rust host menjamin buku besar tidak akan pernah terkontaminasi oleh angka pecahan floating-point IEEE-754.
- **Arsitektur Tanpa Instalasi Eksternal**: Pengguna akhir tidak perlu menginstal Node.js, Python, atau runtime pihak ketiga apa pun di komputer mereka; modul `.wasm` berjalan secara *self-contained* di dalam binary Finnca.

### Trade-off & Mitigasi
- **Pertambahan Ukuran Biner (+3.8 MB)**: Mesin Wasmtime dan compiler Cranelift menambahkan sekitar 3.8 MB hingga 4.5 MB pada biner terkompilasi `finnca_lib`.  
  *Mitigasi*: Dikompensasi dengan konfigurasi optimasi biner rilis (*Fat LTO*, `codegen-units = 1`, dan `strip = true`) yang memangkas overhead biner secara keseluruhan.
- **Overhead Serialisasi Lintas Batas Memori**: Pengiriman buffer data antara memori host Rust dan memori linier Wasm memerlukan serialisasi/deserialisasi (< 0.5 ms untuk berkas 5 MB).  
  *Mitigasi*: Karena parsing dokumen perbankan dipicu oleh interaksi pengguna (bukan loop animasi 60 FPS), latensi sub-milidetik ini sepenuhnya imperseptibel bagi pengguna.
- **Kompleksitas Kompilasi Parser PDF di Guest Wasm**: Parsing dokumen PDF di dalam Wasm memerlukan pustaka PDF berbasis Rust/Go murni tanpa dependensi library C sistem (seperti Poppler).  
  *Mitigasi*: Disediakan template resmi `finnca-plugin-starter` dengan parser PDF berbasis pure-Rust (`lopdf` / `pdf-extract`) yang telah dikonfigurasi untuk target `wasm32-wasip1`.

---

## Technical Architecture & Code Contracts

### 1. Diagram Batas Isolasi Runtime dan Gerbang Invarian

```
+-----------------------------------------------------------------------------------------+
|                                    FINNCA DESKTOP HOST                                  |
|                                                                                         |
|  +--------------------+     +----------------------+     +---------------------------+  |
|  | SQLCipher Database |     | Argon2id / ChaCha20  |     | Master Password / KEK/DEK |  |
|  | (Encrypted Vault)  |     | (Envelope Crypto)    |     | (zeroize::Zeroizing RAM)  |  |
|  +---------+----------+     +----------+-----------+     +-------------+-------------+  |
|            |                           |                               |                |
|            x===========================x===============================x                |
|            |    DINDING PEMISAH MUTLAK: ZERO POINTER / ZERO HANDLE EXPOSURE TO WASM     |
|            |                                                                            |
|  +---------v-------------------------------------------------------------------------+  |
|  |                        HOST CAPABILITY GATE & INVARIANT GATEWAY                   |  |
|  |  1. Menolak fungsi syscall OS / socket yang tidak diizinkan di manifest               |  |
|  |  2. Memindai buffer: REJECT ALL float (num.is_f64) & String NaN/desimal/eksponensial |  |
|  |  3. Memvalidasi Double-Entry: sum(Debit) == sum(Credit) && amount != 0               |  |
|  |  4. Menegakkan urutan Debit-First: [0] = Debit (>= 0), [1] = Credit (<= 0)           |  |
|  +---------------------------------------+-------------------------------------------+  |
+------------------------------------------|----------------------------------------------+
                                           | Extism Memory Safe Buffer Exchange
                                           v
+-----------------------------------------------------------------------------------------+
|                         WASM RUNTIME SANDBOX (Extism / Wasmtime Engine)                 |
|                                                                                         |
|   Memory Hard Cap: 32 MB  |  Fuel Budget: 10M Units  |  Wall-clock Timeout: 5,000 ms    |
|                                                                                         |
|   +----------------------------------------------------------------------------------+  |
|   | GUEST PLUGIN EXECUTION SPACE (wasm32-wasip1)                                     |  |
|   | Bahasa: Rust, TypeScript, Go, Python, Zig, C                                     |  |
|   | - Statement Parsers (CSV, OFX, XML, PDF)                                         |  |
|   | - Custom Financial Reports (Tax Schedules, Runway Simulation)                    |  |
|   | - External Sync Adapters (WebDAV, S3, Beancount plain-text)                      |  |
|   +----------------------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------------------+
```

---

### 2. Spesifikasi Manifestasi Plugin (`plugin.toml`)

Setiap paket plugin wajib didistribusikan bersama berkas manifes deklaratif dengan skema berikut:

```toml
manifest_version = 1

[plugin]
id = "org.finnca.parser.bca"
name = "BCA KlikBCA & e-Statement Parser"
version = "1.2.0"
author = "Finnca Community <community@finnca.org>"
description = "Parser presisi tinggi untuk mutasi rekening CSV dan e-Statement PDF Bank Central Asia."
license = "MIT"
homepage = "https://github.com/finnca-plugins/parser-bca"
min_finnca_version = "0.2.0"
target_abi_version = "1.0"

[binary]
entrypoint = "dist/plugin.wasm"
sha256 = "c8f2b740562e105e192ff788cb20e542ef490538a846ad762e847c1775e4785b"
signature = "untrusted comment: minisign public signature\nRWR...base64..."

[limits]
max_memory_mb = 32
timeout_ms = 5000
fuel_budget = 10000000

[permissions]
filesystem = ["scoped_read"]  # Opsi: "none", "scoped_read", "temp"
network = []                 # Whitelist domain HTTPS; kosong berarti airgapped
ledger = "none"              # Opsi: "none", "read_aggregate", "read_full", "propose_draft"
notifications = true         # Mengizinkan toast notifikasi tersanitasi

[extensions]
statement_parsers = [
  { id = "bca_csv", name = "BCA KlikBCA CSV", file_extensions = ["csv"], mime_types = ["text/csv"] },
  { id = "bca_pdf", name = "BCA e-Statement PDF", file_extensions = ["pdf"], mime_types = ["application/pdf"] }
]
```

---

### 3. Kontrak ABI Antarmuka Ekstensibilitas (Rust Host ABIs)

#### A. Ekstensi 1: Statement Parser (`StatementParser`)

Digunakan untuk membaca mutasi rekening koran dan mengubahnya menjadi baris transaksi terstruktur dalam satuan integer minor units.

```rust
// File: src-tauri/src/plugins/abi/statement.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseStatementInput {
    pub parser_id: String,
    pub file_name: String,
    pub file_bytes_base64: String,
    pub account_currency: String,
    pub account_is_debit_normal: bool,
    pub options: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedStatementRow {
    /// Format tanggal terstandarisasi ISO-8601: "YYYY-MM-DD"
    pub date: String,
    /// Nominal dalam integer minor units (positif untuk uang masuk, negatif untuk uang keluar)
    pub amount: i64,
    pub description: Option<String>,
    pub reference_no: Option<String>,
    pub balance_after: Option<i64>,
    pub payee_or_payer: Option<String>,
    pub category_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedStatementOutput {
    pub bank_name: Option<String>,
    pub statement_account_number: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub opening_balance: Option<i64>,
    pub closing_balance: Option<i64>,
    pub rows: Vec<ParsedStatementRow>,
}
```

#### B. Ekstensi 2: Custom Financial Report (`FinancialReport`)

Digunakan untuk kalkulasi agregat laporan keuangan kustom tanpa mengekspos transaksi individual yang bersifat privat.

```rust
// File: src-tauri/src/plugins/abi/report.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportInput {
    pub report_id: String,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub parameters: std::collections::HashMap<String, String>,
    pub context: ReportLedgerContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportLedgerContext {
    pub base_currency: String,
    pub accounts: Vec<ReportAccountSummary>,
    pub net_income_minor: i64,
    pub total_assets_minor: i64,
    pub total_liabilities_minor: i64,
    pub total_equity_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAccountSummary {
    pub account_id: String,
    pub code: String,
    pub name: String,
    /// Tipe akun: "ASSET", "LIABILITY", "EQUITY", "INCOME", "EXPENSE"
    pub account_type: String,
    pub currency: String,
    pub balance_minor: i64,
    pub period_debit_minor: i64,
    pub period_credit_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomReportView {
    pub title: String,
    pub subtitle: Option<String>,
    pub summary_cards: Vec<ReportCard>,
    pub sections: Vec<ReportTableSection>,
    pub chart_series: Option<Vec<ReportChartSeries>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportCard {
    pub label: String,
    pub amount_minor: i64,
    pub currency: String,
    /// Token gaya antarmuka: "neutral", "ok", "err", "warn", "teal"
    pub tone: String,
    /// Perubahan tren dalam basis points (1 bp = 0.01%, misal 520 = +5.20%, -150 = -1.50%).
    /// Wajib integer murni minor units untuk mematuhi invarian Zero-Float tanpa kontradiksi skema.
    pub trend_basis_points: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTableSection {
    pub title: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_row: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportChartSeries {
    pub name: String,
    /// Titik data berupa pasangan (Label Waktu, Nilai dalam integer minor units)
    pub data_points: Vec<(String, i64)>,
}
```

#### C. Ekstensi 3: External Sync Adapter (`SyncAdapter`)

Digunakan untuk sinkronisasi terenkripsi ke penyimpanan eksternal.

```rust
// File: src-tauri/src/plugins/abi/sync.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncAdapterInput {
    pub adapter_id: String,
    pub vault_name: String,
    /// Format muatan: "ENCRYPTED_ZIP_AGE" atau "BEANCOUNT_PLAIN"
    pub payload_format: String,
    pub payload_bytes_base64: String,
    pub config: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub remote_revision: Option<String>,
    pub message: String,
    pub synced_at_unix: i64,
}
```

---

### 4. Implementasi Host Manager & Zero-Float Validation Gateway

Arsitektur host di sisi Rust yang menginisialisasi sandbox Extism, memeriksa limitasi memori, serta memvalidasi ketiadaan floating-point desimal:

```rust
// File: src-tauri/src/plugins/manager.rs
use crate::shared::AppError;
use extism::{Manifest, Plugin, Wasm};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub struct PluginHostManager {
    plugins_directory: PathBuf,
}

impl PluginHostManager {
    pub fn new(plugins_directory: PathBuf) -> Self {
        Self { plugins_directory }
    }

    /// Memuat dan mengisolasi plugin Extism Wasm dari disk dengan validasi integritas ketat.
    pub fn load_sandboxed_plugin(&self, plugin_id: &str) -> Result<Plugin, AppError> {
        let plugin_dir = self.plugins_directory.join(plugin_id);
        let manifest_path = plugin_dir.join("plugin.toml");

        if !manifest_path.exists() {
            return Err(AppError::NotFound(format!("Plugin manifest not found at {:?}", manifest_path)));
        }

        let manifest_raw = std::fs::read_to_string(&manifest_path)?;
        let manifest: super::manifest::PluginManifest = toml::from_str(&manifest_raw)
            .map_err(|e| AppError::InvalidInput(format!("Invalid plugin manifest TOML: {e}")))?;

        let wasm_file = plugin_dir.join(&manifest.binary.entrypoint);
        let wasm_bytes = std::fs::read(&wasm_file)?;

        // 1. Verifikasi Checksum SHA-256
        let calculated_hash = format!("{:x}", Sha256::digest(&wasm_bytes));
        if calculated_hash != manifest.binary.sha256 {
            return Err(AppError::Crypto(format!(
                "Integritas plugin gagal: checksum hash tidak cocok! Harapan: {}, Terkalkulasi: {}",
                manifest.binary.sha256, calculated_hash
            )));
        }

        // 2. Konfigurasi Sandbox Extism dengan batas memori keras (32 MB)
        let mut extism_manifest = Manifest::new([Wasm::data(wasm_bytes)]);
        let max_pages = (manifest.limits.max_memory_mb * 1024 * 1024 / 65536) as u32;
        extism_manifest = extism_manifest.with_memory_options(extism::manifest::MemoryOptions {
            max_pages: Some(max_pages),
            ..Default::default()
        });

        // 3. Instansiasi modul Wasm (WASI sandboxed, isolasi host mutlak)
        let plugin = Plugin::new(&extism_manifest, [], true)
            .map_err(|e| AppError::InvalidInput(format!("Gagal menginstansiasi sandbox Wasm: {e}")))?;

        Ok(plugin)
    }

    /// Menjalankan fungsi plugin dengan proteksi timeout, fuel, dan gerbang anti-float.
    pub fn invoke_function<I: Serialize, O: for<'de> Deserialize<'de>>(
        &self,
        mut plugin: Plugin,
        function_name: &str,
        input: &I,
    ) -> Result<O, AppError> {
        let input_bytes = serde_json::to_vec(input)
            .map_err(|e| AppError::InvalidInput(format!("Gagal serialisasi input: {e}")))?;

        // Eksekusi di dalam sandbox WebAssembly
        let output_bytes = plugin
            .call(function_name, &input_bytes)
            .map_err(|e| AppError::InvalidInput(format!("Eksekusi plugin gagal: {e}")))?;

        // Gerbang Validasi: Pastikan tidak ada angka pecahan (float) yang diselundupkan
        validate_zero_float_integrity(&output_bytes)?;

        let output: O = serde_json::from_slice(output_bytes)
            .map_err(|e| AppError::InvalidInput(format!("Gagal deserialisasi output plugin: {e}")))?;

        Ok(output)
    }
}

/// Memeriksa seluruh hierarki JSON dari output plugin untuk mencegah kontaminasi IEEE-754 float
/// maupun penyelundupan angka pecahan / nilai non-finit via string.
pub fn validate_zero_float_integrity(payload: &[u8]) -> Result<(), AppError> {
    let val: serde_json::Value = serde_json::from_slice(payload)
        .map_err(|e| AppError::InvalidInput(format!("Payload plugin bukan JSON valid: {e}")))?;

    fn walk_json(v: &serde_json::Value, path: &str) -> Result<(), AppError> {
        match v {
            serde_json::Value::Number(num) => {
                // Aksioma Zero-Float Mutlak: Seluruh tipe floating-point ditolak tanpa syarat!
                // Tidak ada toleransi untuk bilangan bulat berkoma (misal 15000.0, 1e2, 1e20).
                if num.is_f64() {
                    return Err(AppError::InvalidInput(format!(
                        "ERR_PLUGIN_FLOAT_VIOLATION: Nilai floating-point IEEE-754 terdeteksi pada '{path}': {num}. Seluruh nominal moneter wajib berupa integer minor units murni (i64/u64)."
                    )));
                }
            }
            serde_json::Value::String(s) => {
                let trimmed = s.trim();
                // Deteksi angka non-finit (NaN / Infinity) dalam representasi string
                let lower = trimmed.to_lowercase();
                if lower == "nan" || lower == "infinity" || lower == "+infinity" || lower == "-infinity" || lower == "inf" || lower == "-inf" {
                    return Err(AppError::InvalidInput(format!(
                        "ERR_PLUGIN_FLOAT_VIOLATION: Nilai non-finit (NaN/Infinity) terdeteksi pada string '{path}': \"{s}\"."
                    )));
                }
                // Deteksi penyelundupan angka desimal pecahan / eksponensial dalam string (misal "15000.50", "1e3")
                if (trimmed.contains('.') || trimmed.contains('e') || trimmed.contains('E'))
                    && trimmed.parse::<f64>().is_ok()
                {
                    return Err(AppError::InvalidInput(format!(
                        "ERR_PLUGIN_FLOAT_VIOLATION: Representasi desimal/eksponensial pecahan terdeteksi pada string '{path}': \"{s}\". Seluruh data finansial wajib beroperasi pada integer minor units."
                    )));
                }
            }
            serde_json::Value::Array(items) => {
                for (idx, item) in items.iter().enumerate() {
                    walk_json(item, &format!("{path}[{idx}]"))?;
                }
            }
            serde_json::Value::Object(map) => {
                for (key, item) in map.iter() {
                    walk_json(item, &format!("{path}.{key}"))?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    walk_json(&val, "root")
}

/// Memvalidasi integritas invarian Debit-First dan keseimbangan double-entry
/// untuk draf transaksi yang diusulkan oleh plugin pihak ketiga sebelum disajikan ke pengguna.
pub fn validate_plugin_draft_entry(
    postings: &[crate::ledger::model::DraftPosting],
) -> Result<(), AppError> {
    if postings.len() < 2 {
        return Err(AppError::InvalidInput(
            "ERR_PLUGIN_INVALID_ENTRY: Jurnal minimal harus memiliki 2 kaki transaksi.".into(),
        ));
    }

    // Penegakan Standar Invarian Debit-First untuk transaksi 2-kaki (Transfer / Simple Entry)
    if postings.len() == 2 {
        let debit_leg = &postings[0];
        let credit_leg = &postings[1];

        if debit_leg.amount < 0 || credit_leg.amount > 0 {
            return Err(AppError::InvalidInput(format!(
                "ERR_PLUGIN_INVARIANT_DEBIT_FIRST: Urutan kaki transaksi melanggar standar Debit-First. Kaki [0] wajib Debit (amount >= 0, ditemukan {}), Kaki [1] wajib Kredit (amount <= 0, ditemukan {}).",
                debit_leg.amount, credit_leg.amount
            )));
        }
    }

    // Pastikan tidak ada kaki transaksi dengan nominal 0
    for (idx, p) in postings.iter().enumerate() {
        if p.amount == 0 {
            return Err(AppError::InvalidInput(format!(
                "ERR_PLUGIN_ZERO_AMOUNT: Kaki transaksi [{idx}] memiliki nominal nol."
            )));
        }
    }

    // Validasi keseimbangan Double-Entry (sum == 0)
    let total_balance: i128 = postings.iter().map(|p| p.amount as i128).sum();
    if total_balance != 0 {
        return Err(AppError::InvalidInput(format!(
            "ERR_PLUGIN_UNBALANCED_ENTRY: Jurnal tidak seimbang. Total selisih: {total_balance} minor units."
        )));
    }

    Ok(())
}
```

---

### 5. Strategi Pengujian & Verifikasi Kepatuhan Invarian

1. **Uji Penolakan Floating Point Komprehensif (*Zero-Float Rejection Test*)**:
   - Menyuapkan payload dengan pecahan desimal: `{"amount": 15000.50}` -> Wajib gagal (`ERR_PLUGIN_FLOAT_VIOLATION`).
   - Menyuapkan payload dengan pecahan nol: `{"amount": 15000.0}` -> Wajib gagal (`num.is_f64()` ditolak tanpa syarat).
   - Menyuapkan notasi ilmiah: `{"amount": 1e2}` dan `{"amount": 1e20}` -> Wajib gagal (`ERR_PLUGIN_FLOAT_VIOLATION`).
   - Menyuapkan string angka pecahan: `{"amount": "15000.50"}` -> Wajib gagal (`ERR_PLUGIN_FLOAT_VIOLATION`).
   - Menyuapkan string non-finit: `{"amount": "NaN"}`, `{"trend": "Infinity"}` -> Wajib gagal (`ERR_PLUGIN_FLOAT_VIOLATION`).
   - Menyuapkan integer valid: `{"amount": 1500000, "trend_basis_points": 520}` -> Wajib sukses (`Ok(())`).
2. **Uji Penegakan Invarian Debit-First (*Debit-First Enforcement Test*)**:
   - Menyuapkan draf 2-kaki standar: `[Debit: +10000, Credit: -10000]` -> Lulus verifikasi (`Ok(())`).
   - Menyuapkan draf terbalik: `[Credit: -10000, Debit: +10000]` -> Ditolak keras oleh host gateway dengan kode `ERR_PLUGIN_INVARIANT_DEBIT_FIRST`.
   - Menyuapkan draf tidak seimbang: `[Debit: +10000, Credit: -9000]` -> Ditolak dengan kode `ERR_PLUGIN_UNBALANCED_ENTRY`.
3. **Uji Isolasi Memori (*Out-of-Memory Trap Test*)**:
   Menjalankan modul WebAssembly yang mencoba mengalokasikan memori melebihi 32 MB (`vec![0u8; 40 * 1024 * 1024]`). Verifikasi bahwa Extism memicu trap dan host menangani error secara elegan tanpa crash.
4. **Uji Batas Waktu (*Timeout / Infinite Loop Test*)**:
   Menjalankan modul WebAssembly yang memuat `loop {}`. Verifikasi bahwa host memutus eksekusi tepat setelah 5.000 milidetik dan mengembalikan `ERR_PLUGIN_TIMEOUT`.

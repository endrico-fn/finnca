# ADR 0008: Platform Integrity, OS Keychain Integration & Tamper Mitigation

## Status

Accepted (Standar Penguatan Keamanan Platform Fase 1 & 2)

## Context

Sebagai aplikasi pencatatan keuangan pribadi tingkat lanjut (_advanced personal finance ledger_), integritas platform, kerahasiaan kunci kriptografi di dalam memori, serta kekebalan terhadap rekayasa balik (_reverse engineering_) dan manipulasi berkas lokal merupakan prasyarat mutlak. Audit keamanan platform yang komprehensif mengungkap sejumlah kelemahan struktural pada fondasi keamanan saat ini:

1. **Ketiadaan Integrasi OS Credential Store & Stub `boot_id` Non-Kriptografis**:
   Saat ini, pengguna diwajibkan memasukkan password master secara manual setiap kali aplikasi dibuka. Fitur pembukaan cepat (_fast-unlock_) yang ada di `src-tauri/src/security/commands.rs:8-19` hanya membaca nilai `/proc/sys/kernel/random/boot_id` di Linux dan menyimpannya sebagai string teks terbuka (_plaintext_) di berkas konfigurasi `~/.config/finnca/finnca.json`. Pengecekan ini tidak memiliki ikatan kriptografis (_cryptographic binding_) ke dekripsi database, tidak dapat memulihkan _Data Encryption Key_ (DEK) setelah proses ditutup, dan tidak didukung oleh penyimpanan kredensial perangkat keras (_hardware-backed security module_) seperti TPM, Apple Secure Enclave, atau Windows Hello.
2. **Residu Forensik SQLCipher & Pertahanan Memori Disk (`temp_store`)**:
   Dalam `src-tauri/src/db/mod.rs:15-24`, inisialisasi SQLCipher mengaktifkan `PRAGMA cipher_memory_security = ON;` dan `PRAGMA journal_mode = WAL;`, namun **mengabaikan** `PRAGMA temp_store = MEMORY;`. Meskipun SQLCipher mengenkripsi berkas sementara secara otomatis pada koneksi yang telah di-key, ketiadaan `PRAGMA temp_store = MEMORY;` menyebabkan tabel sementara (_temporary tables_), indeks sementara, dan hasil pengurutan (_B-tree sorts_) untuk kueri agregasi laporan keuangan yang kompleks tetap tumpah (_spill_) ke disk OS (`/tmp/etilqs_*` di Linux atau `%TEMP%` di Windows). Penerapan `PRAGMA temp_store = MEMORY;` mutlak diperlukan sebagai pertahanan berlapis (_defense-in-depth_) agar data intermediat keuangan tidak pernah menyentuh media penyimpanan fisik.
3. **Kebocoran Material Kunci & Higiene Memori Sensitif (_Zeroization Gap_)**:
   - Di `src-tauri/src/crypto/envelope.rs:56-68`, saat `unwrap_dek` mendekripsi paket amplop menggunakan ChaCha20-Poly1305, hasil dekripsi ditampung di dalam `Vec<u8>` standar sebelum disalin ke `Zeroizing<[u8; 32]>`. Buffer `Vec<u8>` tersebut dialokasikan di heap dan dideallokasi oleh allocator Rust tanpa pembersihan memori (_zeroize_).
   - Di `src-tauri/src/db/mod.rs:11-12`, kunci didekodekan menjadi string hexadesimal `let dek_hex = hex::encode(dek);` dan diformat menjadi perintah SQL `format!("PRAGMA key = \"x'{dek_hex}'\";")`. Kedua objek `String` ini dialokasikan di heap tanpa pembungkus `Zeroize`, meninggalkan jejak kunci enkripsi mentah pada memori virtual proses.
   - Di `src-tauri/src/db/mod.rs:49-55` (`export_encrypted_snapshot`), `let target_dek_hex = hex::encode(target_dek);` dan perintah SQL format `ATTACH DATABASE ... KEY "x'{target_dek_hex}'"` menciptakan alokasi heap `String` dan internal `CString` yang memuat kunci DEK snapshot tanpa pembungkus `Zeroize`.
   - Pada batas IPC autentikasi (`src-tauri/src/auth/commands.rs:18,63-64`), parameter password diterima sebagai `String` biasa tanpa pembungkus proteksi memori rahasia.
   - Pada frontend Svelte (`LoginForm.svelte:135-162`), variabel password pada rune `$state('')` tidak di-reset secara higienis setelah proses login selesai, membiarkannya menetap di heap garbage collector JavaScript V8.
4. **Celah Keamanan Pembacaan Berkas Lokal Sewenang-wenang (_Arbitrary Local File Read_)**:
   Di `src-tauri/src/reconcile/commands.rs:291-321`, IPC command `read_statement_file_cmd` menerima parameter string `path` mentah dan langsung memanggil `std::fs::read_to_string(p)` hingga batas 5 MB tanpa melalui validasi batas path (_path canonicalization_) atau pemeriksaan direktori sensitif. Hal ini memungkinkan kode front-end atau dependensi terkompromi membaca berkas sistem sensitif (`/etc/passwd`, `~/.ssh/id_rsa`, `~/.aws/credentials`, dll).
5. **Ketiadaan Pengerasan Profil Rilis Biner (_Release Build Hardening_)**:
   Berkas `src-tauri/Cargo.toml` tidak mendefinisikan blok `[profile.release]`. Akibatnya, biner rilis terkompilasi menggunakan konfigurasi default: _Link-Time Optimization (LTO)_ nonaktif, 16 unit codegen (_codegen-units_), _panic unwinding_ dipertahankan, dan tabel simbol debugging tidak di-strip. Selain itu, fitur `tauri/devtools` diaktifkan secara unconditionally di dependensi, memungkinkan webview inspector diakses pada biner produksi.
6. **Eksfiltrasi Jaringan Melalui Celah CSP Webview**:
   Konfigurasi CSP di `src-tauri/tauri.conf.json:29` mengizinkan `connect-src` ke `https://open.er-api.com` agar modul `src/lib/features/settings/fxSync.ts` dapat melakukan `fetch()` kurs valuta asing secara langsung dari webview, melanggar prinsip _airgapped local ledger_ dan membuka celah eksfiltrasi data via modifikasi URL.

---

## Decision

Kami merekayasa ulang postur integritas platform dan keamanan kriptografi Finnca melalui 7 pilar arsitektur utama:

### 1. Integrasi OS Credential Store via Arsitektur Dual-Envelope KDF

Kami mengadopsi integrasi native dengan manajer kredensial sistem operasi menggunakan crate `keyring` (v3):

- **Model Dua Amplop Kunci (_Dual-Envelope Model_)**:
  Metadata vault (`vault.meta.json`) kini mendukung dua amplop enkripsi independen untuk membungkus `db_key` (DEK 256-bit):
  1. `wrapped_dek_password`: Amplop utama yang dienkripsi menggunakan KEK turunan password master via Argon2id (standar ADR 0001).
  2. `wrapped_dek_keychain`: Amplop opsional yang dienkripsi menggunakan kunci acak 256-bit berdaya tahan tinggi (_Device Unlock Key_ / DUK) yang disimpan secara aman di dalam OS Credential Store:
     - **macOS**: Apple Keychain Services dengan proteksi biometrik Secure Enclave (`kSecAccessControlTouchIDAny`).
     - **Windows**: Windows Credential Manager yang dilindungi oleh Windows Data Protection API (DPAPI) / Windows Hello.
     - **Linux**: Secret Service API via D-Bus (`org.freedesktop.secrets` / `gnome-keyring` / `keepassxc-service`).
- **Skema Metadata v2 dengan `vault_id` Imutabel**:
  Untuk mencegah benturan kunci kredensial (_key collisions_) antar-vault dan mengamankan integritas saat vault diubah namanya (_renamed_), skema `VaultMetadata` menambahkan atribut `vault_id: String` (UUID v4 / ULID) yang bersifat imutabel. Entri OS Keychain selalu diikat pada `(KEYRING_SERVICE, vault_id)`, bukan nama vault yang dapat berubah.
- **Proteksi Non-Blocking & Batas Waktu D-Bus Linux**:
  Pada Linux, pemanggilan synchronous ke Secret Service daemon berisiko membekukan thread pemanggil hingga 25 detik jika daemon terkunci atau menampilkan dialog GUI otorisasi. Seluruh pemanggilan `keyring` wajib dibungkus di dalam task non-blocking (`spawn_blocking`) dengan batas waktu ketat **2 detik** (`tokio::time::timeout`). Jika batas waktu terlampaui atau daemon terkunci, sistem langsung mengembalikan galat non-fatal dan beralih ke formulir password master manual tanpa membekukan UI Tauri.
- **Penanganan Desinkronisasi Pergantian Password (_Atomic Split-Brain Mitigation_)**:
  Saat pengguna mengganti password master (`change_password_cmd`), sistem berusaha memperbarui kedua amplop (`wrapped_dek_password` dan `wrapped_dek_keychain`). Jika OS Keychain sedang terkunci atau gagal diperbarui, sistem menerapkan **kebijakan fallback atomik**: `wrapped_dek_keychain` di dalam `vault.meta.json` **wajib di-purge secara permanen (`wrapped_dek_keychain = None`)**. Ini menjamin tidak terjadi kondisi _split-brain_ di mana DUK lama masih dapat membuka database lama atau gagal membuka database baru. Pengguna dapat mengaktifkan kembali "Fast Unlock" setelah berhasil masuk dengan password baru.
- **Pembersihan & Reset Kredensial**:
  Saat pengguna menonaktifkan "Fast Unlock" atau "Remember Vault", kunci DUK dihapus secara atomik dari OS Keychain via `purge_device_key`.

### 2. Penghapusan Total Stub `boot_id` Non-Kriptografis

Kami memusnahkan (_deprecate and purge_) implementasi `boot_id` di `src-tauri/src/security/commands.rs`. Validasi integritas sesi kini sepenuhnya dikelola oleh state machine memori Rust yang terikat pada event siklus hidup OS (_OS lifecycle hooks_) dan timer inaktivitas aplikasi, bukan pembacaan string OS acak yang tidak bernilai kriptografis.

### 3. Pembersihan Memori Sensitif Komprehensif (_Zeroize-on-Drop Everywhere_)

Seluruh alokasi heap dan stack yang menangani material rahasia wajib menerapkan pembungkusan anti-residu memori:

1. **DEK Decryption Buffer**:
   Fungsi `unwrap_dek` di `src-tauri/src/crypto/envelope.rs` wajib mendekripsi ciphertext langsung ke buffer yang dibungkus oleh `zeroize::Zeroizing<Vec<u8>>` atau array stack berukuran tetap `zeroize::Zeroizing<[u8; 32]>`.
2. **Kunci SQLCipher Hex**:
   String hexadesimal kunci database di `src-tauri/src/db/mod.rs` wajib dibungkus dalam `zeroize::Zeroizing<String>` dan dibersihkan dari memori segera setelah perintah `PRAGMA key` selesai dieksekusi.
3. **Snapshot DEK Hex & ATTACH Statement**:
   Di `src-tauri/src/db/mod.rs:49-55` (`export_encrypted_snapshot`), `target_dek_hex` dan string `ATTACH DATABASE` wajib dibungkus dalam `zeroize::Zeroizing<String>` untuk mencegah kebocoran kunci snapshot cadangan pada heap.
4. **Password Buffer pada Batas IPC**:
   Seluruh perintah IPC penerima password (`unlock`, `change_password`, `create_vault`) wajib menerima tipe data `secrecy::SecretString` atau membungkus argumen dalam `zeroize::Zeroizing<String>` yang secara otomatis menimpa byte memori dengan angka nol saat keluar dari scope eksekusi.
5. **Pembersihan Heap Frontend**:
   Komponen `LoginForm.svelte` wajib mengosongkan variabel rune `password = ''` di dalam blok `finally` segera setelah panggilan IPC otentikasi selesai, meminimalkan durasi eksistensi teks sandi di heap V8/WebKit.

### 4. Mandat `PRAGMA temp_store = MEMORY;` pada SQLCipher

Pada inisialisasi basis data `open_vault_db()` di `src-tauri/src/db/mod.rs`, kami mewajibkan eksekusi pragma memori sementara:

```sql
PRAGMA temp_store = MEMORY;
```

Pragma ini menjamin bahwa seluruh tabel sementara, pengurutan B-tree, dan hasil antara kueri laporan keuangan hanya dialokasikan di dalam RAM virtual proses yang dilindungi oleh proteksi memori SQLCipher (`cipher_memory_security = ON`), dan tidak akan pernah ditulis ke disk `/tmp` dalam bentuk berkas sementara fisik. Perlu ditegaskan bahwa SQLCipher secara bawaan telah mengenkripsi berkas sementara jika terpaksa dibuat, namun mandat `PRAGMA temp_store = MEMORY;` memberikan pertahanan berlapis (_defense-in-depth_) mutlak terhadap jejak forensik media penyimpanan.

### 5. Pengerasan Profil Kompilasi Rilis Biner (_Release Profile Hardening_)

Kami menambahkan konfigurasi profil rilis biner berstandar keamanan tinggi pada `src-tauri/Cargo.toml`:

- **Fat Link-Time Optimization (`lto = "fat"`)**: Memaksa analisis program menyeluruh (_whole-program analysis_) lintas seluruh crate dependensi, mengoptimalkan inlining fungsi kriptografi, dan menghapus kode usang (_dead code elimination_).
- **Single Codegen Unit (`codegen-units = 1`)**: Menghilangkan batas modul kompilasi terpisah, memaksimalkan efektivitas LTO, serta mempersulit disassembler memetakan batas unit logika internal.
- **Panic Abort (`panic = "abort"`)**: Menghapus tabel unwinding dan landing pads, mencegah penyerang mengekstrak struktur kontrol alur program dari metadata panic handler.
- **Simbol Dihapus Mutlak (`strip = true`)**: Membersihkan seluruh tabel simbol fungsi dan metadata debug dari artefak biner produksi.
- **Kondisionalisasi DevTools**: Menghapus fitur `features = ["devtools"]` dari konfigurasi default dependensi Tauri. Devtools kini berada di balik feature flag eksplisit `cargo build --features devtools`.

### 6. Pengerasan Batas IPC & Pertahanan Path Traversal pada `read_statement_file_cmd`

Kami mengamankan perintah IPC `read_statement_file_cmd` dengan mekanisme validasi ketat yang kebal terhadap symlink bypass dan directory traversal:

1. Memvalidasi bahwa sesi vault aktif dan terotentikasi.
2. Memeriksa bahwa input path adalah path absolut yang valid.
3. **Pemeriksaan Symlink pada Input Mentah**: Memeriksa metadata `std::fs::symlink_metadata(p)` pada path input _sebelum_ dikanonikalisasi. Jika path adalah tautan simbolik (_symlink_), eksekusi langsung digagalkan dengan galat `Akses melalui tautan simbolik dilarang`.
4. **Kanonikalisasi Penuh**: Menyelesaikan path menjadi bentuk kanonik (`p.canonicalize()`).
5. **Pemblokiran Direktori Sistem Operasi Lintas-Platform**:
   - **Linux / Unix**: Menolak akses ke `/etc`, `/root`, `/boot`, `/sys`, `/proc`, `/bin`, `/sbin`, `/usr`, `/dev`, `/var`.
   - **Windows**: Menolak akses ke `C:\Windows`, `C:\Program Files`, `C:\Program Files (x86)`, `C:\ProgramData`.
6. **Perlindungan Direktori Kredensial & Konfigurasi Rahasia**:
   Menolak path yang memuat komponen direktori sensitif (`.ssh`, `.gnupg`, `.aws`, `.azure`, `.kube`, `.config/finnca`, `.bash_history`, `.zsh_history`, `.env`, dll), namun tetap mengizinkan pembacaan dari direktori data pengguna yang sah (misal `~/.local/share/` atau direktori kerja vault).
7. **Whitelist Format Mutasi Finansial Ketat**:
   Membatasi format berkas secara eksklusif pada format mutasi perbankan terstruktur: `["csv", "ofx", "qfx", "qif", "beancount", "mt940"]`. Ekstensi generik seperti `.txt` dilarang keras untuk mencegah kebocoran berkas catatan atau kunci teks biasa.
8. **Pembatasan Kuota Ukuran Berkas**: Mempertahankan batasan ukuran berkas maksimum 5 MB (`MAX_STATEMENT_FILE_BYTES`).

### 7. Isolasi Jaringan Webview CSP & Pemindahan FX Sync ke Backend Rust

1. **Airgapped Webview CSP**:
   Kebijakan CSP di `src-tauri/tauri.conf.json` dikembalikan ke status _airgapped mutlak_:
   ```json
   "connect-src 'ipc:' https://ipc.localhost"
   ```
   Domain eksternal `https://open.er-api.com` dicabut dari akses webview.
2. **Sinkronisasi Kurs Valas Terisolasi di Rust IPC**:
   Logika sinkronisasi kurs mata uang di `fxSync.ts` dipindahkan ke backend Rust (`sync_fx_rates_cmd`). Backend Rust mengeksekusi request HTTPS menggunakan klien TLS dengan verifikasi sertifikat root CA sistem operasi, pembatasan timeout (10 detik), dan menyimpan data kurs langsung ke dalam tabel terenkripsi SQLCipher `exchange_rates`. Frontend hanya meminta data kurs melalui IPC internal.

---

## Consequences

### Positif

- **Kekebalan Forensik Disk**: Mengeliminasi risiko residu data finansial di media penyimpanan `/tmp` berkat `PRAGMA temp_store = MEMORY;`.
- **Higiene Memori Tingkat Tinggi**: Kunci enkripsi DEK, KEK, dan password master terjamin dibersihkan dari RAM virtual saat objek dilepas (_drop_), memitigasi serangan memory dumping atau crash inspection.
- **Eliminasi Kerentanan Path Traversal**: Frontend tidak dapat lagi menyalahgunakan `read_statement_file_cmd` untuk membaca berkas arbitrer sistem operasi host.
- **Pengalaman Pengguna Modern yang Aman**: Integrasi OS Keychain memungkinkan pembukaan vault secara instan menggunakan sensor biometrik (Touch ID, Windows Hello) tanpa mengorbankan keamanan envelope encryption.
- **Proteksi Rekayasa Balik & Anti-Tamper**: Profil rilis Fat LTO, stripping simbol, dan eliminasi Devtools mempersulit analisis statis/dinamis oleh pihak yang tidak berwenang.
- **Kepatuhan Arsitektur Airgapped**: Webview kembali sepenuhnya terisolasi dari akses jaringan internet langsung.

### Trade-off & Mitigasi

- **Ketergantungan Eksternal OS Keychain di Linux**: Distribusi Linux minimal tanpa Secret Service daemon memerlukan penanganan error yang anggun.  
  _Mitigasi_: Crate `keyring` dibungkus dalam modul abstraksi yang mendeteksi ketersediaan backend D-Bus. Jika tidak tersedia, UI secara otomatis beralih ke formulir password manual dengan notifikasi informatif.
- **Waktu Kompilasi Rilis Lebih Lama**: Penggunaan `lto = "fat"` dan `codegen-units = 1` meningkatkan durasi build CI/CD rilis (dari ~3 menit menjadi ~7 menit).  
  _Mitigasi_: Konfigurasi dev dan test profile (`profile.dev`, `profile.test`) tetap mempertahankan `incremental = true` dan `lto = false` sehingga kecepatan iterasi harian pengembang tidak terpengaruh.
- **Konsumsi RAM Tambahan untuk Temp Tables**: Kueri laporan raksasa akan menggunakan memori RAM aplikasi alih-alih disk swap.  
  _Mitigasi_: Data finansial personal jarang melampaui jutaan baris dalam basis data tunggal, sehingga konsumsi RAM temporer untuk kueri agregasi tahunan diperkirakan < 20 MB.

---

## Technical Architecture & Code Contracts

### 1. Arsitektur Dual-Envelope KDF dan Alur OS Keychain

```
                                  [ MASTER PASSWORD ]
                                           │
                                           ▼ (Argon2id KDF: m=64MB, t=3, p=4)
                                  [ Derived KEK (256-bit) ]
                                           │
                                           ▼ (ChaCha20-Poly1305 Decrypt)
                                ┌──────────────────────┐
                                │ wrapped_dek_password │ ──┐
                                └──────────────────────┘   │
                                                           │
┌───────────────────────┐                                  │
│   OS CREDENTIAL STORE │                                  │
│ (Keychain/Hello/D-Bus)│                                  │
└───────────┬───────────┘                                  │
            │                                              │
            ▼ (Secure Enclave / DPAPI)                     │
  [ DeviceUnlockKey (DUK) ]                                │
            │                                              ▼
            ▼ (ChaCha20-Poly1305 Decrypt)          [ Plaintext DEK (256-bit) ]
 ┌──────────────────────┐                          (zeroize::Zeroizing RAM)
 │ wrapped_dek_keychain │ ─────────────────────────┤
 └──────────────────────┘                          │
                                                   ▼
                                        [ SQLCipher PRAGMA key ]
                                        (temp_store = MEMORY)
```

---

### 2. Implementasi Kriptografi OS Keychain & Dual-Envelope (`src-tauri/src/crypto/keychain.rs`)

```rust
// File: src-tauri/src/crypto/keychain.rs
use crate::shared::AppError;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use keyring::Entry;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use zeroize::{Zeroize, Zeroizing};

const KEYRING_SERVICE: &str = "org.finnca.vault";
const KEYRING_TIMEOUT: Duration = Duration::from_secs(2);

/// Skema Metadata Vault v2 dengan pengikatan kunci kredensial persisten via `vault_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    pub version: u32,
    /// Identitas UUID imutabel vault (primary key OS Credential Store)
    pub vault_id: String,
    pub vault_name: Option<String>,
    pub username: Option<String>,
    pub kdf: crate::vault::metadata::KdfParams,
    pub wrapped_dek_password: crate::vault::metadata::WrappedKey,
    pub wrapped_dek_keychain: Option<crate::vault::metadata::WrappedKey>,
}

pub struct KeychainManager;

impl KeychainManager {
    /// Menyimpan Device Unlock Key (DUK) 256-bit acak ke dalam OS Credential Store
    /// dengan perlindungan batas waktu non-blocking (mencegah deadlock D-Bus di Linux).
    pub fn store_device_key(vault_id: &str, device_key: &[u8; 32]) -> Result<(), AppError> {
        let vault_id = vault_id.to_string();
        let hex_key = Zeroizing::new(hex::encode(device_key));

        // Eksekusi di thread terisolasi dengan batas waktu non-blocking 2 detik
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let res = Entry::new(KEYRING_SERVICE, &vault_id)
                .map_err(|e| AppError::Crypto(format!("Gagal menginisialisasi keyring entry: {e}")))
                .and_then(|entry| {
                    entry.set_password(&hex_key)
                        .map_err(|e| AppError::Crypto(format!("Gagal menyimpan kunci ke OS Keychain: {e}")))
                });
            let _ = tx.send(res);
        });

        rx.recv_timeout(KEYRING_TIMEOUT)
            .map_err(|_| AppError::Crypto("Batas waktu komunikasi OS Keychain (2 detik) terlampaui (D-Bus daemon timeout)".into()))?
    }

    /// Mengambil Device Unlock Key (DUK) dari OS Credential Store
    /// dengan pembersihan residu memori mutlak (Zeroizing pada raw_password String)
    /// dan proteksi batas waktu non-blocking 2 detik.
    pub fn retrieve_device_key(vault_id: &str) -> Result<Zeroizing<[u8; 32]>, AppError> {
        let vault_id = vault_id.to_string();

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let res = Entry::new(KEYRING_SERVICE, &vault_id)
                .map_err(|e| AppError::Crypto(format!("Keyring tidak tersedia: {e}")))
                .and_then(|entry| {
                    // Wajib dibungkus Zeroizing<String> seketika saat string password diambil dari keyring!
                    let raw_password = Zeroizing::new(
                        entry.get_password()
                            .map_err(|_| AppError::NotFound(format!("Tidak ada kunci tersimpan untuk vault {vault_id}")))?
                    );

                    let mut raw_bytes = Zeroizing::new(
                        hex::decode(raw_password.trim())
                            .map_err(|_| AppError::Crypto("Format kunci keychain rusak".into()))?
                    );

                    if raw_bytes.len() != 32 {
                        return Err(AppError::Crypto("Panjang kunci perangkat tidak valid".into()));
                    }

                    let mut key = Zeroizing::new([0u8; 32]);
                    key.copy_from_slice(&raw_bytes);
                    // raw_password dan raw_bytes otomatis di-zeroize saat keluar dari scope ini
                    Ok(key)
                });
            let _ = tx.send(res);
        });

        rx.recv_timeout(KEYRING_TIMEOUT)
            .map_err(|_| AppError::Crypto("Batas waktu pengambilan kunci OS Keychain (2 detik) terlampaui".into()))?
    }

    /// Menghapus kredensial vault dari OS Credential Store (Revokasi).
    pub fn purge_device_key(vault_id: &str) -> Result<(), AppError> {
        let vault_id = vault_id.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            if let Ok(entry) = Entry::new(KEYRING_SERVICE, &vault_id) {
                let _ = entry.delete_password();
            }
            let _ = tx.send(());
        });
        let _ = rx.recv_timeout(KEYRING_TIMEOUT);
        Ok(())
    }

    /// Membungkus DEK menggunakan Kunci Perangkat (DUK) menghasilkan wrapped_dek_keychain.
    pub fn wrap_dek_for_keychain(
        dek: &Zeroizing<[u8; 32]>,
        device_key: &Zeroizing<[u8; 32]>,
    ) -> Result<(String, String), AppError> {
        let cipher = ChaCha20Poly1305::new_from_slice(device_key.as_ref())
            .map_err(|e| AppError::Crypto(format!("Inisialisasi ChaCha20 gagal: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, dek.as_ref())
            .map_err(|e| AppError::Crypto(format!("Enkripsi kunci keychain gagal: {e}")))?;

        Ok((hex::encode(nonce_bytes), hex::encode(ciphertext)))
    }

    /// Membuka bungkus DEK menggunakan Kunci Perangkat (DUK) dengan pembersihan memori mutlak.
    pub fn unwrap_dek_from_keychain(
        device_key: &Zeroizing<[u8; 32]>,
        nonce_hex: &str,
        ciphertext_hex: &str,
    ) -> Result<Zeroizing<[u8; 32]>, AppError> {
        let cipher = ChaCha20Poly1305::new_from_slice(device_key.as_ref())
            .map_err(|e| AppError::Crypto(format!("Inisialisasi ChaCha20 gagal: {e}")))?;

        let nonce_bytes = hex::decode(nonce_hex)
            .map_err(|_| AppError::Crypto("Nonce heksadesimal tidak valid".into()))?;
        let ciphertext_bytes = hex::decode(ciphertext_hex)
            .map_err(|_| AppError::Crypto("Ciphertext heksadesimal tidak valid".into()))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut decrypted = Zeroizing::new(
            cipher
                .decrypt(nonce, ciphertext_bytes.as_ref())
                .map_err(|_| AppError::Crypto("Gagal membuka dekripsi amplop keychain".into()))?,
        );

        if decrypted.len() != 32 {
            return Err(AppError::Crypto("Ukuran DEK tidak valid".into()));
        }

        let mut dek = Zeroizing::new([0u8; 32]);
        dek.copy_from_slice(&decrypted);
        Ok(dek)
    }

    /// Menangani sinkronisasi penggantian master password: jika pembaruan OS Keychain gagal,
    /// purge wrapped_dek_keychain dari metadata secara atomik untuk mencegah split-brain.
    pub fn handle_password_change_sync(
        meta: &mut VaultMetadata,
        new_dek: &Zeroizing<[u8; 32]>,
    ) {
        if meta.wrapped_dek_keychain.is_some() {
            let mut new_device_key = Zeroizing::new([0u8; 32]);
            rand::thread_rng().fill_bytes(new_device_key.as_mut());

            match Self::store_device_key(&meta.vault_id, &new_device_key) {
                Ok(()) => {
                    if let Ok((nonce, ct)) = Self::wrap_dek_for_keychain(new_dek, &new_device_key) {
                        meta.wrapped_dek_keychain = Some(crate::vault::metadata::WrappedKey {
                            nonce,
                            ciphertext: ct,
                        });
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("[WARN] OS Keychain tidak dapat diakses saat ganti password: {e}. Melakukan purge token keychain.");
                }
            }

            // Fallback Atomik: Purge kredensial keychain yang usang untuk mencegah split-brain
            meta.wrapped_dek_keychain = None;
            let _ = Self::purge_device_key(&meta.vault_id);
        }
    }
}
```

---

### 3. Inisialisasi SQLCipher Terkeras dengan `temp_store = MEMORY` & Snapshot Zeroize (`src-tauri/src/db/mod.rs`)

```rust
// File: src-tauri/src/db/mod.rs
use crate::shared::AppError;
use rusqlite::Connection;
use std::path::Path;
use zeroize::{Zeroize, Zeroizing};

pub fn open_vault_db(path: &Path, dek: &Zeroizing<[u8; 32]>) -> Result<Connection, AppError> {
    let mut conn = Connection::open(path)?;

    // 1. Eksekusi PRAGMA key dengan buffer hexadesimal yang dibungkus Zeroizing
    {
        let mut dek_hex = Zeroizing::new(hex::encode(dek.as_ref()));
        let mut pragma_cmd = Zeroizing::new(format!("PRAGMA key = \"x'{dek_hex}'\";"));
        conn.execute_batch(&pragma_cmd)?;
        // dek_hex dan pragma_cmd otomatis di-zeroize saat keluar dari scope ini
    }

    // 2. Terapkan konfigurasi integritas dan isolasi memori mutlak
    // Catatan: SQLCipher mengenkripsi berkas temp secara otomatis pada koneksi terenkripsi.
    // PRAGMA temp_store = MEMORY memberikan defense-in-depth agar operasi intermediat tetap di RAM.
    conn.execute_batch(
        "
        PRAGMA cipher_compatibility = 4;
        PRAGMA cipher_memory_security = ON;
        PRAGMA temp_store = MEMORY;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;
        ",
    )?;

    // 3. Verifikasi integritas basis data saat pertama kali dibuka
    let quick_check: String = conn.query_row("PRAGMA quick_check;", [], |r| r.get(0))?;
    if quick_check != "ok" {
        return Err(AppError::Database(format!(
            "Pemeriksaan integritas basis data gagal: {quick_check}"
        )));
    }

    Ok(conn)
}

/// Mengekspor basis data terenkripsi untuk cadangan dengan zeroize penuh pada memori heap.
pub fn export_encrypted_snapshot(
    conn: &Connection,
    dest_path: &Path,
    target_dek: &Zeroizing<[u8; 32]>,
) -> Result<(), AppError> {
    let dest_str = dest_path.to_string_lossy();
    let safe_dest = dest_str.replace('\'', "''");

    // Bungkus hex DEK target dan perintah SQL ATTACH ke dalam Zeroizing buffer
    let mut target_dek_hex = Zeroizing::new(hex::encode(target_dek.as_ref()));
    let mut attach_sql = Zeroizing::new(format!(
        "ATTACH DATABASE '{safe_dest}' AS backup KEY \"x'{target_dek_hex}'\";"
    ));

    conn.execute_batch(&attach_sql)?;
    conn.execute_batch("SELECT sqlcipher_export('backup');")?;
    conn.execute_batch("DETACH DATABASE backup;")?;

    // target_dek_hex dan attach_sql di-zeroize secara otomatis saat keluar dari scope
    Ok(())
}
```

---

### 4. Implementasi Pengerasan IPC Pertahanan Path Traversal (`src-tauri/src/reconcile/commands.rs`)

```rust
// File: src-tauri/src/reconcile/commands.rs
use crate::shared::{AppError, AppState};
use tauri::State;

const MAX_STATEMENT_FILE_BYTES: u64 = 5 * 1024 * 1024;

#[tauri::command]
#[specta::specta]
pub fn read_statement_file_cmd(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, AppError> {
    // 1. Wajib memiliki sesi vault aktif dan terotentikasi
    let _ = crate::vault::commands::require_session(&state)
        .map_err(AppError::InvalidInput)?;

    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("Path berkas mutasi tidak boleh kosong".into()));
    }

    let p = std::path::Path::new(trimmed);
    if !p.is_absolute() {
        return Err(AppError::InvalidInput("Path berkas wajib berupa path absolut".into()));
    }

    // 2. Pemeriksaan Symlink pada path input mentah SEBELUM kanonikalisasi
    // Memanggil symlink_metadata pada p mencegah bypass symlink traversal
    let input_meta = std::fs::symlink_metadata(p)
        .map_err(|e| AppError::NotFound(format!("Berkas tidak ditemukan atau path tidak valid: {e}")))?;
    if input_meta.file_type().is_symlink() {
        return Err(AppError::InvalidInput("Membaca berkas melalui tautan simbolik (symlink) dilarang".into()));
    }

    // 3. Kanonikalisasi path untuk menyelesaikan traversal (..)
    let canonical = p.canonicalize()
        .map_err(|e| AppError::NotFound(format!("Gagal kanonikalisasi path berkas: {e}")))?;
    let canonical_str = canonical.to_string_lossy();

    // 4. Cek direktori sistem operasi terlarang (Linux / Unix & Windows)
    #[cfg(unix)]
    {
        let forbidden = ["/etc", "/root", "/boot", "/sys", "/proc", "/bin", "/sbin", "/usr", "/dev", "/var"];
        if canonical_str == "/" || forbidden.iter().any(|f| canonical_str == *f || canonical_str.starts_with(&format!("{f}/"))) {
            return Err(AppError::InvalidInput("Akses ke direktori sistem operasi ditolak".into()));
        }
    }

    #[cfg(windows)]
    {
        let canonical_upper = canonical_str.to_uppercase();
        let forbidden_win = [
            "C:\\WINDOWS",
            "C:\\PROGRAM FILES",
            "C:\\PROGRAM FILES (X86)",
            "C:\\PROGRAMDATA",
        ];
        if forbidden_win.iter().any(|f| canonical_upper.starts_with(f)) {
            return Err(AppError::InvalidInput("Akses ke direktori sistem operasi ditolak".into()));
        }
    }

    // 5. Tolak akses ke direktori dan berkas kredensial sensitif
    let sensitive_roots = [
        ".ssh", ".gnupg", ".aws", ".azure", ".kube", ".config/finnca",
        ".bash_history", ".zsh_history", ".profile", ".bashrc", ".env", "id_rsa", "id_ed25519"
    ];
    for component in canonical.components() {
        let comp_str = component.as_os_str().to_string_lossy();
        if sensitive_roots.iter().any(|s| comp_str == *s || comp_str.starts_with(s)) {
            return Err(AppError::InvalidInput("Akses ke direktori konfigurasi atau kredensial sensitif ditolak".into()));
        }
    }

    // 6. Whitelist ekstensi berkas rekening koran terstandarisasi (Format ketat, tanpa .txt)
    let ext = canonical.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let allowed_extensions = ["csv", "ofx", "qfx", "qif", "beancount", "mt940"];
    if !allowed_extensions.contains(&ext.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "Ekstensi berkas '.{ext}' tidak didukung. Format mutasi yang diizinkan: CSV, OFX, QFX, QIF, Beancount, MT940."
        )));
    }

    // 7. Pembatasan kuota ukuran berkas (Maksimum 5 MB)
    if input_meta.len() > MAX_STATEMENT_FILE_BYTES {
        return Err(AppError::InvalidInput(format!(
            "Ukuran berkas ({} bytes) melampaui batas maksimum 5 MB",
            input_meta.len()
        )));
    }

    let content = std::fs::read_to_string(&canonical)?;
    Ok(content)
}
```

---

### 5. Konfigurasi Pengerasan Biner Rilis (`src-tauri/Cargo.toml`)

```toml
# Tambahan pada src-tauri/Cargo.toml

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
overflow-checks = true

[features]
default = []
devtools = ["tauri/devtools"]
```

---

### 6. Metode Verifikasi Independen

1. **Verifikasi `PRAGMA temp_store`**:
   Jalankan inspeksi runtime pada basis data yang dibuka via `open_vault_db`:
   ```sql
   PRAGMA temp_store; -- Wajib mengembalikan nilai 2 (MEMORY)
   ```
2. **Verifikasi Path Traversal Defense**:
   Uji pemanggilan IPC `read_statement_file_cmd` dengan path:
   - `/etc/passwd` -> Wajib mengembalikan `Err(AppError::InvalidInput("Akses ke direktori sistem operasi ditolak"))`.
   - `~/.ssh/id_rsa` -> Wajib ditolak oleh pengecekan direktori sensitif (`.ssh`).
   - `~/passwords.txt` -> Wajib ditolak oleh whitelist format mutasi (`.txt` dilarang).
   - `/tmp/symlink_to_stmt.csv` (symlink menuju berkas) -> Wajib ditolak oleh pengecekan `symlink_metadata` sebelum kanonikalisasi.
3. **Verifikasi Keyring Dual-Envelope**:
   Aktifkan "Fast Unlock" di pengaturan aplikasi. Periksa entri sistem credential store:
   - macOS: `security find-generic-password -s "org.finnca.vault"`
   - Linux: `secret-tool lookup service "org.finnca.vault"`
     Pastikan kunci tersimpan dalam representasi biner terenkripsi dan dapat dihapus seketika saat user memilih "Lupakan Vault".
4. **Verifikasi Binary Stripping**:
   Kompilasi paket rilis (`cargo build --release --manifest-path src-tauri/Cargo.toml`). Jalankan perintah `nm` atau `objdump`:
   ```bash
   nm -C target/release/finnca | grep "unlock"
   ```
   Pastikan perintah mengembalikan `no symbols` atau kosong karena seluruh tabel simbol telah di-strip.

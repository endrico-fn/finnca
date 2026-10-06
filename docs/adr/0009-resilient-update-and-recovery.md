# ADR 0009: Resilient Update Lifecycle, Release Channels & Crash Recovery

## Status

Accepted (Standar Rekayasa Distribusi & Pemulihan Crash Fase 1 & 2)

## Context

Sebagai aplikasi pembukuan finansial mandiri (*self-sovereign financial ledger*), Finnca bergantung pada integritas mekanisme distribusi biner dan siklus hidup pembaruan (*update lifecycle*). Pembaruan peranti lunak harus kebal terhadap pembajakan jaringan (*man-in-the-middle*), manipulasi repositori, kegagalan migrasi skema basis data, dan siklus kegagalan peluncuran (*crash loop*).

Saat ini, aplikasi menggunakan `tauri-plugin-updater` dengan verifikasi kriptografi Minisign Ed25519 untuk pembaruan in-app. Namun, audit rekayasa distribusi mengungkap beberapa kelemahan kritis:

1. **Fallback Berbahaya pada Otomasi CI/CD Release Pipeline**:
   Dalam workflow GitHub Actions `.github/workflows/release.yml:89-98`, terdapat blok fallback permisif:
   ```javascript
   if (!signingKey.trim()) {
     console.log('NOTE: TAURI_SIGNING_PRIVATE_KEY is not configured... Disabling updater artifact generation');
     delete tauri.plugins.updater.pubkey;
   }
   ```
   Jika secret `TAURI_SIGNING_PRIVATE_KEY` tidak sengaja terhapus atau gagal dibaca oleh runner, pipeline akan secara diam-diam menghapus kunci publik dari `tauri.conf.json` dan merilis artefak biner tanpa tanda tangan Minisign alih-alih membatalkan workflow. Ini merupakan risiko keamanan rantai pasok (*supply chain risk*) yang tidak dapat diterima.
2. **Ketiadaan Verifikasi Kriptografis pada Skrip Distribusi Terminal (*Shell & PowerShell*)**:
   Skrip instalasi dan pembaruan CLI di `scripts/finnca.sh` (baris 331, 355, 380) dan `scripts/finnca.ps1` (baris 119, 136) mengunduh biner rilis (`.AppImage`, `.deb`, `.rpm`, `_x64-setup.exe`) langsung dari URL GitHub Releases via `curl` atau `Invoke-WebRequest` tanpa memvalidasi checksum SHA-256 maupun tanda tangan Minisign sebelum dieksekusi. Kompromi pada jaringan pengguna atau DNS dapat menyebabkan eksekusi biner palsu dengan hak akses pengguna lokal.
3. **Ketiadaan Watchdog Peluncuran & Rollback Otomatis Saat Crash**:
   Jika versi biner baru mengalami kepanikan runtime (*unhandled panic*), regresi dependensi dinamis (misal: inkompatibilitas pustaka WebKitGTK/GLib di Linux), atau kegagalan inisialisasi pada saat booting awal (10–15 detik pertama), aplikasi akan langsung tertutup. Pengguna terjebak dalam *infinite crash loop* tanpa kemampuan untuk secara otomatis memulihkan biner versi sebelumnya.
4. **Non-Transaksionalitas pada Migrasi Skema Basis Data SQLCipher**:
   Di `src-tauri/src/db/schema.rs:18-82`, 12 migrasi skema dijalankan satu per satu menggunakan `conn.execute_batch(sql)` dan `PRAGMA user_version = N;` tanpa dibungkus dalam blok transaksi eksplisit (`BEGIN IMMEDIATE ... COMMIT`). Jika terjadi interupsi daya (*power failure*) atau crash di tengah eksekusi DDL, kolom atau tabel baru telah terbentuk di SQLite tetapi `user_version` belum bertambah. Peluncuran ulang berikutnya akan menjalankan kembali migrasi yang sama, memicu galat fatal `duplicate column name` dan mengunci pengguna keluar dari vault mereka.
5. **Keterbatasan Endpoint Tunggal & Ketiadaan Pembaruan Offline (*Airgapped Environment*)**:
   Konfigurasi `src-tauri/tauri.conf.json` mengunci endpoint pembaruan ke satu URL statis (`.../releases/latest/download/latest.json`). Tidak ada dukungan untuk kanal pengujian (*Beta* atau *Nightly*). Lebih jauh lagi, pengguna pada workstation berkeamanan tinggi yang terisolasi dari internet (*airgapped vault*) tidak memiliki mekanisme untuk menerapkan pembaruan secara manual via berkas paket terverifikasi (`.finnca-pkg`).

---

## Decision

Kami merekayasa ulang seluruh siklus hidup pembaruan, distribusi biner, dan mekanisme pemulihan crash melalui 6 ketetapan arsitektur:

### 1. Saluran Distribusi Multi-Channel Dinamis (*Stable, Beta, Nightly*)

Kami memisahkan kanal rilis ke dalam tiga jalur terisolasi dengan manifest JSON independen:
- **Stable**: `https://github.com/endrico-fn/finnca/releases/latest/download/latest.json`  
  Rilis yang telah melewati seluruh quality gate dan uji stabilitas.
- **Beta**: `https://github.com/endrico-fn/finnca/releases/download/beta/latest-beta.json`  
  Rilis pra-peluncuran untuk validasi fitur baru oleh pengguna komunitas.
- **Nightly**: `https://github.com/endrico-fn/finnca/releases/download/nightly/latest-nightly.json`  
  Build otomatis harian dari cabang utama (`main`) untuk pengujian regresi cepat.

Konfigurasi preferensi kanal disimpan di berkas konfigurasi lokal `~/.config/finnca/finnca.json` (`"update_channel": "stable"`). Modul pengecek pembaruan di frontend (`src/lib/core/updater/updateChecker.ts`) menyelesaikan URL endpoint secara dinamis berdasarkan preferensi pengguna sebelum memanggil plugin updater Tauri.

### 2. Kebijakan Tanda Tangan CI/CD Tanpa Toleransi (*Zero-Fallback Policy*)

Kami memusnahkan (*purge*) seluruh skrip fallback yang menonaktifkan tanda tangan di `.github/workflows/release.yml`. Jika `TAURI_SIGNING_PRIVATE_KEY` tidak tersedia di lingkungan CI/CD, proses build rilis **wajib langsung digagalkan (*fail-fast with exit code 1*)**:
```yaml
- name: Verify Signing Credentials
  run: |
    if [ -z "${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}" ]; then
      echo "FATAL: TAURI_SIGNING_PRIVATE_KEY secret is missing! Aborting release build."
      exit 1
    fi
```
Tidak ada biner rilis resmi yang boleh diterbitkan ke publik tanpa tanda tangan kriptografis Minisign Ed25519 yang valid.

### 3. Pengerasan Skrip Distribusi Terminal (*Mandatory Minisign Verification Gateway*)

Skrip instalasi dan pembaruan CLI (`scripts/finnca.sh` di Linux dan `scripts/finnca.ps1` di Windows) diwajibkan melakukan validasi kriptografis mutlak tanpa fallback permisif:
1. Mengunduh berkas biner target beserta manifes `SHA256SUMS` dan berkas tanda tangan digitalnya `SHA256SUMS.minisig`.
2. **Validasi Tanda Tangan Minisign Wajib (Zero-Unauthenticated-Fallback)**:
   - Skrip memverifikasi integritas dan keaslian `SHA256SUMS` menggunakan tanda tangan Minisign Ed25519 terhadap kunci publik resmi Finnca:
     `RWSDZ0KzV4jU7j9wX8+eY1Z2f3g4h5i6j7k8l9m0n1o2p3q4r5s6t7u8`
   - Dilarang keras melakukan *silent fallback* ke pengecekan hash tanpa tanda tangan jika `minisign` tidak ada! Jika utilitas `minisign` belum terpasang:
     - Di Linux (`scripts/finnca.sh`): Skrip mencoba memverifikasi via OpenSSL 3.x Ed25519 (`openssl pkeyutl -verify`) atau mengunduh biner mandiri `minisign` yang terverifikasi. Jika gagal, instalasi **wajib langsung dibatalkan (*fail-fast with exit code 1*)** dengan pesan instruksi kepada pengguna.
     - Di Windows (`scripts/finnca.ps1`): Skrip memverifikasi tanda tangan menggunakan utilitas `minisign.exe` atau modul kriptografi .NET Ed25519 sebelum mengeksekusi installer NSIS.
3. Memvalidasi hash SHA-256 dari biner yang diunduh terhadap nilai hash di dalam `SHA256SUMS` yang telah terverifikasi.
4. Jika hash tidak cocok atau berkas rusak, skrip segera membatalkan proses instalasi, menghapus seluruh berkas sementara, dan keluar dengan galat fatal.
5. Proses instalasi biner ke direktori target dilakukan secara atomik menggunakan operasi `mv` / `Move-Item`.

### 4. Protokol Watchdog Startup & Rollback Atomik (*Startup Crash Watchdog*)

Untuk menjamin aplikasi kebal terhadap *crash loop* pasca-pembaruan, kami menetapkan protokol state machine watchdog berbasis berkas penanda boot (`~/.config/finnca/.boot_state`):

```
                   [ PEMBARUAN DITERAPKAN ]
                              │
                              ▼
        Salin biner lama: <exe_path> -> <exe_path>.backup
        Tulis state: { target_version: "X.Y.Z", attempts: 0 }
                              │
                              ▼
                   [ LAUNCH BINER BARU ]
                              │
                              ▼
                   Baca .boot_state: attempts += 1
                              │
             ┌────────────────┼────────────────┐
             ▼                ▼                ▼
   [ BERJALAN STABIL >=15s ] [ KELUAR BERSIH ] [ CRASH / ABORT <15s ]
             │                │                │
             │                │                ▼
             │                │        Peluncuran ulang:
             │                │        Jika attempts >= 2:
             │                │        --> Rollback Atomik Lintas-Platform:
             │                │            - Linux: unlink running exe -> restore backup
             │                │            - Windows: rename exe -> restore backup
             │                │            - AppImage: rollback $APPIMAGE outer file
             │                │            Hapus .boot_state & Notifikasi pemulihan
             ▼                ▼
   Hapus <exe_path>.backup   Tandai stable/bersih
   Hapus .boot_state         (attempts tidak bertambah)
   (Status: SUCCESSFUL UPDATE)
```

1. **Pencadangan Biner Sebelum Instalasi**:
   Sebelum biner digantikan, biner saat ini disalin ke lokasi pencadangan lokal: `<exe_path>.backup`.
2. **Pencatatan Penanda Boot**:
   Sebuah berkas JSON `.boot_state` ditulis dengan skema:
   ```json
   {
     "target_version": "0.3.0",
     "installed_at_unix": 1728123456,
     "attempts": 0,
     "confirmed_stable": false
   }
   ```
3. **Pemberian Sinyal Kestabilan (*Heartbeat Confirmation*)**:
   Aplikasi dianggap stabil jika:
   - Berjalan selama minimal **15 detik** tanpa mengalami *panic* atau *fatal signal*, ATAU
   - Pengguna berhasil membuka kunci (*unlock*) vault keuangan pertama kali.
   Ketika syarat ini terpenuhi, proses Rust menandai `confirmed_stable: true`, menghapus berkas `.boot_state`, dan menghapus berkas cadangan `<exe_path>.backup`.
4. **Hook Penutupan Bersih Pengguna (*Early Clean-Exit Hook*)**:
   Jika pengguna menutup jendela aplikasi, menekan pintasan keluar (Ctrl+Q / Cmd+Q), atau melakukan logout secara normal sebelum batas waktu 15 detik berakhir:
   Lifecycle hook Tauri (`RunEvent::Exit` / `WindowEvent::CloseRequested`) menangkap peristiwa ini sebagai penutupan sah. Watchdog menandai `confirmed_stable: true` atau membersihkan `.boot_state` sehingga penutupan aplikasi secara cepat oleh pengguna **tidak dihitung sebagai crash** dan tidak memicu rollback palsu.
5. **Mekanisme Rollback Biner Berjalan Lintas-Platform (*Cross-Platform In-Process Rollback*)**:
   Menimpa langsung biner yang sedang berjalan (`std::fs::copy`) dilarang keras karena memicu `ETXTBSY` di Linux, `ERROR_SHARING_VIOLATION` di Windows, dan `EROFS` pada squashfs AppImage. Watchdog menerapkan strategi pemulihan spesifik platform:
   - **Pada Linux (Standar ELF)**: Kernel Linux mengizinkan pelepasan tautan (*unlink*) direktori untuk inode berkas yang sedang dieksekusi. Watchdog memanggil `std::fs::remove_file(&target_exe)` untuk menghapus entri direktori lama, kemudian memindahkan cadangan kembali via `std::fs::rename(&backup_exe, &target_exe)`. Alternatifnya, watchdog memicu helper script terpisah (`finnca-rollback.sh`) di latar belakang saat mendeteksi crash loop.
   - **Pada Windows (PE Biner)**: Windows melarang penghapusan atau penimpaan berkas yang sedang dibuka, namun mengizinkan operasi *rename* dalam volume yang sama. Watchdog menggunakan pola *rename-before-replace*:
     1. `std::fs::rename(&target_exe, target_exe.with_extension("corrupt.old"))`.
     2. `std::fs::rename(&backup_exe, &target_exe)`.
     3. Berkas `.old` dihapus pada peluncuran berikutnya atau dijadwalkan via `MoveFileExW(..., MOVEFILE_DELAY_UNTIL_REBOOT)`. Alternatifnya, helper proses detached (`finnca-rollback.cmd`) dijalankan untuk menunggu PID utama mati sebelum mengganti biner.
   - **Pada Linux AppImage**:
     Di lingkungan AppImage, `/proc/self/exe` berada di dalam mount squashfs yang read-only (`/tmp/.mount_XXXX`). Watchdog mendeteksi variabel environment `$APPIMAGE` yang menunjuk ke berkas paket AppImage fisik di disk pengguna (misal `~/.local/bin/finnca.AppImage`). Operasi rollback dilakukan terhadap berkas outer AppImage tersebut (`$APPIMAGE`), bukan mount squashfs internal.
6. **Penjagaan Crash Dynamic Linker (*Pre-Main Guard*)**:
   Kegagalan pemuatan dependensi dinamis sistem operasi (seperti ketidakcocokan glibc atau ketiadaan library runtime WebKitGTK) terjadi pada tahap pemuatan ELF sebelum fungsi Rust `main()` dieksekusi. Skrip peluncur eksternal (`scripts/finnca.sh` / desktop launcher wrapper) menyertakan validasi dependensi via `ldd` atau eksekusi probe (`--probe`) untuk mendeteksi kegagalan linker dan memulihkan biner cadangan secara otomatis di luar proses aplikasi.

### 5. Migrasi Skema Basis Data Transaksional Atomik

Kami merombak modul migrasi di `src-tauri/src/db/schema.rs`:
1. **Transaksi DDL Eksplisit (*Immediate Transaction*)**:
   Setiap versi migrasi dibungkus di dalam blok transaksi SQLite `Immediate` yang ketat:
   ```rust
   let mut tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
   tx.execute_batch(migration_sql)?;
   tx.pragma_update(None, "user_version", target_version)?;
   tx.commit()?;
   ```
   Jika terjadi interupsi daya atau kesalahan sintaksis di tengah migrasi, seluruh perubahan DDL dibatalkan secara atomik (*rollback*), sehingga integritas database tetap terjaga pada `user_version` sebelumnya.
2. **Pemeriksaan Integritas Saat Membuka Vault**:
   Setiap kali vault dibuka, aplikasi menjalankan `PRAGMA quick_check;`. Jika terdeteksi korupsi basis data, proses membuka vault dihentikan dan pengguna ditawarkan opsi pemulihan cadangan (*snapshot recovery*).
3. **Penjaga Versi Masa Depan (*Forward-Version Guard*)**:
   Jika sebuah basis data dibuka oleh biner Finnca dengan versi skema lebih rendah dari yang tercatat di `PRAGMA user_version` (`current_user_version > MAX_SUPPORTED_SCHEMA_VERSION`), aplikasi menolak membuka vault dengan pesan informatif: *"Basis data ini diperbarui oleh versi Finnca yang lebih baru. Silakan perbarui aplikasi Anda."*

### 6. Spesifikasi Berkas Pembaruan Offline Mandiri (`.finnca-pkg`)

Untuk mendukung instalasi pada komputer yang terisolasi dari internet (*airgapped network*), kami menetapkan spesifikasi format kontainer paket pembaruan offline: `.finnca-pkg`.

Struktur kontainer berkas `.finnca-pkg` (format ZIP):
```
finnca-0.3.0-linux-x86_64.finnca-pkg
├── manifest.json            # Metadata rilis, changelog, dan hash
├── binary.payload.tar.gz    # Arsip biner aplikasi terkompresi
├── signature.minisig        # Tanda tangan Minisign Ed25519 terhadap payload
└── checksums.sha256         # Hash SHA-256 dari seluruh isi payload
```

Backend Rust menyediakan perintah IPC `import_offline_update_cmd(file_path: String)` yang:
1. Mengekstrak dan memverifikasi tanda tangan `signature.minisig` terhadap kunci publik Minisign bawaan aplikasi.
2. Memverifikasi hash `checksums.sha256`.
3. Mempersiapkan biner baru ke area staging dan mengaktifkan protokol watchdog pemulihan crash.
4. Meminta izin pengguna di antarmuka frontend untuk me-restart aplikasi.

---

## Consequences

### Positif
- **Kekebalan Mutlak Terhadap Crash Loop**: Pengguna terlindungi 100% dari situasi biner rusak atau inkompatibilitas sistem operasi berkat fitur rollback atomik otomatis.
- **Integritas Rantai Pasok Biner**: Skrip shell, PowerShell, dan in-app updater memiliki perlindungan tanda tangan Minisign dan hash SHA-256 yang seragam.
- **Ketahanan Basis Data Finansial**: Transaksionalitas migrasi menjamin bahwa kegagalan sistem saat upgrade tidak akan pernah merusak skema SQLCipher.
- **Fleksibilitas Pengujian Komunitas**: Dukungan multi-channel memungkinkan pengujian fitur baru di kanal Beta tanpa mengorbankan stabilitas pengguna reguler di kanal Stable.
- **Kepatuhan Fasilitas Airgapped**: Format `.finnca-pkg` memungkinkan pembaharuan di lingkungan institusi atau brankas offline tanpa sambungan internet.

### Trade-off & Mitigasi
- **Alokasi Ruang Disk Temporer untuk Cadangan Biner**: Penyimpanan berkas `<exe_path>.backup` membutuhkan ruang disk temporer sekitar 60–120 MB selama jendela waktu pengujian 15 detik.  
  *Mitigasi*: Berkas cadangan otomatis dihapus secara permanen segera setelah aplikasi melewati masa uji coba stabil 15 detik.
- **Waktu Eksekusi Skrip CLI Bertambah Sedikit**: Verifikasi hash SHA-256 di skrip bash/PowerShell menambah waktu instalasi sekitar 0.5–1 detik.  
  *Mitigasi*: Tambahan waktu ini sepadan dengan jaminan keamanan dan pencegahan instalasi berkas biner korup.

---

## Technical Architecture & Code Contracts

### 1. Diagram Alur Siklus Hidup Pembaruan & Watchdog Rollback

```
+─────────────────────────────────────────────────────────────────────────────────────────+
|                               CI/CD & DISTRIBUSI SERVER                                 |
|                                                                                         |
|   GitHub Actions Release Pipeline (Zero-Fallback Policy)                                |
|   ├── Kompilasi Biner (Linux AppImage/deb/rpm, Windows setup.exe)                       |
|   ├── Tanda Tangan Minisign Ed25519 (Wajib: Gagal jika kunci tidak ada)                 |
|   └── Publikasi Kanal Terisolasi: latest.json / latest-beta.json / latest-nightly.json  |
+--------------------------------------------+--------------------------------------------+
                                             │
                      ┌──────────────────────┴──────────────────────┐
                      ▼                                             ▼
        [ In-App Tauri Updater ]                      [ Terminal Scripts CLI ]
      1. Ambil URL kanal terpilih                   1. Download SHA256SUMS + .minisig
      2. Verifikasi Minisign Ed25519                2. Verifikasi Minisign & Checksum
      3. Unduh biner ke staging                     3. Ganti biner secara atomik
                      │
                      ▼
+─────────────────────────────────────────────────────────────────────────────────────────+
|                         PROTOKOL STARTUP CRASH WATCHDOG                                 |
|                                                                                         |
|   Sebelum penggantian biner:                                                            |
|   - Salin biner aktif saat ini ke: finnca.backup                                        |
|   - Tulis state marker: ~/.config/finnca/.boot_state (attempts = 0)                     |
|                                                                                         |
|   Peluncuran biner baru:                                                                |
|   - main() membaca .boot_state:                                                         |
|       attempts += 1;                                                                    |
|       if attempts >= 2 {                                                                |
|           // DETEKSI CRASH LOOP!                                                        |
|           Salin balik: finnca.backup -> finnca                                          |
|           Hapus .boot_state                                                             |
|           Relaunch biner lama dengan notifikasi pemulihan                               |
|       }                                                                                 |
|                                                                                         |
|   Kestabilan Tercapai (Setelah 15 detik berjalan stabil ATAU vault berhasil dibuka):    |
|   - Hapus finnca.backup                                                                 |
|   - Hapus .boot_state                                                                   |
|   - Status: Update Resmi Permanen                                                       |
+─────────────────────────────────────────────────────────────────────────────────────────+
```

---

### 2. Implementasi Startup Watchdog di Rust (`src-tauri/src/updater/watchdog.rs`)

```rust
// File: src-tauri/src/updater/watchdog.rs
use crate::shared::AppError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BOOT_STATE_FILE: &str = ".boot_state";
const MAX_CRASH_ATTEMPTS: u32 = 2;
const STABILITY_THRESHOLD_SECS: u64 = 15;

#[derive(Debug, Serialize, Deserialize)]
pub struct BootState {
    pub target_version: String,
    pub timestamp_unix: u64,
    pub attempts: u32,
    pub confirmed_stable: bool,
}

pub struct UpdateWatchdog {
    config_dir: PathBuf,
    current_exe: PathBuf,
}

impl UpdateWatchdog {
    pub fn new(config_dir: PathBuf, current_exe: PathBuf) -> Self {
        Self { config_dir, current_exe }
    }

    /// Menemukan path biner sebenarnya yang dapat ditulis:
    /// Pada Linux AppImage, target adalah berkas outer .AppImage yang ditunjuk oleh $APPIMAGE,
    /// bukan mount read-only squashfs di /tmp/.mount_XXXX/.
    fn resolve_target_binary(&self) -> PathBuf {
        if let Ok(appimage_path) = std::env::var("APPIMAGE") {
            let p = PathBuf::from(appimage_path);
            if p.exists() {
                return p;
            }
        }
        self.current_exe.clone()
    }

    fn state_path(&self) -> PathBuf {
        self.config_dir.join(BOOT_STATE_FILE)
    }

    fn backup_exe_path(&self) -> PathBuf {
        let target = self.resolve_target_binary();
        let mut backup = target.clone();
        backup.set_extension("backup");
        backup
    }

    /// Dipanggil pada titik paling awal fungsi main() sebelum inisialisasi WebView.
    pub fn inspect_and_guard_startup(&self) -> Result<(), AppError> {
        let state_file = self.state_path();
        if !state_file.exists() {
            return Ok(()); // Peluncuran normal, tidak ada pembaruan tertunda
        }

        let raw = std::fs::read_to_string(&state_file)
            .map_err(|e| AppError::Io(e))?;
        let mut state: BootState = serde_json::from_str(&raw)
            .unwrap_or(BootState {
                target_version: "unknown".into(),
                timestamp_unix: 0,
                attempts: 0,
                confirmed_stable: false,
            });

        state.attempts += 1;

        if state.attempts >= MAX_CRASH_ATTEMPTS {
            eprintln!("[WATCHDOG ALERT] Peluncuran crash berulang terdeteksi ({} kali). Memulai rollback otomatis!", state.attempts);
            self.execute_atomic_rollback()?;
            return Ok(());
        }

        // Tulis ulang counter percobaan peluncuran
        let updated_raw = serde_json::to_string_pretty(&state)
            .map_err(|e| AppError::InvalidInput(format!("Gagal serialisasi boot state: {e}")))?;
        std::fs::write(&state_file, updated_raw)
            .map_err(|e| AppError::Io(e))?;

        // Jadwalkan pemeriksaan kestabilan di latar belakang
        self.spawn_stability_monitor();
        Ok(())
    }

    /// Mengembalikan biner lama dari cadangan secara atomik dengan mengatasi lock OS:
    /// - Linux: Unlink biner aktif lama untuk menghindari ETXTBSY.
    /// - Windows: Rename biner aktif lama (.corrupt.old) untuk menghindari ERROR_SHARING_VIOLATION.
    /// - AppImage: Mengoperasikan berkas outer $APPIMAGE, bukan mount squashfs read-only.
    fn execute_atomic_rollback(&self) -> Result<(), AppError> {
        let target_exe = self.resolve_target_binary();
        let backup = self.backup_exe_path();

        if backup.exists() {
            #[cfg(unix)]
            {
                // Pada Linux/Unix: Unlink target lama terlebih dahulu untuk mencegah error ETXTBSY (Text file busy).
                // Kernel mengizinkan unlink file descriptor yang sedang terbuka dari direktori.
                let _ = std::fs::remove_file(&target_exe);
                std::fs::rename(&backup, &target_exe)
                    .map_err(|e| AppError::Io(e))?;
            }

            #[cfg(windows)]
            {
                // Pada Windows: File yang sedang dibuka tidak dapat ditimpa atau dihapus,
                // namun Windows mengizinkan RENAME pada berkas yang sedang dieksekusi dalam volume yang sama.
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                let corrupt_swap = target_exe.with_extension(format!("corrupt.{}.old", now));
                let _ = std::fs::rename(&target_exe, &corrupt_swap);
                std::fs::rename(&backup, &target_exe)
                    .map_err(|e| AppError::Io(e))?;
            }

            let _ = std::fs::remove_file(&backup);
        }

        let _ = std::fs::remove_file(self.state_path());
        eprintln!("[WATCHDOG SUCCESS] Rollback ke biner sebelumnya berhasil diselesaikan.");
        Ok(())
    }

    /// Hook penutupan bersih aplikasi: Jika pengguna menutup aplikasi secara normal
    /// dalam waktu < 15 detik (misal: cek saldo lalu tutup), hapus state marker
    /// agar tidak dihitung sebagai startup crash.
    pub fn handle_clean_exit(&self) {
        let state_file = self.state_path();
        let backup_file = self.backup_exe_path();
        if state_file.exists() {
            let _ = std::fs::remove_file(state_file);
            let _ = std::fs::remove_file(backup_file);
            println!("[WATCHDOG] Aplikasi ditutup secara bersih oleh pengguna. Status pembaruan divalidasi aman.");
        }
    }

    /// Memulai timer kestabilan 15 detik di thread terpisah.
    fn spawn_stability_monitor(&self) {
        let state_file = self.state_path();
        let backup_file = self.backup_exe_path();

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(STABILITY_THRESHOLD_SECS));
            // Jika aplikasi masih hidup setelah 15 detik, bersihkan marker pembaruan
            if state_file.exists() {
                let _ = std::fs::remove_file(state_file);
                let _ = std::fs::remove_file(backup_file);
                println!("[WATCHDOG] Aplikasi berjalan stabil selama 15 detik. Pembaruan disahkan permanen.");
            }
        });
    }

    /// Dipanggil sebelum updater menimpa biner aplikasi.
    pub fn arm_watchdog(&self, target_version: &str) -> Result<(), AppError> {
        let target_exe = self.resolve_target_binary();
        let backup = self.backup_exe_path();

        // 1. Cadangkan biner yang dapat ditulis saat ini
        std::fs::copy(&target_exe, &backup)
            .map_err(|e| AppError::Io(e))?;

        // 2. Tulis penanda boot awal
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let state = BootState {
            target_version: target_version.to_string(),
            timestamp_unix: now,
            attempts: 0,
            confirmed_stable: false,
        };

        let raw = serde_json::to_string_pretty(&state)
            .map_err(|e| AppError::InvalidInput(format!("Gagal serialisasi: {e}")))?;
        std::fs::write(self.state_path(), raw)
            .map_err(|e| AppError::Io(e))?;

        Ok(())
    }
}
```

---

### 3. Implementasi Migrasi Skema Basis Data Transaksional (`src-tauri/src/db/schema.rs`)

```rust
// File: src-tauri/src/db/schema.rs
use crate::shared::AppError;
use rusqlite::Connection;

pub const MAX_SUPPORTED_SCHEMA_VERSION: u32 = 12;

/// Menerapkan migrasi skema secara atomik di dalam transaksi database individual
/// menggunakan Immediate Transaction untuk mengunci penulisan DDL dan user_version sekaligus.
pub fn run_migrations_transactional(conn: &mut Connection) -> Result<(), AppError> {
    let current_version: u32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;

    // Guard: Tolak basis data yang dibuat oleh versi aplikasi masa depan yang lebih tinggi
    if current_version > MAX_SUPPORTED_SCHEMA_VERSION {
        return Err(AppError::InvalidInput(format!(
            "Versi basis data ({current_version}) lebih tinggi dari batas versi yang didukung ({MAX_SUPPORTED_SCHEMA_VERSION}). Silakan perbarui aplikasi Finnca ke versi terbaru."
        )));
    }

    // Jalankan seluruh migrasi yang tertunda dalam transaksi terisolasi
    for v in (current_version + 1)..=MAX_SUPPORTED_SCHEMA_VERSION {
        let migration_sql = get_migration_sql(v)?;
        
        let mut tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        tx.execute_batch(migration_sql)?;
        tx.pragma_update(None, "user_version", v)?;
        tx.commit()?;
        
        println!("[MIGRATION SUCCESS] Skema berhasil ditingkatkan ke versi {v}");
    }

    Ok(())
}

fn get_migration_sql(version: u32) -> Result<&'static str, AppError> {
    match version {
        1 => Ok(include_str!("migrations/001_initial.sql")),
        2 => Ok(include_str!("migrations/002_add_reconcile.sql")),
        // ... definisi hingga migrasi 12
        _ => Err(AppError::NotFound(format!("Skrip migrasi versi {version} tidak ditemukan"))),
    }
}
```

---

### 4. Pengerasan Verifikasi Minisign Mandatori pada Skrip CLI (`scripts/finnca.sh` & `scripts/finnca.ps1`)

Skrip distribusi terminal diwajibkan memvalidasi tanda tangan kriptografis Minisign Ed25519 tanpa fallback tidak terotentikasi:

```bash
# File: scripts/finnca.sh (Bagian download biner)

verify_and_install_binary() {
    local target_file="$1"
    local expected_filename="$2"
    local checksums_file="/tmp/SHA256SUMS"
    local minisig_file="/tmp/SHA256SUMS.minisig"
    local pubkey="RWSDZ0KzV4jU7j9wX8+eY1Z2f3g4h5i6j7k8l9m0n1o2p3q4r5s6t7u8"

    echo "==> Mengunduh manifest checksum dan tanda tangan Minisign..."
    curl -fsSL "https://github.com/${REPO}/releases/download/${VERSION}/SHA256SUMS" -o "${checksums_file}"
    curl -fsSL "https://github.com/${REPO}/releases/download/${VERSION}/SHA256SUMS.minisig" -o "${minisig_file}"

    # Verifikasi integritas tanda tangan Minisign (MANDATORI: Dilarang fallback tanpa autentikasi)
    echo "==> Memvalidasi tanda tangan Minisign Ed25519..."
    if command -v minisign >/dev/null 2>&1; then
        if ! minisign -Vm "${checksums_file}" -P "${pubkey}" -x "${minisig_file}"; then
            echo "GALAT FATAL: Tanda tangan Minisign pada SHA256SUMS tidak valid! Batalkan instalasi."
            rm -f "${target_file}" "${checksums_file}" "${minisig_file}"
            exit 1
        fi
    else
        # Jika minisign belum terpasang, coba verifikasi dengan OpenSSL 3.x Ed25519 atau verifier mandiri
        echo "PERINGATAN: Alat 'minisign' tidak ditemukan di sistem host."
        echo "Mencoba verifikasi fallback berbasis OpenSSL Ed25519..."
        if ! openssl_verify_minisign "${checksums_file}" "${minisig_file}" "${pubkey}"; then
            echo "GALAT FATAL: Verifikasi tanda tangan digital gagal atau alat verifikasi tidak tersedia!"
            echo "Instalasi dibatalkan demi keamanan rantai pasok. Silakan pasang 'minisign' terlebih dahulu."
            rm -f "${target_file}" "${checksums_file}" "${minisig_file}"
            exit 1
        fi
    fi

    echo "==> Memvalidasi hash SHA-256 berkas biner..."
    local expected_hash
    expected_hash=$(grep "${expected_filename}" "${checksums_file}" | awk '{print $1}')

    if [ -z "${expected_hash}" ]; then
        echo "GALAT FATAL: Berkas ${expected_filename} tidak ditemukan di dalam SHA256SUMS!"
        rm -f "${target_file}" "${checksums_file}" "${minisig_file}"
        exit 1
    fi

    local actual_hash
    actual_hash=$(sha256sum "${target_file}" | awk '{print $1}')

    if [ "${expected_hash}" != "${actual_hash}" ]; then
        echo "GALAT FATAL: Hash berkas biner tidak cocok!"
        echo "Harapan:      ${expected_hash}"
        echo "Terkalkulasi: ${actual_hash}"
        rm -f "${target_file}" "${checksums_file}" "${minisig_file}"
        exit 1
    fi

    echo "==> Verifikasi kriptografis sukses. Melanjutkan instalasi atomik..."
}
```

Pada lingkungan Windows (`scripts/finnca.ps1`), mekanisme setara diterapkan: skrip memanfaatkan `minisign.exe` atau assembly .NET `System.Security.Cryptography` untuk memverifikasi tanda tangan `SHA256SUMS.minisig` sebelum menjalankan `finnca_setup.exe`. Tidak ada jalur instalasi yang mengizinkan eksekusi biner mentah tanpa verifikasi Minisign.

---

### 5. Skema Manifes Paket Offline (`.finnca-pkg`)

Format manifes `manifest.json` di dalam kontainer ZIP `.finnca-pkg`:

```json
{
  "format_version": 1,
  "package_id": "org.finnca.desktop",
  "version": "0.3.0",
  "release_channel": "stable",
  "target_os": "linux",
  "target_arch": "x86_64",
  "min_os_version": "glibc-2.35",
  "published_at": "2026-10-05T12:00:00Z",
  "payload": {
    "filename": "binary.payload.tar.gz",
    "sha256": "8a35b13e9a4f210d72f913d092d6e4b8590ef9351e3c23d08f3319be740df6e5",
    "uncompressed_bytes": 115343360
  },
  "signature": {
    "algorithm": "minisign-ed25519",
    "public_key": "RWSDZ0KzV4jU7j9wX8+eY1Z2f3g4h5i6j7k8l9m0n1o2p3q4r5s6t7u8",
    "signature_file": "signature.minisig"
  },
  "changelog_url": "https://github.com/endrico-fn/finnca/releases/tag/v0.3.0"
}
```

---

### 6. Metode Verifikasi Independen

1. **Uji Penanganan Crash Watchdog (*Watchdog Simulation*)**:
   - Tulis berkas tiruan `.boot_state` dengan `attempts: 1`.
   - Jalankan fungsi `inspect_and_guard_startup()` di bawah kondisi di mana peluncuran berikutnya crash.
   - Verifikasi bahwa sistem mengeksekusi `execute_atomic_rollback()` dan memulihkan biner cadangan `<exe_path>.backup`.
2. **Uji Transaksionalitas Migrasi Skema Basis Data**:
   - Sisipkan pernyataan SQL ilegal (misal: `SYNTAX ERROR TABLE`) di tengah migrasi skrip nomor 13 tiruan.
   - Jalankan `run_migrations_transactional()`.
   - Verifikasi bahwa migrasi mengembalikan galat `Err`, seluruh perubahan skrip 13 dibatalkan, dan nilai `PRAGMA user_version` tetap berada di angka 12 tanpa korupsi.
3. **Uji Penolakan Forward Version Basis Data**:
   - Atur `PRAGMA user_version = 15;` pada basis data pengujian.
   - Panggil `run_migrations_transactional()`.
   - Verifikasi bahwa sistem menolak membuka basis data dengan galat bahwa versi basis data lebih tinggi daripada `MAX_SUPPORTED_SCHEMA_VERSION`.
4. **Uji Kegagalan CI/CD Signing Fallback**:
   - Lakukan pengujian eksekusi simulasi GitHub Actions tanpa menyediakan variabel environment `TAURI_SIGNING_PRIVATE_KEY`.
   - Pastikan bahwa alur kerja keluar dengan exit code `1` dan membatalkan pekerjaan rilis.
5. **Uji Early Clean-Exit Watchdog**:
   - Tulis berkas `.boot_state` dengan `attempts: 1`.
   - Panggil `handle_clean_exit()` saat aplikasi ditutup dalam durasi < 15 detik.
   - Verifikasi bahwa `.boot_state` dan berkas backup dibersihkan seketika, memastikan penutupan cepat pengguna tidak pernah dihitung sebagai startup crash atau memicu rollback palsu.
6. **Uji Rollback Biner Berjalan Lintas-Platform**:
   - Simulasikan pemanggilan `execute_atomic_rollback()` saat biner sedang aktif.
   - Verifikasi bahwa pada Linux berkas lama di-unlink sebelum penggantian (menghindari `ETXTBSY`), pada Windows dilakukan rename `.corrupt.old` swap, dan jika `$APPIMAGE` diset, berkas outer AppImage yang dipulihkan (menghindari `EROFS` pada mount squashfs).


# ADR 0005: Tauri IPC Hardening & Granular Capabilities

## Status

Accepted

## Context

Sebelumnya aplikasi menggunakan konfigurasi kemampuan Tauri monolitik (`capabilities/default.json`) dengan izin sistem berkas global (`"fs:default"`). Pengaturan ini memberi hak akses frontend untuk membaca dan menulis di sembarang path sistem operasi, membuka celah keamanan serius terhadap eksekusi script jahat atau eksfiltrasi data (_filesystem directory traversal_). Selain itu, pembacaan file mutasi bank di halaman Reconcile langsung dilakukan dari frontend melalui plugin FS.

## Decision

Kami merekayasa ulang seluruh lapisan IPC dan kemampuan Tauri:

1. Menghapus total berkas `capabilities/default.json` dan mencabut plugin `"fs:default"`.
2. Membagi capability menjadi file-file granular dengan prinsip _least privilege_:
   - `capabilities/vault.json`
   - `capabilities/ledger.json`
   - `capabilities/security.json`
   - `capabilities/reports.json`
   - `capabilities/settings.json`
3. Memindahkan operasi pembacaan berkas rekening koran ke Rust command `read_statement_file_cmd` yang memiliki:
   - Validasi session aktif (_session authentication_).
   - Pembatasan ukuran berkas maksimal 5 MB untuk mencegah serangan _Denial of Service (DoS)_.
   - Pengecekan sanitasi path (_path canonicalization_).

## Consequences

- **Positif:** Frontend Svelte tidak memiliki akses langsung ke filesystem OS.
- **Positif:** Serangan XSS atau injeksi frontend tidak dapat mengakses atau membocorkan file di luar session vault aktif.
- **Trade-off:** Operasi file eksternal harus melewati Rust IPC wrapper dan melalui konversi DTO.

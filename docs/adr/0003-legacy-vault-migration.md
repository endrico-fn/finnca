# ADR 0003: Legacy Vault Migration (Age Blob to SQLCipher)

## Status
Accepted

## Context
Versi awal Finnca menyimpan seluruh data finansial sebagai satu berkas JSON terkompresi yang dienkripsi menggunakan format enkripsi `age`. Pendekatan ini memiliki kelemahan skalabilitas fatal: setiap kali satu transaksi dicatat, seluruh berkas harus didekripsi ke memori, dimodifikasi, dan dienkripsi ulang secara penuh. Hal ini menyebabkan lonjakan memori (*OOM risk*) dan latensi tinggi saat data bertumbuh.

## Decision
Kami mengimplementasikan modul migrasi otomatis satu kali (*one-time automatic migration*):
1. Saat pengguna membuka vault format lama (`vault.age`), mesin mendeteksi ketiadaan `vault.db`.
2. Sistem secara otomatis membuat berkas cadangan (*backup copy*) `vault.age.bak` sebelum melakukan operasi penulisan apa pun.
3. Mesin `legacy_migration.rs` mendekripsi payload `age`, mem-parse entitas JSON lama, dan memindahkannya ke tabel SQLite SQLCipher baru (`0001_init.sql`).
4. Total saldo seluruh akun dihitung ulang dan dicocokkan terhadap data lama sebelum commit transaksi database. Jika terdapat selisih satu sen pun, transaksi di-rollback dan berkas lama dipertahankan.

## Consequences
- **Positif:** Pengguna lama dapat langsung bermigrasi tanpa kehilangan data historis.
- **Positif:** Keamanan data terjamin melalui mekanisme *backup-before-write* dan verifikasi saldo sebelum penyelesaian migrasi.
- **Trade-off:** Kode pustaka kriptografi lama (`x25519-dalek`, `scrypt`) harus tetap ada di dependensi Rust hingga seluruh basis pengguna lama bertransisi penuh.

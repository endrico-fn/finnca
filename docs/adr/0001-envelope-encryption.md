# ADR 0001: Envelope Encryption via Argon2id Key Wrapping

## Status
Accepted

## Context
Aplikasi Finnca menyimpan data finansial pengguna dalam berkas SQLite yang dienkripsi menggunakan SQLCipher. Pengguna memiliki hak untuk mengganti password vault kapan saja. Jika password digunakan secara langsung melalui KDF untuk menjadi `PRAGMA key`, setiap pergantian password akan memerlukan re-enkripsi penuh (*page-by-page rekeying*) terhadap seluruh basis data SQLite, yang berisiko tinggi mengalami korupsi data jika terjadi interupsi daya (*power failure*) di tengah proses.

## Decision
Kami menerapkan pola **Envelope Encryption**:
1. Basis data SQLCipher dienkripsi menggunakan kunci acak kriptografis berkekuatan 256-bit (`db_key`).
2. Kunci `db_key` dienkripsi (di-*wrap*) menggunakan kunci turunan (*derived key*) dari password pengguna menggunakan algoritma **Argon2id**.
3. Paket terenkripsi (*wrapped envelope*) disimpan dalam metadata vault (`vault.meta.json`).
4. Saat pengguna mengubah password, sistem hanya perlu melakukan de-wrapping terhadap `db_key` menggunakan password lama, kemudian me-re-wrap `db_key` menggunakan password baru. Basis data SQLite tidak perlu disentuh sama sekali.

## Consequences
- **Positif:** Mengganti password berlangsung instan ($O(1)$) tanpa risiko korupsi basis data.
- **Positif:** Parameter KDF Argon2id (`m_cost`, `t_cost`, `p_cost`) dapat disesuaikan di masa depan tanpa harus memigrasikan database.
- **Trade-off:** Metadata vault (`vault.meta.json`) menjadi komponen kritis yang wajib dijaga integritasnya bersama dengan berkas `vault.db`.

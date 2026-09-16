# ADR 0002: Flat Ledger Schema with Orthogonal Dimensions

## Status
Accepted

## Context
Aplikasi akuntansi tradisional kerap memaksakan struktur Chart of Accounts (COA) bersarang (*deeply nested tree*) hingga 5-7 level. Pendekatan ini mempersulit penulisan query SQL, memperlambat agregasi saldo (*recursive CTE overhead*), dan membuat antarmuka UI sempit serta membingungkan pengguna pada layar desktop.

## Decision
Kami mengadopsi skema **Flat Ledger dengan Hierarki Path Datar**:
1. Akun disimpan secara datar dengan representasi path: `Root > Subaccount > Account` (misal: `Assets > Banking > BCA Checking`).
2. Transaksi disimpan sebagai `journal_entries` dan `postings` berpasangan.
3. Kategori analitik, proyek, atau penanda tambahan ditangani sebagai dimensi ortogonal (*orthogonal dimensions* / tags) alih-alih membuat rantai akun anak yang tidak perlu.
4. UI merender path akun secara datar dan jelas, bukan dalam bentuk *tree-folder* yang rumit.

## Consequences
- **Positif:** Query agregasi saldo (*balance rollups*) menjadi sangat cepat dan sederhana.
- **Positif:** UI input transaksi bersih dan intuitif—pengguna langsung memilih akun berdasarkan jalur path lengkap.
- **Trade-off:** Hubungan relasi parent-child akun divalidasi pada lapisan service di Rust untuk mencegah dependensi siklik (*cycle prevention*).

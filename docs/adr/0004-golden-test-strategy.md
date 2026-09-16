# ADR 0004: Golden Test Strategy for Financial Engine Verification

## Status
Accepted

## Context
Ketika memindahkan logika perhitungan keuangan kompleks (kalkulasi saldo, penanganan multi-split, rekonsiliasi, konversi multi-currency) dari TypeScript ke Rust, risiko deviasi pembulatan (*rounding mismatch*) atau perubahan perilaku logika adalah ancaman kritis terhadap integritas data pengguna.

## Decision
Kami mengadopsi metodologi **Golden Test Fixture Verification**:
1. Menghasilkan snapshot data transaksi kompleks dari logika TypeScript lama yang telah terbukti benar ke dalam format JSON standar di `tests/fixtures/ledger-golden/`.
2. Menulis suite integrasi pengujian Rust (`tests/ledger_golden_test.rs`) yang mengeksekusi fixture golden tersebut secara deterministik.
3. Mesin Rust baru wajib menghasilkan saldo akun, status cleared/uncleared, dan nilai konversi mata uang yang identik 100% dengan fixture golden lama.
4. Kode TS lama baru diizinkan untuk dimusnahkan hanya setelah seluruh golden test di Rust lulus secara konsisten.

## Consequences
- **Positif:** Verifikasi matematis tanpa keraguan (*zero regression*).
- **Positif:** Pengujian integrasi Rust dapat dijalankan secara cepat pada pipeline CI (`cargo test`).
- **Trade-off:** Memerlukan pemeliharaan berkas fixture JSON di repositori.

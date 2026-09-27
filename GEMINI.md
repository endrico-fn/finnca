# 📐 finnca architecture & development guidelines

aplikasi desktop pencatatan personal note finance double entry tingkat lanjut yang di sesuaikan

1. **kategory teme**: brutalist, utilitarium, minimalis modern UI, industrial tech ui yang menerapkan sudut tajam pada border, animasi ringan UI/UX konsisten dan di atur secara rapih struktur code yang unik, efisiensi pengembangan
2. **tipografi & hierarki**:
   - `proto mono`: wajib digunakan untuk _heading_, deskripsi pendek, label aksi, _tabs_, numerik/angka, kode akun, dan _badge_ status.
   - `aux mono`: digunakan khusus untuk teks _body_ atau penjelasan detail yang panjang, serta input data-entry (natural case, tanpa pemaksaan kapital)
3. **privasi**
   - menerapkan konsep kunci vault timeout
   - register(setup): user baru akan melihat tampilan membuat vault dan profile, seperti membuat nama vault dan menentukan path di mana vault di simpan(terinspirasi dari meknisme vault obsidian)
     dan menentukan username(identitas) dan password(privasi tambahan unlock/del vault), mendukung multi vault, opsi delete vault yang selalu menggunakan password, login vault dimana user sudah melewati tahap pendaftaran
     , tersedia juga opsi import vault jika user memiliki folder cadangan, format vault: app_name-vault_name/
     - fitur: add/delete vault, multi vault, rename vault/username, change password, import vault
4. zero inline comment pada code kecuali penting
5. **i18n strictness**: dilarang ada _string_ teks bahasa ui yang _hardcoded_. semua wajib melalui `src/lib/core/i18n.svelte.ts`.
6. penolakan hardcode string, gunakan pendekatan rapih memisahkan di file pusan di mana string di atur misal FINNCA nama aplikasi tidak di tulis string melainkan gunakan data project FINNCA dan di atur sebagai APP_NAME begitu juga APP_VERSION
7. gunakan struktur code ts,rs,tailwind dan svelte yang rapih dan modular untuk pengembangan jangka panjang
8. logika perhitungan atur di rust dan konsisten sampai memiliki data transaksi pencatatan keuangan semkin besar
9. UI/UX layout konsisten, hindari redunansi code(code berulang) atau fungsi
10. selalu gunakan struktur code yang konsisten, unik pada backend maupun frontend, struktur code dan mekanisme unik yang tidak kaku
11. UI yang di sesuaikan pada finnca adalah tampilan account tidak berpenampilan sebagai tree melainkan path account: rootaccount > subaccount > account
12. **second brain & architectural knowledge protocol**: seluruh keputusan arsitektur, token UI, spesifikasi komponen, dan status fitur wajib terdokumentasi dan dirujuk pada `docs/architecture/` (`00_MANIFESTO.md`, `01_DESIGN_SYSTEM.md`, `02_COMPONENT_SPEC.md`, `03_DOMAIN_CATALOG.md`, `04_FEATURE_REGISTRY.md`) & `docs/adr/`. Setiap agen/pengembang wajib merujuk second brain ini sebelum merancang perubahan besar.
13. **zero-tolerance css drift**: dilarang keras menggunakan warna _hex/rgb_ atau arbitrary utility (`px-[13px]`) langsung di komponen Svelte. Wajib gunakan semantic design tokens Tailwind v4 (`bg-bg-app`, `bg-bg-card`, `bg-bg-btn`, `border-line`, `text-teal`, `text-asset`, `text-expense`, dll).
14. **dead code vs unwired stubs policy**: bedakan secara mutlak antara kode usang/rusak (_dead code_) dan fitur tertunda (_unwired feature stubs_). Kode usang wajib dimusnahkan. Fitur/algoritma yang dipersiapkan untuk masa depan tetapi belum tersambung ke UI wajib didaftarkan di `docs/architecture/04_FEATURE_REGISTRY.md` dengan status `[STUB/PLANNED]`, dan dilarang dihapus tanpa evaluasi arsitektur dan hapus deadcode sesudah refaktor dan pastikan itu tidak berfungs lagi etelah refaktor
15. **contract-first ui components**: seluruh komponen atomik dan molekul wajib memiliki antarmuka yang seragam (menggunakan Svelte 5 Snippets `children`, `actions`, `header`), sudut tajam mutlak (`rounded-none`), varian tone terstandar (`neutral`, `ok`, `err`, `warn`), dan tipografi disiplin (`font-proto` untuk metrik/aksi vs `font-aux` untuk narasi).
16. **sandboxing & least privilege**: tidak ada akses filesystem langsung dari frontend (`fs:default` dicabut). Seluruh I/O berkas wajib melewati Rust IPC command yang terotentikasi session aktif dengan pembatasan ukuran kuota untuk mencegah DoS.

struktur baru:

```
finnca/
├── docs/                                          🆕 [p] — engineering docs & project second brain
│   ├── architecture/                              # second brain: fondasi arsitektur per project
│   │   ├── 00_MANIFESTO.md                        # prinsip fundamental, zero-float, estetika brutalist
│   │   ├── 01_DESIGN_SYSTEM.md                    # kontrak token warna, tipografi proto/aux mono, spacing
│   │   ├── 02_COMPONENT_SPEC.md                   # kontrak props/snippets komponen UI (Card, PageLayout, dll)
│   │   ├── 03_DOMAIN_CATALOG.md                   # katalog domain sistem & batas subsistem
│   │   └── 04_FEATURE_REGISTRY.md                 # matriks status fitur (Live, In-Progress, Stub, Purged)
│   └── adr/                                       # architecture decision records
│       ├── 0001-envelope-encryption.md            # kenapa argon2id wrap key, bukan derive pragma key langsung
│       ├── 0002-ledger-schema.md                  # kenapa flat ledger + dimensi ortogonal (bukan nested coa dalam)
│       ├── 0003-legacy-vault-migration.md         # strategi migrasi blob(age) -> sqlite, rollback plan
│       └── 0004-golden-test-strategy.md           # bagaimana fixture ts lama jadi oracle rust baru
│       └── 0005-tauri-ipc-hardening.md            # pencabutan fs:default, session-gated reader, clamping DoS
│
├── .github/
│   └── workflows/
│       └── ci.yml                                 🆕 [p] — cargo fmt --check, clippy -d warnings, cargo test,
│                                                   #        tsc --noemit, eslint, vitest; wajib hijau sebelum merge
│
├── src/                                            ============ frontend (svelte 5 + ts) ============
│   ├── app.html
│   ├── main.ts
│   ├── app.css                                     # tailwind entry — token restructure sudah jalan, tidak disentuh di sini
│   │
│   ├── lib/
│   │   ├── core/                                   # [w] infrastruktur lintas-fitur — tidak boleh import dari features/
│   │   │   ├── ipc/
│   │   │   │   ├── bindings.ts                     # 🤖 auto-generated tauri-specta — [w] jangan pernah edit manual
│   │   │   │   ├── client.ts                       🆕 [w] — wrapper invoke() + withtimeout + error mapping terpusat
│   │   │   │   │                                   #        (ganti pola withtimeout yang sekarang ada per-fungsi di api.ts)
│   │   │   │   └── errors.ts                       🆕 [w] — union type utk apperror code dari rust; ui switch atas
│   │   │   │                                       #        kode terstruktur, bukan string message mentah
│   │   │   ├── events/
│   │   │   │   └── eventbus.svelte.ts              ♻️ [p] — wrap listen() tauri jadi reactive source
│   │   │   ├── router/
│   │   │   │   └── tabrouter.svelte.ts             🆕 [p]
│   │   │   ├── format/
│   │   │   │   └── currency.ts                     🆕 [w] — satu-satunya tempat format integer-minor-units -> display
│   │   │   │                                       #        string (intl.numberformat). semua komponen wajib lewat sini,
│   │   │   │                                       #        dilarang keras format manual di komponen (float leak risk)
│   │   │   ├── i18n.svelte.ts                      ♻️ [p] — pindah dari src/lib/ ke core/, ini infra bukan feature
│   │   │   └── state/                              # global reactive state (runes)
│   │   │       ├── session.svelte.ts               ♻️ [w] — isunlocked, currentvault, currentuser
│   │   │       ├── tabs.svelte.ts                  ♻️ [p]
│   │   │       ├── modal.svelte.ts                 ♻️ [p]
│   │   │       ├── security.svelte.ts              ♻️ [w] — auto-lock config & countdown
│   │   │       ├── window.svelte.ts                🆕 [p] — state os window (fullscreen, always-on-top)
│   │   │       └── notification.svelte.ts          ♻️ [p]
│   │   │
│   │   ├── components/                             # ui generik/reusable, tidak tahu domain
│   │   │   ├── ui/                                 ♻️ [p] — button, input, table, datepicker, currencyinput...
│   │   │   ├── layout/
│   │   │   │   ├── sidebar.svelte
│   │   │   │   ├── topbar.svelte
│   │   │   │   ├── tabbar.svelte
│   │   │   │   └── modalhost.svelte
│   │   │   └── feedback/
│   │   │       ├── confirmdialog.svelte            ♻️ (dari confirmmodal.svelte)
│   │   │       └── loadingspinner.svelte           ♻️ (dari spinner.svelte)
│   │   │
│   │   └── features/                               # === feature-based / domain modules ===
│   │       │                                       # [w] aturan wajib per feature: state/*.svelte.ts tidak boleh
│   │       │                                       #     diimport lintas-feature. kalau butuh data feature lain,
│   │       │                                       #     lewat core/ipc atau event, bukan import langsung.
│   │       ├── vault/
│   │       │   ├── components/
│   │       │   │   ├── vaultpicker.svelte          ♻️ (dari vaultswitcher.svelte)
│   │       │   │   ├── registerform.svelte
│   │       │   │   └── loginform.svelte
│   │       │   └── state/vaultlist.svelte.ts       ♻️ (dari stores/vault-registry.svelte.ts)
│   │       ├── security/
│   │       │   ├── components/autolocksettings.svelte
│   │       │   └── state/lockpolicy.svelte.ts
│   │       ├── dashboard/
│   │       │   └── components/
│   │       │       ├── summarycards.svelte         ♻️ (dari routes/app/dashboard/_widgets/balancecard.svelte dll)
│   │       │       ├── cashflowchart.svelte         ♻️ (dari cashflowcard.svelte)
│   │       │       └── recenttx.svelte             ♻️ (dari recenttransactions.svelte)
│   │       ├── accounts/                           # chart of accounts
│   │       │   ├── components/
│   │       │   │   ├── accounttree.svelte          ♻️ (dari accountrow.svelte, direstruktur jadi tree)
│   │       │   │   └── accountform.svelte          ♻️ (dari accountform.svelte, sudah ada)
│   │       │   └── state/accounts.svelte.ts        ♻️ (logic dari accounting/ledger/accounts.ts pindah ke rust;
│   │       │                                       #   file ini setelah refactor cuma state+ipc call, bukan business logic)
│   │       ├── journal/                            # general ledger entry
│   │       │   ├── components/
│   │       │   │   ├── journalentryform.svelte     ♻️ (dari transactioneditor.svelte)
│   │       │   │   ├── journallinerow.svelte        🆕
│   │       │   │   └── journallist.svelte           🆕
│   │       │   └── state/journaldraft.svelte.ts    ♻️ [w] — logic invariant debit=kredit pindah ke rust
│   │       │                                       #   (ledger/validation.rs), file ini hanya draft ui + call ipc.
│   │       │                                       #   jangan hitung ulang balance di frontend untuk keputusan apa pun.
│   │       ├── budget/
│   │       │   └── state/budget.svelte.ts          ♻️ (logic dari accounting/features/budgeting.ts -> rust)
│   │       ├── plan/
│   │       │   ├── components/
│   │       │   │   ├── planmodal.svelte            ♻️
│   │       │   │   ├── calendarview.svelte         ♻️
│   │       │   │   └── plansoverview.svelte        ♻️
│   │       │   └── state/plan.svelte.ts            ♻️ (logic dari accounting/features/planning.ts -> rust)
│   │       ├── reconcile/
│   │       │   ├── components/reconcilewizard.svelte
│   │       │   └── state/reconcile.svelte.ts       ♻️ (logic dari features/reconcile.ts, reconcilejobs.ts -> rust)
│   │       ├── report/
│   │       │   ├── components/
│   │       │   │   ├── reportviewer.svelte
│   │       │   │   ├── cashflow.svelte             ♻️
│   │       │   │   ├── debtreport.svelte           ♻️
│   │       │   │   ├── debtsimulator.svelte        ♻️
│   │       │   │   ├── fxreport.svelte             ♻️
│   │       │   │   ├── trends.svelte               ♻️
│   │       │   │   └── trialbalance.svelte         ♻️
│   │       │   └── state/report.svelte.ts          ♻️ (logic dari accounting/reports/*.ts -> rust generators)
│   │       ├── audit/                              🆕 viewer log aktivitas (read-only, tidak ada logic sisi fe)
│   │       │   └── components/auditlogviewer.svelte
│   │       └── settings/
│   │           ├── components/
│   │           │   ├── datasettings.svelte         ♻️
│   │           │   ├── financesettings.svelte       ♻️
│   │           │   ├── generalsettings.svelte       ♻️
│   │           │   └── securitysettings.svelte      ♻️
│   │           └── state/settings.svelte.ts        🆕
│   │
│   ├── routes/                                     # sveltekit routing — tipis, cuma compose feature components
│   │   ├── +layout.svelte
│   │   ├── +layout.ts
│   │   ├── +page.svelte
│   │   ├── login/+page.svelte
│   │   ├── setup/+page.svelte
│   │   └── app/
│   │       ├── +layout.svelte
│   │       ├── +page.svelte
│   │       ├── accounts/{+page.svelte,[code]/+page.svelte}
│   │       ├── budget/+page.svelte
│   │       ├── journal/+page.svelte
│   │       ├── plan/+page.svelte
│   │       ├── reconcile/+page.svelte
│   │       ├── reports/+page.svelte
│   │       └── setting/+page.svelte
│   │
│   └── tests/
│       ├── unit/                                   [p] — hindari dump semua test di sini; lihat catatan di bawah
│       └── fixtures/ya
│           └── ledger-golden/                      🆕 [w] — snapshot output dari logic ts lama sebelum dihapus,
│                                                   #        dipakai sbg oracle utk verifikasi hasil rust identik
│
├── src-tauri/                                       ============ backend (rust) ============
│   ├── cargo.toml
│   ├── cargo.lock
│   ├── tauri.conf.json
│   ├── build.rs                                    ♻️ [w] — tambah tauri_specta::builder export di sini
│   ├── icons/
│   │
│   ├── capabilities/                               # [w] least-privilege per domain — ganti default.json monolitik
│   │   ├── vault.json                              # register/login/switch vault
│   │   ├── ledger.json                             # post/read journal & account balance
│   │   ├── security.json                           # auto-lock, touch_activity
│   │   ├── reports.json                            # read-only
│   │   └── settings.json
│   │                                                # [w] fs:default dihapus — scope fs permission ke path vault aktif
│   │                                  ya              #     saja (fs:scope), bukan seluruh filesystem
│   │
│   ├── tests/                                      # integration test (cargo test --test *)
│   │   ├── ledger_balance_test.rs                  🆕 [w] — jalan terhadap fixtures/ledger-golden/*
│   │   ├── vault_crypto_test.rs                    🆕 [w]
│   │   ├── legacy_migration_test.rs                🆕 [w] — round-trip blob lama -> sqlite -> verifikasi total saldo
│   │   └── fixtures/                               🆕 (symlink/copy dari src/tests/fixtures/ledger-golden)
│   │
│   └── src/
│       ├── main.rs                                 ♻️ [w] — tetap tipis, cuma panggil finnca_lib::run()
│       ├── lib.rs                                  ♻️ [w] — builder tauri + invoke_handler + tauri-specta collect_commands!
│       │                                           #        (bukan main.rs — koreksi dari draft awal, ini konvensi tauri 2:
│       │                                           #        lib.rs perlu #[cfg_attr(mobile, tauri::mobile_entry_point)])
│       ├── app_state.rs                            ♻️ (dari state.rs) — appstate: vaultsession, activitytracker (mutex)
│       ├── app_config.rs                           🆕 [w] — ♻️ (dari config.rs) config global non-vault-scoped:
│       │                                           #        known_vaults registry, settings, legacy path migration.
│       │                                           #        ini sengaja dipisah dari db/ — ini bukan data finansial,
│       │                                           #        hidup di ~/.config, tidak dienkripsi sqlcipher.
│       │
│       ├── vault/                                  # === domain: vault management ===
│       │   ├── mod.rs
│       │   ├── manager.rs                          ♻️ — add/del/list vault, resolve path finnca-<nama>
│       │   ├── metadata.rs                         🆕 — baca/tulis vault.meta.json (salt, kdf params, schema version)
│       │   ├── legacy_migration.rs                 🆕 [w] — satu-kali-pakai: decode vault.age lama, tulis ke
│       │   │                                       #        sqlite baru. isolasi di sini supaya gampang dihapus
│       │   │                                       #        setelah semua vault termigrasi. wajib backup-before-write.
│       │   ├── dto.rs
│       │   └── commands.rs
│       │
│       ├── crypto/                                 # === domain: enkripsi & key derivation ===
│       │   ├── mod.rs
│       │   ├── kdf.rs                               🆕 [w] — argon2id, parameter eksplisit (m_cost/t_cost/p_cost)
│       │   │                                       #        dikomentari alasannya di adr 0001, jangan pakai default lib
│       │   ├── envelope.rs                         🆕 [w] — password -> argon2id -> unwrap random db key.
│       │   │                                       #        pola envelope ini wajib dipertahankan (bukan derive
│       │   │                                       #        pragma key langsung dari password) supaya ganti
│       │   │                                       #        password tidak perlu re-encrypt seluruh db.
│       │   └── sqlcipher.rs                        🆕 [w] — pragma key, pragma rekey, cipher params
│       │
│       ├── auth/                                   # === domain: autentikasi ===
│       │   ├── mod.rs
│       │   ├── register.rs
│       │   ├── login.rs
│       │   └── commands.rs
│       │
│       ├── security/                                # === domain: auto-lock ===
│       │   ├── mod.rs
│       │   ├── activity_tracker.rs
│       │   ├── lock_policy.rs                       # enum: onclose, onreboot, ontimeout(duration)
│       │   └── commands.rs
│       │
│       ├── ledger/                                  # === domain: double-entry accounting (core — port pertama) ===
│       │   ├── mod.rs
│       │   ├── models.rs                            🆕 — account, journalentry, posting, dimension
│       │   ├── dto.rs                               🆕 — kontrak serialisasi ke frontend (specta::type)
│       │   ├── service.rs                           🆕 [w] — posting, balance calc; semua logic dari
│       │   │                                       #        accounting/ledger/transactions.ts pindah ke sini,
│       │   │                                       #        diverifikasi terhadap golden fixture sebelum cutover
│       │   ├── validation.rs                        🆕 [w] — sum(debit)==sum(kredit), akun aktif — pure function,
│       │   │                                       #        no i/o, gampang di-unit-test
│       │   ├── repository.rs                        🆕 — query/insert sqlite
│       │   ├── currency.rs                          🆕 — multi-currency, exchange rate snapshot (bukan formatting;
│       │   │                                       #        formatting display tetap di frontend core/format/currency.ts)
│       │   └── commands.rs
│       │
│       ├── accounts/                                # chart of accounts crud — [w] port paling pertama
│       │   ├── mod.rs                              #   (risiko rendah, validasi seluruh pipeline baru: schema ->
│       │   ├── models.rs                           #    repository -> service -> command -> specta -> capability)
│       │   ├── dto.rs
│       │   ├── repository.rs
│       │   └── commands.rs
│       │
│       ├── budget/{mod.rs,models.rs,service.rs,repository.rs,commands.rs}       🆕
│       ├── plan/{mod.rs,models.rs,service.rs,repository.rs,commands.rs}         🆕
│       │
│       ├── reconcile/
│       │   ├── mod.rs
│       │   ├── matcher.rs                          # pencocokan transaksi vs statement
│       │   └── commands.rs
│       │
│       ├── report/
│       │   ├── mod.rs
│       │   ├── generators/                         # profit_loss.rs, balance_sheet.rs, cash_flow.rs
│       │   └── commands.rs
│       │
│       ├── audit/                                   🆕 domain: audit trail (append-only)
│       │   ├── mod.rs
│       │   ├── models.rs                           # auditentry { actor, action, entity, before/after, ts }
│       │   ├── repository.rs
│       │   └── commands.rs                         # get_audit_log (read-only, paginated)
│       │
│       ├── db/
│       │   ├── mod.rs
│       │   ├── schema.rs
│       │   ├── migrations/                         # [w] file bernomor, append-only, jangan pernah edit yang lama
│       │   │   ├── 0001_init.sql                   🆕 — accounts, journal_entries, postings, dimensions
│       │   │   ├── 0002_add_budgets.sql
│       │   │   └── 0003_add_audit_log.sql
│       │   └── pool.rs                             # r2d2 connection pool per vault aktif
│       │
│       └── shared/
│           ├── error.rs                            🆕 [w] — apperror terpusat (thiserror), map ke error code
│           │                                       #        terstruktur ke frontend, bukan format!("...: {e}") mentah
│           │                                       #        (kode lama saat ini bocorkan io::error/path lokal ke
│           │                                       #        frontend — sanitasi di sini, detail mentah cuma di log)
│           ├── events.rs                           🆕 [w] — internal event bus (mis. `transactionposted`), supaya
│           │                                       #        ledger tidak depend langsung ke audit; audit subscribe
│           │                                       #        event, ledger tidak tahu audit itu ada. cross-domain
│           │                                       #        coupling dicegah lewat ini, bukan lewat import langsung.
│           └── ids.rs                              # generator id (ulid) — urut, aman utk pk finansial
│
├── package.json
├── vite.config.ts
└── tsconfig.json
```

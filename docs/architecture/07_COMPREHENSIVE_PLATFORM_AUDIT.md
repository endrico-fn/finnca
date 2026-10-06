# Comprehensive Platform Architecture, Cryptographic Security & Desktop Maturity Audit

**Document Reference**: `FINNCA-AUDIT-2026-10-05`  
**Classification**: Engineering Architecture & Security Specification  
**Status**: Publication-Ready Master Audit Report  
**Author Guild**: Platform Security, Architecture & Reliability Survey Guild  
**Audit Target**: Finnca Desktop (`src/`, `src-tauri/`, `scripts/`, `docs/`)  
**Repository Working Copy**: `/home/endrico/Projects/finnca`  
**Audit Date**: 2026-10-05  
**Version Under Audit**: `v0.1.0` (Tauri v2.12.0 / Svelte 5.56.9)

---

## 1. Executive Summary & Audit Metadata

### 1.1 Audit Metadata

| Metadata Field | Value |
|:---|:---|
| **Audit Scope** | Core Cryptography, IPC Capabilities, Sensitive Memory Hygiene, Plugin Architecture, Release Pipelines, Desktop Reliability, Dependency Vulnerabilities, Industry Benchmarks |
| **Commit Target** | HEAD (Git branch `main`) |
| **Operating System Targets** | Linux (x86_64 AppImage/deb/rpm), Windows (x64 NSIS currentUser), macOS (Universal planned) |
| **Lead Survey Explorers** | Explorer 1 (Security & Crypto), Explorer 2 (Plugin Architecture), Explorer 3 (Updates & Desktop Maturity) |
| **Lead Auditor / Author** | Worker Report 1 (Platform Audit Report Author) |
| **Primary Frameworks** | Rust 1.85+ (Tauri v2.12.0, SQLCipher 4), Svelte 5 (Runes mode), TypeScript 5.8 |

---

### 1.2 Executive Posture Assessment

Finnca demonstrates exceptional architectural discipline in its foundational accounting and cryptographic primitives. The application enforces a strict **Zero-Float Arithmetic Invariant** using signed 64-bit integer minor units (`i64`), an uncompromised double-entry equation check (`sum(Debit) == sum(Credit)`), an immutable SHA-256 audit log with `GENESIS`-rooted hash chaining, and an enterprise-grade envelope encryption scheme (Argon2id KDF + ChaCha20-Poly1305 + SQLCipher).

However, an exhaustive evaluation across the application's attack surface, platform lifecycle, and release pipelines uncovered **several critical-to-high security and operational hazards**:

1. **Arbitrary Local File Read Vulnerability (`SEC-01`)**: An unauthenticated IPC command (`read_statement_file_cmd`) permits arbitrary reads of host files up to 5 MB (e.g., `/etc/passwd`, `~/.ssh/id_rsa`), completely subverting the Tauri capability sandbox. Remediation requires pre-canonicalization symlink inspection, strict statement-only extensions (`.csv`, `.ofx`, `.qfx`, `.mt940`, `.sta` — eliminating `.txt`), Windows system directory blocks, and credential root isolation while allowing user vaults.
2. **Cryptographic Storage Memory Isolation (`SEC-02`)**: SQLCipher initialization fails to configure `PRAGMA temp_store = MEMORY;`. While SQLCipher transparently encrypts temporary b-trees on keyed connections, omitting this pragma allows temporary query tables to spill to disk (`/tmp` or `%TEMP%`), increasing forensic exposure and degrading query performance compared to strictly enforced in-memory temporary storage.
3. **Sensitive Memory Residue (`SEC-03` & `SEC-07`)**: Key encryption buffers (`Vec<u8>`), hex key strings (including `export_encrypted_snapshot` ATTACH queries and `rusqlite::execute_batch` CString allocations), and master password inputs are deallocated without active heap zeroization across Rust and Svelte V8 runtime boundaries.
4. **Unhardened Release Profile (`SEC-04`)**: Production builds lack Link-Time Optimization (LTO), retain stack unwinding frames, leave symbols unstripped, and enable developer inspection tools unconditionally.
5. **Distribution Pipeline Integrity Gaps (`SEC-UP-01` & `SEC-05`)**: Terminal installer scripts (`finnca.sh`, `finnca.ps1`) download binary packages without mandatory Minisign Ed25519 signature verification, risking unauthenticated execution or MITM tampering, while the frontend CSP permits direct external HTTP calls.
6. **Database Migration Atomicity Hazard (`REL-DB-01`)**: Schema migrations run outside explicit transactional boundaries (`BEGIN IMMEDIATE ... COMMIT`), creating split-brain schema states on sudden power loss.

---

### 1.3 Executive Scorecard

| Assessment Dimension | Rating (0–10) | Status | Key Observation |
|:---|:---:|:---:|:---|
| **Cryptographic Primitives** | **9.0** | Robust | OWASP-compliant Argon2id parameters; SQLCipher page encryption. |
| **IPC & Sandboxing Boundaries** | **6.5** | High Risk | Strict capability declarations undermined by `SEC-01` arbitrary file read. |
| **Sensitive Memory Hygiene** | **7.0** | Needs Hardening | Envelope keys protected, but temporary buffers and V8 memory unzeroized. |
| **Plugin & Extensibility Blueprint** | **8.5** | High Maturity | Extism (Wasmtime) architecture guarantees linear memory isolation. |
| **Update Lifecycle & Distribution** | **7.2** | Moderate Risk | In-app Minisign is strong, but distribution scripts lack signature validation. |
| **Desktop Maturity & Observability** | **6.0** | Incomplete | Zero local crash logging; missing OS sleep lock; transactional migration gaps. |
| **Accessibility (a11y) & UX Navigation** | **6.5** | Needs Work | Vim hotkeys present, but ARIA widget roles omitted in tabs and dropdowns. |
| **Supply Chain & Dependency Health** | **7.0** | Moderate Risk | 10 NPM advisories in `pnpm-lock.yaml`; 1 unsoundness advisory in `Cargo.lock`. |
| **Overall Platform Maturity** | **7.4 / 10** | **Grade B+** | Solid core requiring focused platform hardening before general availability. |

---

## 2. R1: Platform Integrity & Cryptographic Security Audit

### 2.1 Tauri v2 Sandboxing & Capability Manifest Analysis

Finnca's IPC surface is governed by Tauri v2's capability engine (`src-tauri/capabilities/` and `src-tauri/tauri.conf.json`). The architecture demonstrates significant security maturity by revoking blanket filesystem access (`fs:default`) and enforcing domain-scoped capability manifests:

- `capabilities/vault.json`: Restricts vault creation, discovery, and lifecycle operations.
- `capabilities/ledger.json`: Grants access to posting validation, account trees, and journal drafting.
- `capabilities/security.json`: Governs auto-lock policies and activity timestamps.
- `capabilities/reconcile.json`: Confines bank statement imports and rule matching.

#### Architectural Vulnerabilities in Capability Design
1. **False Sense of Sandbox Isolation**: While `capabilities/reconcile.json` permits `reconcile:allow-read-statement-file-cmd`, the underlying Rust handler in `src-tauri/src/reconcile/commands.rs` executes unrestricted POSIX file I/O. Capability scoping at the manifest layer is meaningless if the backend command fails to validate filesystem boundaries.
2. **Redundant Capabilities**: `capabilities/vault.json:6` grants `opener:default`, yet `src-tauri/src/vault/commands.rs:735-757` bypasses the opener plugin entirely, invoking `std::process::Command::new("xdg-open")` directly.
3. **Dead Plugins in Runtime**: `src-tauri/src/lib.rs:248` initializes `tauri_plugin_fs::init()`, yet no capability manifest grants access to `fs` plugin commands, unnecessarily enlarging the attack surface.

---

### 2.2 Vulnerability Catalog (SEC-01 through SEC-12)

```
========================================================================================
VULNERABILITY CATALOG SUMMARY MATRIX
========================================================================================
ID       SEVERITY       DOMAIN                 FILE & LINES
----------------------------------------------------------------------------------------
SEC-01   CRITICAL/HIGH  IPC Sandboxing         src-tauri/src/reconcile/commands.rs:293-321
SEC-02   HIGH           Database Encryption    src-tauri/src/db/mod.rs:15-24
SEC-03   HIGH           Memory Hygiene         src-tauri/src/crypto/envelope.rs:56-68
                                               src-tauri/src/db/mod.rs:11-12,49-55
                                               src-tauri/src/auth/commands.rs:18,63-64
SEC-04   HIGH           Binary Hardening       src-tauri/Cargo.toml:54-65
SEC-05   MEDIUM         Network Isolation      src-tauri/tauri.conf.json:29
                                               src/lib/features/settings/fxSync.ts:8
SEC-06   MEDIUM         Cryptographic Binding  src-tauri/src/crypto/envelope.rs:23-38,40-68
SEC-07   MEDIUM         Frontend Memory        src/lib/features/vault/components/LoginForm.svelte:135-162
SEC-08   MEDIUM         Platform Credentials   src-tauri/src/security/commands.rs:8-19
                                               src/routes/app/+layout.svelte:69-73
SEC-09   LOW            Attack Surface         src-tauri/src/lib.rs:248
                                               src-tauri/Cargo.toml:29
SEC-10   LOW            IPC Redundancy         src-tauri/capabilities/vault.json:6
                                               src-tauri/src/vault/commands.rs:735-757
SEC-11   LOW            Denial of Service      src-tauri/src/vault/commands.rs:480-505
SEC-12   HIGH (NPM)     Supply Chain           package.json, pnpm-lock.yaml
========================================================================================
```

---

#### SEC-01: Arbitrary Local File Read via `read_statement_file_cmd`
- **Severity**: **CRITICAL / HIGH** (CVSS 8.6 — `CVSS:3.1/AV:L/AC:L/PR:N/UI:R/S:C/C:H/I:N/A:N`)
- **Citations**: `src-tauri/src/reconcile/commands.rs:293-321`
- **Mechanism**:
  ```rust
  // Vulnerable pattern in src-tauri/src/reconcile/commands.rs
  pub fn read_statement_file_cmd(state: State<'_, AppState>, path: String) -> Result<String, AppError> {
      let _ = state.get_db()?;
      let trimmed = path.trim();
      let p = std::path::Path::new(trimmed);
      if !p.exists() || !p.is_file() { ... }
      let content = std::fs::read_to_string(p)?;
      Ok(content)
  }
  ```
  While `vault::commands::validate_safe_export_target` rigorously sanitizes export paths, `read_statement_file_cmd` accepts any path string without canonicalization, boundary validation, or extension checks.
- **Exploitation Vector & Defense Pitfalls**: Any frontend component, rogue script, or compromised dependency executing in the webview can invoke `commands.readStatementFileCmd("/etc/shadow")` or `commands.readStatementFileCmd("~/.ssh/id_rsa")`. The contents (up to 5 MB) are read into memory and returned directly to the webview caller.
  - *Dead Symlink Check Pitfall*: Inspecting `symlink_metadata` only *after* invoking `path.canonicalize()` is completely ineffective because `canonicalize()` resolves symlinks to their destination targets, causing `meta.file_type().is_symlink()` to evaluate to `false` permanently. Symlink inspection MUST occur on the raw uncanonicalized path before resolving.
  - *Extension Whitelist Leakage*: Broad whitelists containing `.txt` permit reading user documents, password lists, and private keys located anywhere outside blocked directories. The extension whitelist must be strictly confined to banking statement formats.
  - *Platform Parity Gap*: Denying only Unix root paths leaves Windows system targets (`C:\Windows`, `C:\Program Files`, `C:\ProgramData`) vulnerable.
  - *Legitimate User Vaults*: Blindly blocking all dotfiles breaks user setups (e.g. `~/.local/share/finnca/`). Protection must specifically target sensitive credential roots (`~/.ssh`, `~/.gnupg`, `~/.aws`, `~/.config`) while permitting legitimate user data.
- **Remediation**: Implement mandatory pre-canonicalization symlink checking on uncanonicalized input paths (`path.symlink_metadata()`), post-canonicalization path resolution, cross-platform system directory blocklists (Linux `/etc`, `/root`, `/var`, `/proc`; Windows `C:\Windows`, `C:\Program Files`, `C:\ProgramData`), sensitive credential root blocking (`~/.ssh`, `~/.gnupg`, `~/.aws`, `~/.config`) while permitting legitimate user vaults, and strict statement format extension whitelisting (`.csv`, `.ofx`, `.qfx`, `.mt940`, `.sta`), removing broad `.txt`.

---

#### SEC-02: Forensic Database Residue & Disk Spillage via Missing `PRAGMA temp_store = MEMORY;`
- **Severity**: **MEDIUM / HIGH** (CVSS 6.5 — `CVSS:3.1/AV:L/AC:L/PR:L/UI:N/S:U/C:H/I:N/A:N`)
- **Citations**: `src-tauri/src/db/mod.rs:15-24`
- **Mechanism**: SQLite defaults `temp_store` to file storage (`0` / `FILE`). When complex ledger reports (historical trends, monthly cashflow rollups, multi-thousand transaction sorting) execute, SQLite spills temporary tables, materialized indices, and transient B-trees to disk in `/tmp` or `%TEMP%`.
- **Impact & Cryptographic Nuance**: On keyed SQLCipher connections, SQLCipher transparently encrypts temporary database files and b-trees with a random KDF salt (`sqlcipher_codec_ctx_init_kdf_salt`). However, leaving `temp_store = FILE` leaves physical artifacts on host disk, creates forensic residue across OS crashes, and incurs significant I/O performance penalties. Note that the pragma `PRAGMA cipher_default_temp_store = MEMORY;` does not exist in SQLCipher/SQLite and must not be used.
- **Remediation**: Execute `PRAGMA temp_store = MEMORY;` immediately upon opening the database connection. This ensures all transient sorting b-trees and temporary tables remain exclusively in volatile RAM, eliminating disk I/O forensic traces and maximizing query throughput.

---

#### SEC-03: Incomplete Heap Zeroization for DEK, Hex Key, Snapshot ATTACH, and Master Passwords
- **Severity**: **HIGH** (CVSS 7.0 — `CVSS:3.1/AV:L/AC:H/PR:L/UI:N/S:U/C:H/I:N/A:N`)
- **Citations**:
  - `src-tauri/src/crypto/envelope.rs:56-68`: Decrypted DEK vector is deallocated without zeroization.
  - `src-tauri/src/db/mod.rs:11-12`: `dek_hex` is formatted into a plain heap `String`.
  - `src-tauri/src/db/mod.rs:49-55`: `export_encrypted_snapshot` creates unzeroized heap `String` for `target_dek_hex` and ATTACH SQL statement.
  - `src-tauri/src/auth/commands.rs:18,63-64`: Plaintext password `String` arguments passed across async boundaries.
- **Mechanism**:
  In `envelope.rs`, `cipher.decrypt()` returns a standard `Vec<u8>`. While the final `dek` array is wrapped in `Zeroizing<[u8; 32]>`, the intermediate `decrypted` vector is freed by the Rust global allocator without overwriting memory pages. In `src-tauri/src/db/mod.rs:11-12`, `hex::encode(dek)` creates an unzeroized 64-character heap string.
  Crucially, in `src-tauri/src/db/mod.rs:49-55` (`export_encrypted_snapshot`), `let target_dek_hex = hex::encode(target_dek);` and `format!("ATTACH DATABASE '{safe_dest}' AS backup KEY \"x'{target_dek_hex}'\";")` allocate plaintext backup keys on the heap that are dropped without zeroization.
  Furthermore, under the hood in `rusqlite::Connection::execute_batch`, the SQL `&str` is converted into an internal C-compatible `std::ffi::CString`, which allocates unzeroized heap memory at the C runtime layer.
- **Impact**: Plaintext key material, backup snapshot keys, and master passwords linger in deallocated heap memory, vulnerable to memory dumping, process inspection (`/proc/<pid>/mem`), or cold-boot extraction.
- **Remediation**: Wrap all decrypted buffers, hex strings, snapshot backup keys, and password arguments in `zeroize::Zeroizing` or `secrecy::SecretString`. For zero-leakage key passing to SQLCipher, architect a migration from `PRAGMA key` / `ATTACH KEY` SQL strings toward direct invocation of the C API `sqlite3_key_v2` / `sqlite3_rekey_v2`.

---

#### SEC-04: Unhardened Release Build Profile in `src-tauri/Cargo.toml`
- **Severity**: **HIGH** (CVSS 6.8 — `CVSS:3.1/AV:L/AC:L/PR:N/UI:N/S:U/C:L/I:L/A:L`)
- **Citations**: `src-tauri/Cargo.toml:54-65`, `src-tauri/Cargo.toml:24`
- **Mechanism**: `Cargo.toml` specifies `[profile.dev]` and `[profile.test]`, but completely omits `[profile.release]`. Consequently, release builds inherit Rust default settings: Link-Time Optimization (LTO) is disabled, code is split across 16 codegen units, `panic = "unwind"` retains full unwinding tables and landing pads, and debug symbols remain unstripped. Furthermore, `tauri = { version = "2", features = ["devtools"] }` unconditionally compiles webview inspector capabilities into production binaries.
- **Impact**: Binary inspection using tools like Ghidra or IDA Pro is trivialized. Attackers can hook internal symbols, analyze double-entry logic, or open the Tauri devtools console in production to invoke privileged IPC commands.
- **Remediation**: Configure `[profile.release]` with `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`, and make `devtools` an opt-in Cargo feature.

---

#### SEC-05: Webview CSP `connect-src` External HTTP Leakage
- **Severity**: **MEDIUM** (CVSS 5.3 — `CVSS:3.1/AV:N/AC:H/PR:N/UI:R/S:U/C:H/I:N/A:N`)
- **Citations**: `src-tauri/tauri.conf.json:29`, `src/lib/features/settings/fxSync.ts:8`
- **Mechanism**:
  ```json
  "csp": "... connect-src 'ipc:' https://ipc.localhost https://open.er-api.com; ..."
  ```
  `fxSync.ts` directly calls `fetch('https://open.er-api.com/v6/latest/USD')` from the frontend renderer.
- **Impact**: Violates the core system axiom of an airgapped, local-only ledger. Any frontend prototype pollution or cross-site scripting flaw could exfiltrate user account data by appending encoded payloads as query parameters to `open.er-api.com`.
- **Remediation**: Remove `https://open.er-api.com` from CSP `connect-src`, completely airgapping the webview. Relocate currency exchange synchronization to a dedicated, certificate-pinned Rust IPC command (`sync_fx_rates_cmd`).

---

#### SEC-06: ChaCha20-Poly1305 Empty Associated Authenticated Data (AAD)
- **Severity**: **MEDIUM** (CVSS 4.8 — `CVSS:3.1/AV:L/AC:H/PR:L/UI:N/S:U/C:L/I:L/A:N`)
- **Citations**: `src-tauri/src/crypto/envelope.rs:23-38,40-68`
- **Mechanism**: ChaCha20-Poly1305 supports AEAD (Authenticated Encryption with Associated Data). In `wrap_dek` and `unwrap_dek`, the envelope ciphertext is created and validated with empty AAD (`b""`).
- **Impact**: The encrypted DEK envelope is not cryptographically bound to the vault ID, metadata version, or creation timestamp. An attacker could swap key envelopes across different vaults encrypted with the same password without triggering authentication failure.
- **Remediation**: Bind the unique `vault_id` and schema version as AAD in both `wrap_dek` and `unwrap_dek`.

---

#### SEC-07: Svelte 5 Login Form Password Retention in V8 Heap
- **Severity**: **MEDIUM** (CVSS 4.4 — `CVSS:3.1/AV:L/AC:H/PR:L/UI:N/S:U/C:H/I:N/A:N`)
- **Citations**: `src/lib/features/vault/components/LoginForm.svelte:135-162`
- **Mechanism**: The master password is bound to a Svelte 5 rune `let password = $state('')`. Upon successful authentication, the component triggers navigation (`goto(resolve('/app'))`) without explicitly clearing the rune value (`password = ''`).
- **Impact**: The plaintext master password persists indefinitely in the JavaScript V8 heap until non-deterministic garbage collection occurs. A memory dump of the webview process can expose the vault master password.
- **Remediation**: Wrap authentication in a `try ... finally` block that explicitly resets `password = ''` and zeroes the DOM input element.

---

#### SEC-08: Non-Cryptographic `boot_id` Fast-Unlock Stub
- **Severity**: **MEDIUM** (CVSS 4.0 — `CVSS:3.1/AV:L/AC:L/PR:L/UI:N/S:U/C:N/I:L/A:N`)
- **Citations**: `src-tauri/src/security/commands.rs:8-19`, `src/routes/app/+layout.svelte:69-73`
- **Mechanism**: Fast-unlock reads Linux `/proc/sys/kernel/random/boot_id` and stores it as a plain string in `~/.config/finnca/finnca.json`. Because the Rust backend process drops the SQLite connection and DEK memory upon termination, `boot_id` provides zero cryptographic recovery capability across app restarts.
- **Impact**: The feature creates user confusion regarding session persistence and provides no genuine cryptographic or biometric fast-unlock mechanism.
- **Remediation**: Implement a Dual Envelope Architecture integrating OS Credential Stores (`keyring = "3"`), macOS Touch ID, Windows Hello/DPAPI, and Linux Secret Service.

---

#### SEC-09: Unused `tauri-plugin-fs` Runtime Initialization
- **Severity**: **LOW** (CVSS 2.5)
- **Citations**: `src-tauri/src/lib.rs:248`, `src-tauri/Cargo.toml:29`
- **Mechanism**: The `tauri-plugin-fs` plugin is registered during Tauri setup, but no capabilities grant access to its endpoints. It represents dead code and unnecessary IPC surface.
- **Remediation**: Remove `tauri-plugin-fs` from `Cargo.toml` and `lib.rs`.

---

#### SEC-10: Redundant Opener Capability vs Raw Process Execution
- **Severity**: **LOW** (CVSS 2.5)
- **Citations**: `src-tauri/capabilities/vault.json:6`, `src-tauri/src/vault/commands.rs:735-757`
- **Mechanism**: The capability manifest authorizes `opener:default`, but `open_vault_folder` spawns OS file managers using raw `std::process::Command::new("xdg-open")`.
- **Remediation**: Standardize folder opening through the official Tauri opener plugin or remove the unused capability declaration.

---

#### SEC-11: Zip Bomb Denial of Service Exposure in Vault Import
- **Severity**: **LOW** (CVSS 3.3 — `CVSS:3.1/AV:L/AC:L/PR:N/UI:R/S:U/C:N/I:N/A:L`)
- **Citations**: `src-tauri/src/vault/commands.rs:480-505`
- **Mechanism**: In `unpack_finnca_archive`, zip entries are unpacked via `std::io::copy(&mut entry, &mut outfile)` without tracking cumulative uncompressed bytes or limiting total entry counts.
- **Impact**: Importing a crafted 20 KB zip bomb that expands to 50 GB can exhaust host disk space and freeze the application.
- **Remediation**: Enforce a strict ceiling of 500 MB cumulative extracted bytes and a maximum of 50 archive entries.

---

#### SEC-12: Frontend Supply Chain Vulnerabilities (`pnpm audit`)
- **Severity**: **HIGH** (NPM Ecosystem)
- **Citations**: `package.json`, `pnpm-lock.yaml`
- **Mechanism**: 10 known advisories affecting `brace-expansion` (uncontrolled recursion DoS) and `devalue` (quadratic expansion and shared memory serialization). Detailed in Section 6.

---

### 2.3 Concrete Production Remediation Blueprints

#### Remediation Blueprint 1: Hardened Path Gateway for `read_statement_file_cmd` (`SEC-01`)
```rust
// Proposed fix in src-tauri/src/reconcile/commands.rs
use std::path::{Path, PathBuf};
use crate::shared::AppError;

const MAX_STATEMENT_FILE_BYTES: u64 = 5 * 1024 * 1024; // 5 MB ceiling
// Strictly whitelist financial statement formats; broad formats like '.txt' are forbidden
const ALLOWED_STATEMENT_EXTENSIONS: &[&str] = &["csv", "ofx", "qfx", "mt940", "sta"];

#[tauri::command]
#[specta::specta]
pub fn read_statement_file_cmd(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, AppError> {
    // 1. Ensure user has an authenticated, unlocked vault session
    let _ = crate::vault::commands::require_session(&state)
        .map_err(AppError::InvalidInput)?;

    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("Statement path cannot be empty".into()));
    }

    let p = Path::new(trimmed);
    if !p.is_absolute() {
        return Err(AppError::InvalidInput("Statement path must be absolute".into()));
    }

    // 2. Pre-canonicalization symlink check: verify the input path itself is NOT a symlink.
    // Inspecting symlink metadata ONLY after canonicalize() is dead code because
    // canonicalize() resolves the target, causing is_symlink() to always evaluate to false.
    let input_meta = std::fs::symlink_metadata(p)
        .map_err(|e| AppError::NotFound(format!("Statement file not found: {e}")))?;
    if input_meta.file_type().is_symlink() {
        return Err(AppError::InvalidInput("Reading symlinked statement files is strictly forbidden".into()));
    }

    // 3. Canonicalize path to resolve relative components ('..') and obtain normalized path
    let canonical = p.canonicalize()
        .map_err(|e| AppError::NotFound(format!("Invalid file path: {e}")))?;
    let canonical_str = canonical.to_string_lossy();

    // 4. Post-canonicalization symlink check (defense-in-depth against TOCTOU race)
    let meta = std::fs::symlink_metadata(&canonical)?;
    if meta.file_type().is_symlink() {
        return Err(AppError::InvalidInput("Reading symlinked statement files is strictly forbidden".into()));
    }

    // 5. Cross-platform system and credential directory protections
    #[cfg(unix)]
    {
        let forbidden_system = [
            "/etc", "/root", "/boot", "/sys", "/proc", "/bin", "/sbin", "/usr", "/dev", "/var",
        ];
        if canonical_str == "/" || forbidden_system.iter().any(|f| canonical_str == *f || canonical_str.starts_with(&format!("{f}/"))) {
            return Err(AppError::InvalidInput("Access to system directory is strictly prohibited".into()));
        }

        // Block sensitive credential/configuration roots while allowing legitimate user vaults
        if let Some(home) = dirs::home_dir() {
            let home_str = home.to_string_lossy();
            let forbidden_user_roots = [
                format!("{home_str}/.ssh"),
                format!("{home_str}/.gnupg"),
                format!("{home_str}/.aws"),
                format!("{home_str}/.config"),
            ];
            if forbidden_user_roots.iter().any(|d| canonical_str == *d || canonical_str.starts_with(&format!("{d}/"))) {
                return Err(AppError::InvalidInput("Access to sensitive user configuration/credential directory is prohibited".into()));
            }
        }
    }

    #[cfg(windows)]
    {
        let canonical_upper = canonical_str.to_uppercase();
        let forbidden_windows = [
            "C:\\WINDOWS",
            "C:\\PROGRAM FILES",
            "C:\\PROGRAM FILES (X86)",
            "C:\\PROGRAMDATA",
        ];
        if forbidden_windows.iter().any(|w| canonical_upper == *w || canonical_upper.starts_with(&format!("{w}\\"))) {
            return Err(AppError::InvalidInput("Access to Windows system directory is strictly prohibited".into()));
        }

        // Block sensitive Windows user credential vaults
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let prof = user_profile.to_string_lossy().to_uppercase();
            let forbidden_win_dirs = [
                format!("{prof}\\.SSH"),
                format!("{prof}\\.AWS"),
                format!("{prof}\\APPDATA\\LOCAL\\MICROSOFT\\CREDENTIALS"),
                format!("{prof}\\APPDATA\\ROAMING\\MICROSOFT\\PROTECT"),
            ];
            if forbidden_win_dirs.iter().any(|d| canonical_upper == *d || canonical_upper.starts_with(&format!("{d}\\"))) {
                return Err(AppError::InvalidInput("Access to sensitive user credential store is prohibited".into()));
            }
        }
    }

    // 6. Validate file extension against strictly allowed statement formats
    let ext = canonical.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !ALLOWED_STATEMENT_EXTENSIONS.contains(&ext.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "Unsupported statement format '.{ext}'. Allowed: {}",
            ALLOWED_STATEMENT_EXTENSIONS.join(", ")
        )));
    }

    // 7. Verify file size ceiling
    if meta.len() > MAX_STATEMENT_FILE_BYTES {
        return Err(AppError::InvalidInput(format!(
            "File size ({} bytes) exceeds the 5 MB limit", meta.len()
        )));
    }

    let content = std::fs::read_to_string(&canonical)?;
    Ok(content)
}
```

---

#### Remediation Blueprint 2: Hardened SQLCipher Pragmas (`SEC-02`)
```rust
// In src-tauri/src/db/mod.rs open_vault_db()
pub fn open_vault_db(path: &Path, dek: &[u8; 32]) -> Result<Connection, AppError> {
    let mut conn = Connection::open(path)?;

    // Zeroizing wrapper for temporary hex string
    let mut dek_hex = zeroize::Zeroizing::new(hex::encode(dek));
    conn.execute_batch(&format!("PRAGMA key = \"x'{dek_hex}'\";"))?;

    // Set performance, memory security, and in-memory disk isolation pragmas.
    // NOTE: SQLCipher provides transparent page encryption for temporary b-trees
    // on keyed connections using internal codec KDF salt (sqlcipher_codec_ctx_init_kdf_salt).
    // PRAGMA temp_store = MEMORY is retained as the authoritative SQLite directive to keep
    // temporary indices strictly in volatile RAM, eliminating disk artifacts and boosting speed.
    // (PRAGMA cipher_default_temp_store is non-existent in SQLCipher/SQLite and omitted).
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

    // Run quick integrity check upon opening
    let check: String = conn.query_row("PRAGMA quick_check;", [], |r| r.get(0))?;
    if check != "ok" {
        return Err(AppError::Database(format!("SQLCipher quick_check failed: {check}")));
    }

    Ok(conn)
}
```

---

#### Remediation Blueprint 3: Zeroize Sensitive Cryptographic Buffers & Snapshot Keys (`SEC-03`)
```rust
// In src-tauri/src/crypto/envelope.rs unwrap_dek()
use zeroize::{Zeroize, Zeroizing};

pub fn unwrap_dek(
    kek: &[u8; 32],
    nonce_bytes: &[u8; 12],
    ciphertext_bytes: &[u8],
    vault_id: &str, // Cryptographic AAD binding (SEC-06)
) -> Result<Zeroizing<[u8; 32]>, AppError> {
    let cipher = ChaCha20Poly1305::new(kek.into());
    let nonce = Nonce::from_slice(nonce_bytes);

    let mut decrypted = Zeroizing::new(
        cipher
            .decrypt(
                nonce,
                chacha20poly1305::aead::Payload {
                    msg: ciphertext_bytes,
                    aad: vault_id.as_bytes(),
                },
            )
            .map_err(|_| AppError::Crypto("Invalid password or corrupted vault envelope".into()))?,
    );

    if decrypted.len() != 32 {
        return Err(AppError::Crypto("Decrypted database key has invalid length".into()));
    }

    let mut dek = Zeroizing::new([0u8; 32]);
    dek.copy_from_slice(&decrypted);
    Ok(dek)
}

// In src-tauri/src/db/mod.rs export_encrypted_snapshot()
pub fn export_encrypted_snapshot(
    conn: &Connection,
    dest_path: &Path,
    target_dek: &[u8; 32],
) -> Result<(), AppError> {
    let dest_str = dest_path
        .to_str()
        .ok_or_else(|| AppError::InvalidInput("Destination path is not valid UTF-8".into()))?;

    // Hardened zeroization for target DEK hex and ATTACH query buffer
    let mut target_dek_hex = Zeroizing::new(hex::encode(target_dek));
    let user_version: u32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;

    let safe_dest = dest_str.replace('\'', "''");
    let mut attach_sql = Zeroizing::new(format!(
        "ATTACH DATABASE '{safe_dest}' AS backup KEY \"x'{target_dek_hex}'\";"
    ));
    conn.execute_batch(&attach_sql)?;

    let export_res = conn.execute_batch(&format!(
        "SELECT sqlcipher_export('backup'); PRAGMA backup.user_version = {user_version};"
    ));
    let detach_res = conn.execute_batch("DETACH DATABASE backup;");

    export_res?;
    detach_res?;
    Ok(())
}
```

> **Architectural Note on `sqlite3_key_v2` Migration**:
> In `rusqlite`, `execute_batch` converts Rust `&str` into an internal `std::ffi::CString` that is allocated on the C heap without zeroization. For complete elimination of key residue on the heap, Phase 2 migration will utilize direct FFI calls to `sqlite3_key_v2(db, db_name, key, key_len)` and `sqlite3_rekey_v2`, passing raw byte slices directly into SQLCipher's internal codec without SQL string formatting or intermediate heap CString allocations.

---

#### Remediation Blueprint 4: Production Release Profile Hardening (`SEC-04`)
```toml
# Append to src-tauri/Cargo.toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
overflow-checks = true

# Make devtools strictly opt-in
[features]
default = []
devtools = ["tauri/devtools"]
```

---

## 3. R2: Plugin System & Extensibility Blueprint

### 3.1 Runtime Technology Evaluation

To support community bank statement parsers, regional financial reports, and external sync adapters without compromising vault encryption or ledger invariants, six runtime approaches were systematically evaluated:

```
+--------------------------------------------------------------------------------------------------+
| CANDIDATE RUNTIME COMPARISON ACROSS 6 CRITICAL ARCHITECTURAL DIMENSIONS                           |
+--------------------------------------------------------------------------------------------------+
| Dimension           | Extism (Wasmtime)  | Standalone Wasmtime| QuickJS (rquickjs) | Native (libloading) |
+---------------------+--------------------+--------------------+--------------------+---------------------+
| 1. Memory Isolation | Grade A+ (Linear)  | Grade A+ (Linear)  | Grade C (Host Heap)| Grade F (Zero Guard)|
| 2. Execution Budget | Built-in (Fuel/ms) | Manual (Fuel/ms)   | Hook-based only    | None (Unstoppable)  |
| 3. Cold/Warm Latency| 2.5ms / < 0.2ms    | 1.8ms / < 0.1ms    | < 0.5ms / < 0.05ms | < 0.01ms / 5ns      |
| 4. Polyglot DX      | 16+ PDKs (RS/TS/Go)| Complex ABI/WIT    | JS/TS Only         | C-ABI Only          |
| 5. Binary Footprint | +3.8 MB            | +3.2 MB            | +1.5 MB            | < 50 KB             |
| 6. Final Verdict    | RECOMMENDED WINNER | Viable Alternative | REJECTED (Unsafe)  | REJECTED (Fatal)    |
+--------------------------------------------------------------------------------------------------+
```

#### Detailed Technology Rationale
1. **Extism (Bytecode Alliance Wasmtime Engine) — WINNER**:
   - **Isolation**: Runs inside a dedicated WebAssembly linear memory sandbox (32 MB default cap). Plugins have zero access to host memory, SQLite pointers, or KEK/DEK keys.
   - **Deterministic Budgeting**: Instruction fuel consumption (`Store::set_fuel`) and epoch deadlines guarantee that infinite loops or CPU exhaustion cannot freeze the Tauri UI.
   - **Polyglot Developer Experience**: Community plugin authors can write statement parsers in TypeScript, Go, Python, or Rust using standardized PDKs.
2. **Embedded QuickJS / Lua — REJECTED**:
   - Execute in host memory heap. A buffer overflow or memory corruption bug inside the C interpreter compromises host process memory, exposing decrypted database pages.
   - Scripting ecosystems lack robust modern parsers for complex banking standards (e.g., ISO 20022 CAMT.053 XML or PDF extraction).
3. **Native Shared Libraries (`libloading`) — FORBIDDEN**:
   - Dynamic libraries (`.so`, `.dll`) share full memory space and execution context with the host binary. A malicious plugin could immediately scan memory for the 32-byte DEK, extract plaintext records, or inject unbalanced journal entries.

---

### 3.2 Permission Boundary Matrix

Plugins declare required capabilities in their manifest. The Finnca host enforces these boundaries at runtime through an isolated capability gate:

| Capability Scope | Identifier | Host Enforcement Mechanism | Default | Audit Severity if Violated |
|:---|:---|:---|:---:|:---:|
| **Filesystem** | `fs:none` | Guest has zero filesystem syscalls; WASI fs access disabled. | **YES** | None |
| | `fs:scoped_read` | Only files explicitly passed via host memory buffer (max 5 MB). Guest cannot initiate path reads. | Opt-in | High (Prevented by host buffer injection) |
| | `fs:temp` | Sandboxed temporary scratch directory, auto-purged on plugin unload. | Opt-in | Medium |
| | `fs:write` | **BLOCKED**. Plugins cannot write directly to OS disk. Only return buffers for host to save. | **NEVER** | Critical |
| **Network** | `net:none` | Network sockets blocked; WASI networking disabled; Extism HTTP disabled. | **YES** | None |
| | `net:outbound` | Explicit domain whitelist (e.g., `["api.exchangerate.host"]`). HTTPS (port 443) only. | Opt-in | High (Requires user prompt at install time) |
| **Ledger** | `ledger:none` | Zero ledger access (for statement parsers and utility plugins). | **YES** | None |
| | `ledger:read_aggregate`| Pre-computed summary data passed (account names, monthly totals, net worth). No raw transaction PII. | Opt-in | Medium |
| | `ledger:read_full` | Full account hierarchy and posting history. Requires high-privilege user permission prompt. | Opt-in | High |
| | `ledger:propose_draft` | Plugin outputs `DraftJournalEntry` structures for user review in UI. | Opt-in | Medium |
| | `ledger:direct_write` | **FORBIDDEN BY ARCHITECTURE**. Host API does NOT expose any direct insert/update/delete. | **NEVER** | Critical (Inadmissible) |
| **Notifications**| `notify:system` | Emits in-app toasts. Rate-limited to 1 msg / 5 sec. Prefixed with `[Plugin: <name>]`. Cannot spoof system alerts. | Opt-in | Low |
| **Crypto Secrets**| `crypto:*` | **ABSOLUTELY PROHIBITED**. Plugins have zero access to master key, KEK, DEK, salt, or SQLCipher connection. | **NEVER** | Fatal / Inadmissible |

---

### 3.3 Zero-Float & Ledger Invariant Enforcement Gateway

To prevent third-party code from contaminating the ledger, all data crossing the host-guest boundary passes through a strict validation pipeline:

```
[Untrusted Plugin Output]
           │
           ▼
┌────────────────────────────────────────────────────────┐
│           Zero-Float Invariant Gateway                 │
│  - Unconditionally rejects ALL IEEE-754 floats         │
│    (is_f64() rejected: no fract() == 0.0 allowance)    │
│  - Rejects string-disguised decimals ("15000.50")      │
│  - Rejects float literals ("NaN", "Infinity")          │
│  - Requires integer minor units (i64 / u64)            │
└──────────────────────────┬─────────────────────────────┘
                           │ Validated
                           ▼
┌────────────────────────────────────────────────────────┐
│     Double-Entry & Debit-First Validation Gateway      │
│  - validate_postings_balance_with_context()            │
│    * Verifies sum(Debit) == sum(Credit) == 0           │
│  - validate_plugin_draft_entry()                       │
│    * Enforces Debit-First leg order for 2-leg drafts:  │
│      Index 0 = Debit (amount >= 0)                     │
│      Index 1 = Credit (amount <= 0)                    │
└──────────────────────────┬─────────────────────────────┘
                           │ Validated
                           ▼
┌────────────────────────────────────────────────────────┐
│              Interactive User Approval UI              │
│  - Never writes directly to SQLite                     │
│  - Emits Draft Transaction for user review in UI       │
└────────────────────────────────────────────────────────┘
```

#### Hardened Zero-Float Validator Implementation
```rust
/// Scans raw JSON bytes across the host-plugin boundary to guarantee zero float contamination.
/// Unconditionally rejects all IEEE-754 floats (including whole floats like 15000.0 or scientific notation 1e2),
/// rejects string-disguised decimal floats ("15000.50", "NaN", "Infinity"), and requires
/// all numeric financial fields to be integer minor units.
pub fn validate_no_raw_floats(bytes: &[u8]) -> Result<(), AppError> {
    let val: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| AppError::InvalidInput(format!("Malformed JSON output from plugin: {e}")))?;

    fn scan_value(v: &serde_json::Value, path: &str) -> Result<(), AppError> {
        match v {
            serde_json::Value::Number(n) => {
                // Reject ANY floating-point number unconditionally.
                // Allowing f.fract() == 0.0 is an invariant hazard that admits whole-number floats (15000.0),
                // scientific notation (1e2), and 53-bit mantissa precision-loss values.
                if n.is_f64() {
                    return Err(AppError::InvalidInput(format!(
                        "ERR_PLUGIN_FLOAT_VIOLATION: Unconditional IEEE-754 float detected at '{path}': {n}. All financial values must be signed integer minor units (i64)."
                    )));
                }
            }
            serde_json::Value::String(s) => {
                let trimmed = s.trim();
                // Reject string-encoded IEEE-754 special values and disguised decimal numbers
                if trimmed.eq_ignore_ascii_case("nan")
                    || trimmed.eq_ignore_ascii_case("infinity")
                    || trimmed.eq_ignore_ascii_case("-infinity")
                    || trimmed.eq_ignore_ascii_case("+infinity")
                {
                    return Err(AppError::InvalidInput(format!(
                        "ERR_PLUGIN_FLOAT_VIOLATION: Disguised non-numeric float literal detected at '{path}': '{s}'."
                    )));
                }
                // Detect strings concealing decimal points or scientific notation
                if (trimmed.contains('.') || trimmed.contains('e') || trimmed.contains('E'))
                    && trimmed.parse::<f64>().is_ok()
                {
                    return Err(AppError::InvalidInput(format!(
                        "ERR_PLUGIN_FLOAT_VIOLATION: String-disguised decimal float detected at '{path}': '{s}'. Monetary fields must not encode fractional values."
                    )));
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, elem) in arr.iter().enumerate() {
                    scan_value(elem, &format!("{path}[{i}]"))?;
                }
            }
            serde_json::Value::Object(obj) => {
                for (k, val) in obj.iter() {
                    scan_value(val, &format!("{path}.{k}"))?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    scan_value(&val, "root")
}
```

#### Host-Enforced Debit-First Gateway Implementation
While core ledger validation `validate_postings_balance_with_context` validates commodity balance (`sum(postings) == 0`), it does not govern leg indexing. In accordance with `AGENTS.md § 2.5`, all 2-leg entries submitted by plugins must strictly satisfy the **Debit-First Standard** before being presented to the user:

```rust
/// Enforces Debit-First standard and double-entry balance on plugin-proposed draft entries.
pub fn validate_plugin_draft_entry(
    entry: &DraftJournalEntry,
    default_currency: &str,
    default_fx_rate: i64,
) -> Result<(), AppError> {
    // 1. Core double-entry balance and currency verification
    crate::ledger::validation::validate_postings_balance_with_context(
        &entry.postings,
        default_currency,
        default_fx_rate,
    )?;

    // 2. Strict Debit-First standard for 2-leg transactions:
    // Index 0 = Debit (Destination Account, amount >= 0)
    // Index 1 = Credit (Source Account, amount <= 0)
    if entry.postings.len() == 2 {
        let leg0 = &entry.postings[0];
        let leg1 = &entry.postings[1];

        if leg0.amount < 0 || leg1.amount > 0 {
            return Err(AppError::InvalidInput(format!(
                "ERR_PLUGIN_DEBIT_FIRST_VIOLATION: 2-leg entry violates Debit-First standard. \
                Index 0 must be Debit (amount >= 0, found {}), Index 1 must be Credit (amount <= 0, found {}).",
                leg0.amount, leg1.amount
            )));
        }
    }

    Ok(())
}
```

---

### 3.4 Concrete Extension Point Specifications

#### Extension Point A: Statement Parsers (`StatementParser`)
Replaces frontend-only `Papa.parse` with sandboxed parsers supporting CSV, OFX, CAMT.053, and PDF bank statements.

```rust
// Rust Host ABI Contract for Statement Parsers
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
    pub date: String,             // ISO-8601 "YYYY-MM-DD"
    pub amount: i64,              // Minor units (positive for inflow, negative for outflow)
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

#### Extension Point B: Custom Financial Reports (`FinancialReport`)
Enables user-defined regional reports (tax schedules, cashflow runway forecasts) without modifying core Rust code.

```rust
// Rust Host ABI Contract for Financial Reports
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
    pub account_type: String,     // "ASSET", "LIABILITY", "EQUITY", "INCOME", "EXPENSE"
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
    pub tone: String,             // "neutral", "ok", "err", "warn", "teal"
    pub trend_basis_points: Option<i64>, // Basis points integer (e.g., +1250 bps = +12.50%). Resolves Zero-Float ABI contradiction.
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
    pub data_points: Vec<(String, i64)>, // (Label, Minor units)
}
```

#### Extension Point C: External Sync Adapters (`SyncAdapter`)
Enables bidirectional or export-only synchronization to cloud/remote targets (WebDAV, Nextcloud, S3, Beancount plain-text git repo) without exposing raw database handles.

```rust
// Rust Host ABI Contract for Sync Adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncAdapterInput {
    pub adapter_id: String,
    pub vault_name: String,
    pub payload_format: String,   // "ENCRYPTED_ZIP_AGE" or "BEANCOUNT_PLAIN"
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

### 3.5 Plugin Manifest Specification (`plugin.toml`)

```toml
manifest_version = 1

[plugin]
id = "com.finnca.parser.bca"
name = "BCA KlikBCA & e-Statement Parser"
version = "1.2.0"
author = "Finnca Community <community@finnca.org>"
description = "High-precision parser for Bank Central Asia CSV and PDF bank statements."
license = "MIT"
min_finnca_version = "0.2.0"
target_abi_version = "1.0"

[binary]
entrypoint = "dist/plugin.wasm"
sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
signature = "untrusted comment: minisign public signature\nRWR12948...base64..."

[limits]
max_memory_mb = 32
timeout_ms = 5000
fuel_budget = 10000000

[permissions]
filesystem = ["scoped_read"]
network = []
ledger = "none"
notifications = true

[extensions]
statement_parsers = [
  { id = "bca_csv", name = "BCA KlikBCA CSV", file_extensions = ["csv"], mime_types = ["text/csv"] },
  { id = "bca_pdf", name = "BCA e-Statement PDF", file_extensions = ["pdf"], mime_types = ["application/pdf"] }
]
```

---

## 4. R3: Update Lifecycle & Distribution Resilience

### 4.1 In-App Minisign Updater Audit & CI/CD Pipeline Gaps

#### In-App Update Engine
Finnca integrates `tauri-plugin-updater` with Minisign Ed25519 public key verification:
- `src-tauri/tauri.conf.json:65-76`: Enforces signature checks against embedded public key `dW50cnVzdGVkIGNvbW1lbnQ...`.
- `src/lib/core/updater/updateChecker.ts` and `updaterState.svelte.ts`: Drives checking, downloading, and installing via typed IPC.
- **Security Assessment**: Tauri v2 discards downloaded packages if signature verification fails, providing strong protection against man-in-the-middle tampering of GitHub Releases.

#### CI/CD Signing Fallback Risk (`.github/workflows/release.yml`)
Lines 89–98 in `.github/workflows/release.yml` reveal a serious security loophole:
```javascript
const signingKey = process.env.SIGNING_KEY || '';
if (!signingKey.trim()) {
  console.log('NOTE: TAURI_SIGNING_PRIVATE_KEY is not configured in GitHub Secrets. Disabling updater artifact generation to allow package builds.');
  tauri.bundle.createUpdaterArtifacts = false;
  if (tauri.plugins && tauri.plugins.updater) {
    delete tauri.plugins.updater.pubkey;
  }
}
```
If `TAURI_SIGNING_PRIVATE_KEY` is accidentally removed or misconfigured in GitHub Secrets, the CI workflow strips `pubkey` and emits unsigned packages without failing the build. The pipeline must be modified to fail with a hard exit code if signing keys are missing.

---

### 4.2 Distribution Script Vulnerabilities (`finnca.sh`, `finnca.ps1`)

While in-app updates verify cryptographic signatures, the shell and PowerShell installer scripts completely bypass verification:
- `scripts/finnca.sh`: Lines 331, 355, and 380 download `.deb`, `.rpm`, and `.AppImage` packages using `curl` and install them directly without computing SHA-256 hashes or verifying Minisign signatures. Line 406 downloads the update engine script itself via raw `curl | bash`.
- `scripts/finnca.ps1`: Line 119 uses `Invoke-WebRequest` to download `_x64-setup.exe` and line 136 immediately executes `Start-Process $TempFile /S` without Authenticode or checksum verification.

#### Remediation Blueprint for Distribution Scripts
Update `scripts/finnca.sh` and `scripts/finnca.ps1` to require **mandatory Minisign Ed25519 digital signature verification** on the checksum manifest before computing hashes or running packages. Falling back to unauthenticated `SHA256SUMS` over raw HTTP leaves users vulnerable to network MITM attacks where an attacker simultaneously replaces both the binary and the checksum file:

```bash
# Hardened snippet for scripts/finnca.sh
MINISIGN_PUBKEY="RWQn1xW5rV3k4i7L9p8s..." # Embedded Finnca Release Public Key

echo "==> Downloading checksum manifest and cryptographic signature..."
curl -fsSL "${release_url}/SHA256SUMS" -o "${temp_dir}/SHA256SUMS"
curl -fsSL "${release_url}/SHA256SUMS.minisig" -o "${temp_dir}/SHA256SUMS.minisig"

echo "==> Verifying Minisign Ed25519 signature..."
cd "${temp_dir}"
if command -v minisign >/dev/null 2>&1; then
    minisign -V -p "${MINISIGN_PUBKEY}" -m "SHA256SUMS" || {
        echo "ERROR: Minisign signature verification failed! Package manifest is untrusted or tampered." >&2
        exit 1
    }
elif openssl pkeyutl -help 2>&1 | grep -q "rawin"; then
    # Fallback to OpenSSL 3.x Ed25519 raw verification if standalone minisign is absent
    openssl pkeyutl -verify -rawin -pubin -inkey <(echo "${MINISIGN_PUBKEY_PEM}") \
        -sigfile "SHA256SUMS.minisig" -in "SHA256SUMS" || {
        echo "ERROR: OpenSSL Ed25519 signature verification failed! Manifest is untrusted." >&2
        exit 1
    }
else
    echo "ERROR: No cryptographic verification tool (minisign or openssl with ed25519) found on host!" >&2
    echo "Refusing to install unverified packages. Please install 'minisign' or upgrade OpenSSL." >&2
    exit 1
fi

echo "==> Verifying package SHA256 integrity..."
sha256sum --check --ignore-missing SHA256SUMS || {
    echo "ERROR: Cryptographic checksum verification failed! Download corrupted or tampered." >&2
    exit 1
}
```

---

### 4.3 Startup Crash Watchdog Architecture & Rollback Safeguards

Currently, neither Tauri nor the OS scripts implement startup crash recovery. If a new release panics during initialization (e.g., bad migration, corrupted state, ABI incompatibility), the application enters an unrecoverable crash loop.

```
+-----------------------------------------------------------------------------------+
| STARTUP CRASH WATCHDOG STATE MACHINE WITH ACTIVE-BINARY & CLEAN-EXIT GUARDS       |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  [In-App Update Applied]                                                          |
|            │                                                                      |
|            ├─► 1. Identify target binary path (Host AppImage via $APPIMAGE on     |
|            │      Linux, or current executable on Windows/Debian)                 |
|            ├─► 2. Create backup executable: <bin>.backup                          |
|            ├─► 3. Swap binary avoiding OS active locks:                           |
|            │      • Linux: unlink (remove_file) old bin before write, or rename   |
|            │      • Windows: rename current_exe -> current_exe.old before write   |
|            └─► 4. Write watchdog marker: update_pending.json                      |
|                    { target_version: "0.2.0", attempts: 0 }                       |
|                                                                                   |
|  [Application Launch]                                                             |
|            │                                                                      |
|            ▼                                                                      |
|  Does update_pending.json exist?                                                  |
|       ├──── NO ──► Normal Startup                                                 |
|       │                                                                           |
|       └──── YES ─► attempts = attempts + 1                                        |
|                     │                                                             |
|                     ├─► attempts >= 2?                                            |
|                     │     ├─► YES: CRASH LOOP DETECTED!                           |
|                     │     │        • Revert: swap <bin>.backup -> <bin> via       |
|                     │     │          unlink/rename or detached helper             |
|                     │     │        • Purge update_pending.json                    |
|                     │     │        • Show notification: "Update reverted"         |
|                     │     │        • Restart into previous working version        |
|                     │     │                                                       |
|                     │     └─► NO: Save attempts; Arm 15s timer & clean-exit hook  |
|                     │                                                             |
|                     ├───► [Clean User Exit Under 15s (Window Close / Quit)]       |
|                     │         • Disarm watchdog marker (NOT counted as crash)     |
|                     │         • Exit normally                                     |
|                     │                                                             |
|                     ▼                                                             |
|         [App Runs 15 Seconds Without Panic]                                       |
|                     │                                                             |
|                     ▼                                                             |
|         • Delete update_pending.json                                              |
|         • Delete <bin>.backup                                                     |
|         • Disarm Watchdog (Update Confirmed Healthy)                              |
|                                                                                   |
+-----------------------------------------------------------------------------------+
```

#### Resolution of Active Binary Replacement Hazards (`ETXTBSY`, Windows Locks, AppImage)
Directly invoking `std::fs::copy(&backup, &self.current_exe)` on a running binary fails across all supported desktop platforms:
1. **Linux `ETXTBSY` (Text File Busy)**:
   - In Linux kernels, attempting to open an active executable file for writing returns `ETXTBSY` (`[Errno 26] Text file busy`).
   - **Remediation**: The watchdog must unlink the file first via `std::fs::remove_file(&self.current_exe)` or atomic `rename(&self.current_exe, &format!("{}.old", self.current_exe.display()))`. In POSIX, unlinking an open file removes its directory entry while existing running processes continue executing until exit. A replacement binary can then be written to the original path without error.
2. **Windows File Locking (`ERROR_ACCESS_DENIED`)**:
   - The Windows kernel strictly locks executing binaries against deletion or overwriting.
   - **Remediation**: Windows permits *renaming* an open executable within the same filesystem volume. The updater renames `finnca.exe` to `finnca.exe.old`, writes the new `finnca.exe`, and schedules `.old` cleanup upon process termination via a lightweight detached helper (`finnca-rollback.exe`).
3. **Linux AppImage Squashfs Read-Only Mount (`EROFS`)**:
   - In AppImage deployments, `std::env::current_exe()` resolves to `/tmp/.mount_XXXX/usr/bin/finnca`, located on an immutable, read-only squashfs filesystem (`EROFS`).
   - **Remediation**: The watchdog and updater must inspect `std::env::var("APPIMAGE")`. If defined, the target for backup, swap, and rollback is the outer user-facing `.AppImage` file on the host filesystem (e.g. `~/Applications/Finnca.AppImage`), completely avoiding writes inside `/tmp/.mount_*`.

#### Watchdog Short Clean Session Handling
A major operational pitfall of naive startup watchdogs is misinterpreting brief user interactions as application crashes:
- If a user opens Finnca, checks a quick account balance for 5 seconds, and cleanly closes the window or presses `Ctrl+Q`, an unhandled session timer would treat the process termination as an abnormal exit. Two consecutive short sessions would falsely trigger an automated rollback!
- **Remediation**: The watchdog must hook Tauri's clean application lifecycle events (`RunEvent::Exit`, `WindowEvent::CloseRequested`, and POSIX `SIGTERM`). When a clean shutdown is detected, the watchdog immediately disarms or clears `update_pending.json` before process termination, ensuring that only genuine panics, fatal unhandled exceptions, or abnormal aborts trigger crash loop increments.

#### Pre-Main Crash Loop Protection (Dynamic Linker Failures)
Because the in-process watchdog runs inside Rust `main()`, it cannot intercept pre-`main` failures caused by missing shared libraries (e.g. `libwebkit2gtk-4.1.so.0`, glibc version mismatches) or corrupt ELF/PE headers. To mitigate pre-main crashes:
- `scripts/finnca.sh` and the `.desktop` launcher include a lightweight pre-flight probe (`ldd "$APPIMAGE" | grep "not found"` or launching with `--check-runtime`) before delegating to the primary GUI process. If pre-flight verification fails, the launcher automatically rolls back the binary before entering a dead-lock state.

---

### 4.4 Multi-Channel Release Pipeline & Offline `.finnca-pkg` Specification

#### Multi-Channel Release Architecture
Expand `tauri.conf.json` updater endpoints to support dynamic channel resolution in `src-tauri/src/updater/`:
- **Stable**: `https://github.com/endrico-fn/finnca/releases/latest/download/latest.json`
- **Beta**: `https://github.com/endrico-fn/finnca/releases/download/beta/latest-beta.json`
- **Nightly**: `https://github.com/endrico-fn/finnca/releases/download/nightly/latest-nightly.json`

Users select their update channel in Settings (`finnca.json`), which dynamically configures `@tauri-apps/plugin-updater`.

#### Offline / Airgapped Update Package (`.finnca-pkg`)
For airgapped and high-security workstations:
- **Format**: Standard zip archive containing:
  - `payload.tar.gz`: Platform-specific binary archive.
  - `manifest.json`: Version metadata, target OS, minimum schema version.
  - `signature.minisig`: Minisign signature of `payload.tar.gz`.
- **In-App Handling**: Users open Settings -> "Install Offline Update" -> select `.finnca-pkg`. The Rust command `import_offline_update` verifies the Minisign signature using the embedded public key, arms the crash watchdog, and replaces the executable.

---

## 5. R4: Desktop Maturity & Operational Reliability Assessment

### 5.1 Privacy-Preserving Local Crash Reporting & Zero-Telemetry Diagnostics

Finnca currently has zero crash reporting infrastructure: no Rust `panic::set_hook`, no logging crate, and no frontend `window.onerror` handler.

#### Production Crash Hook Architecture
```rust
// In src-tauri/src/main.rs
pub fn init_crash_watchdog() {
    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::capture();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // 1. Sanitize error message: scrub paths, monetary figures, account codes
        let sanitized_msg = sanitize_panic_message(panic_info);

        let report = serde_json::json!({
            "timestamp": timestamp,
            "version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "panic": sanitized_msg,
            "backtrace": format!("{backtrace}")
        });

        // 2. Persist to local diagnostic crash log (max 5 files, 0600 permissions)
        write_sanitized_crash_report(&report);
    }));
}

fn sanitize_panic_message(info: &std::panic::PanicHookInfo<'_>) -> String {
    let raw = format!("{info}");
    // Strip user home directories: /home/<user> or C:\Users\<user>
    let re_path = regex::Regex::new(r"(/home/[^/\s]+|C:\\Users\\[^\\]+)").unwrap();
    let scrubbed = re_path.replace_all(&raw, "[REDACTED_PATH]");

    // Strip currency and monetary figures
    let re_money = regex::Regex::new(r"\b\d+([.,]\d{2,})?\b").unwrap();
    re_money.replace_all(&scrubbed, "[NUMERIC_MASKED]").to_string()
}
```

---

### 5.2 Utilitarian-Brutalist UI Accessibility (a11y) & Keyboard Navigation Audit

Finnca features powerful utilitarian keyboard shortcuts (vim navigation in `JournalList.svelte`: `j`/`k`, `n`, `t`, `Space`, `Enter`). However, several components exhibit significant accessibility deficiencies:

| Component | Path | Current Defect | WAI-ARIA Remediation Blueprint |
|:---|:---|:---|:---|
| **`Tabs.svelte`** | `src/lib/components/ui/Tabs.svelte:24-85` | Omits `role="tablist"`; buttons use `aria-pressed` instead of `role="tab"`; missing arrow navigation. | Add `role="tablist"`, `role="tab"`, `aria-selected`, and attach `ArrowLeft`/`ArrowRight` key handlers. |
| **`SelectDropdown.svelte`** | `src/lib/components/ui/SelectDropdown.svelte:88-157` | Button uses `aria-haspopup="listbox"` but dropdown menu lacks `role="listbox"`, options lack `role="option"`. | Wrap items in `role="listbox"`, set `role="option"`, and support `ArrowUp`/`ArrowDown` item cycling. |
| **`CommandPalette.svelte`** | `src/lib/components/layout/CommandPalette.svelte:58-135` | Lacks `role="combobox"`; Tab focus escapes into background webview DOM. | Add `role="combobox"`, `role="listbox"`, and enforce focus trap using `handleDialogKey` pattern. |
| **`JournalTable.svelte`** | `src/lib/features/ledger/components/JournalTable.svelte:105` | Action header is an empty `<th>` element with no accessible label. | Add `<span class="sr-only">Actions</span>` to header. |
| **`DonutChart.svelte`** | `src/lib/components/charts/DonutChart.svelte:133` | Root `<svg>` element has no accessible role or title. | Add `role="img"` and `<title>Asset Allocation Breakdown</title>`. |

---

### 5.3 Background Lifecycle & OS Sleep Auto-Lock Daemon

1. **System Tray Integration**: `tauri.conf.json` currently omits `"trayIcon"`. A system tray icon with a minimalist menu (`Open Vault`, `Lock Vault`, `Quick Add Transaction`, `Exit`) should be introduced to support background operation and quick entry.
2. **OS Sleep / Suspend Auto-Lock Daemon**:
   - If a laptop is suspended while Finnca is running, decrypted database handles and SQLCipher keys remain in memory.
   - **Linux**: Hook D-Bus interface `org.freedesktop.login1.Manager` for signal `PrepareForSleep(bool)`.
   - **Windows**: Hook `WM_POWERBROADCAST` message with `PBT_APMSUSPEND`.
   - Upon receiving sleep notifications, the application must immediately invoke `app_state.clear_session()`, drop database locks, and transition the UI to the locked state.

---

### 5.4 Database Schema Longevity & Open-Source Interoperability

#### Transactional Schema Migrations (`REL-DB-01`)
`src-tauri/src/db/schema.rs:18-82` executes migrations sequentially, but fails to wrap each script and its corresponding `PRAGMA user_version = N;` update in an explicit database transaction. In SQLite, DDL statements (`CREATE TABLE`, `CREATE INDEX`, `ALTER TABLE`) are fully transactional; however, executing them non-transactionally creates a severe split-brain vulnerability:

- **Failure Scenario**: If power is cut or the application terminates after table creation DDL executes but before `conn.pragma_update(None, "user_version", N)` completes, the tables persist on disk while `user_version` remains at `N - 1`. On subsequent launch, `migrate_schema` observes `current_version < N` and attempts to rerun migration `N`, immediately crashing with `rusqlite::Error::SqliteFailure: table accounts already exists` and permanently bricking the vault.
- **Remediation**: Wrap each migration script and its `user_version` bump in `conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)`. This acquires an immediate `RESERVED` write lock in WAL mode, ensuring that both DDL execution and `PRAGMA user_version` updates commit atomically or roll back completely upon unexpected interruption:

```rust
// Proposed fix in src-tauri/src/db/schema.rs
pub fn apply_migration(conn: &mut Connection, version: u32, sql: &str) -> Result<(), AppError> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    tx.execute_batch(sql)?;
    tx.pragma_update(None, "user_version", version)?;
    tx.commit()?;
    Ok(())
}
```

#### Forward-Version Protection & Health Check
1. **Health Verification**: Execute `PRAGMA quick_check;` on every vault unlock.
2. **Forward-Version Guard**: Reject database files if `current_version > MAX_KNOWN_SCHEMA_VERSION` (preventing older software builds from silently corrupting databases upgraded by future versions).

#### Open-Source Interoperability Status
- **Beancount Plain Text**: Fully implemented via `src-tauri/src/ledger/beancount_export.rs`.
- **CSV**: Implemented for ledger exports and statement reconciliation.
- **OFX / QFX / QIF**: Not yet implemented in backend Rust (over-claimed in `04_FEATURE_REGISTRY.md`). Scheduled for Phase 2 via Wasm parsers.
- **GnuCash XML**: Scheduled for Phase 3 via dedicated export mapping module.

---

## 6. Automated Dependency Vulnerability Scan Report

### 6.1 Frontend Dependency Vulnerabilities (`pnpm audit`)

A full audit of the frontend dependency tree (`pnpm v10.6.1`, `Node.js v26.10.0`) identified **10 vulnerabilities** across two packages:

| Advisory ID | Severity | Affected Package | Vulnerable Range | Patched In | Vulnerability Summary | Dependency Chain |
|:---|:---:|:---|:---:|:---:|:---|:---|
| `GHSA-j22f-vq7h-c4qm` | **HIGH** | `devalue` | `>=5.1.0 <=5.9.2` | `>=5.9.3` | `stringify`/`uneval` serialize shared memory | Transitive via `svelte@5.56.9`, `@sveltejs/kit@2.70.2` |
| `GHSA-mcm9-63f2-9j32` | **HIGH** | `devalue` | `<=5.9.2` | `>=5.9.3` | Repeated primitive strings cause quadratic expansion in `uneval` | Transitive via `svelte@5.56.9` |
| `GHSA-x5rw-q4pp-hg5g` | **HIGH** | `devalue` | `>=5.8.0 <=5.9.2` | `>=5.9.3` | `stringifyAsync` causes unhandled rejection despite caught promise | Transitive via `svelte@5.56.9` |
| `GHSA-qhr7-859c-m2p7` | **HIGH** | `brace-expansion` | `>=4.0.0 <5.0.11` | `>=5.0.11` | DoS via uncontrolled recursion on nested brace groups | Transitive via `minimatch` in ESLint plugins |
| `GHSA-6j4f-fj2g-mc7p` | **HIGH** | `brace-expansion` | `>=4.0.0 <5.0.10` | `>=5.0.10` | DoS via uncontrolled recursion in `parseCommaParts` | Transitive via `minimatch` in ESLint plugins |
| `GHSA-9rgm-9g3h-6x36` | **MODERATE**| `devalue` | `<5.9.1` | `>=5.9.2` | Denial of service via malformed input serialization | Transitive via `svelte@5.56.9` |
| `GHSA-hx4r-w6wj-j8fg` | **MODERATE**| `devalue` | `<=5.9.2` | `>=5.9.3` | Residual sparse-array CPU amplification in `uneval` | Transitive via `svelte@5.56.9` |
| `GHSA-4q55-j62x-fr9h` | **MODERATE**| `devalue` | `<=5.9.2` | `>=5.9.3` | Null-prototype object keys bypass `__proto__` rejection | Transitive via `svelte@5.56.9` |
| `GHSA-q2hr-2g5m-vwhr` | **MODERATE**| `brace-expansion` | `>=4.0.0 <5.0.12` | `>=5.0.12` | Quadratic-time expansion of `{a},b}` rewrite causes CPU DoS | Transitive via `minimatch` in ESLint |
| `GHSA-wf3x-273g-mvxv` | **LOW** | `devalue` | `>=1.0.0 <=5.9.2` | `>=5.9.3` | Sparse arrays emitted by `uneval` cause eager allocation | Transitive via `svelte@5.56.9` |

#### Remediation Plan for Frontend
Add resolution overrides to `package.json`:
```json
"pnpm": {
  "overrides": {
    "devalue": ">=5.9.3",
    "brace-expansion": ">=5.0.12"
  }
}
```
Run `pnpm update` to regenerate `pnpm-lock.yaml`.

---

### 6.2 Backend Dependency Vulnerabilities (`Cargo.lock` / RustSec OSV)

Scanning 635 packages in `src-tauri/Cargo.lock` against the Google OSV and RustSec databases identified 4 items:

| Advisory ID | Severity | Crate | Version | Fixed In | Vulnerability Summary & Impact | Dependency Chain |
|:---|:---:|:---|:---:|:---:|:---|:---|
| `RUSTSEC-2024-0429` | **MODERATE** | `glib` | `0.18.5` | `>=0.20.0` | **Unsoundness in `VariantStrIter`**: Passes immutable reference `&p` to a NULL `*mut libc::c_char`, triggering undefined behavior. | `finnca` -> `tauri 2.12.0` -> `tao 0.37.1` -> `gtk 0.18.2` -> `glib 0.18.5` |
| `RUSTSEC-2024-0436` | **INFO** | `paste` | `1.0.15` | N/A | Crate is officially unmaintained and archived by its author. | `finnca` -> `specta 2.0.0-rc.25` -> `paste 1.0.15` |
| `RUSTSEC-2024-0370` | **INFO** | `proc-macro-error` | `1.0.4` | N/A | Unmaintained crate with no releases in 4+ years; pulls redundant `syn 1.x`. | `finnca` -> `tauri` -> `gtk` -> `glib-macros` -> `proc-macro-error` |

#### Remediation Plan for Backend
- `glib 0.18.5`: Cannot be updated independently because it is pinned by Tauri's Linux windowing backend (`tao v0.37.1`). Track Tauri upstream release notes and upgrade Tauri when `tao` updates to GTK/Glib `0.20+`.
- `paste` & `proc-macro-error`: Harmless build-time procedural macro crates. Specta is actively migrating away from `paste` in upcoming Specta v2 stable.

---

## 7. Objective Industry Benchmarking Matrix

Finnca was evaluated against four leading tier-1 desktop applications across five architectural dimensions:

```
+=======================================================================================================================+
| OBJECTIVE INDUSTRY BENCHMARKING MATRIX                                                                                |
+=======================================================================================================================+
| Dimension              | FINNCA            | OBSIDIAN          | BITWARDEN         | VS CODE           | GNUCASH          |
+------------------------+-------------------+-------------------+-------------------+-------------------+------------------+
| 1. Sandboxing &        | HIGH              | LOW               | VERY HIGH         | HIGH              | NONE             |
|    Process Isolation   | Tauri v2 Webview  | Electron          | Hardened Electron | Multi-proc Electron| Monolithic C/C++ |
|                        | Revoked fs:default| Node integration  | Context isolation | Out-of-proc host  | Direct OS access |
+------------------------+-------------------+-------------------+-------------------+-------------------+------------------+
| 2. Memory Safety &     | HIGH              | MEDIUM            | VERY HIGH         | MEDIUM            | LOW              |
|    Key Hygiene         | 100% Safe Rust    | V8 GC Heap        | Rust CLI Core     | V8 GC Heap        | C/C++ memory risk|
|                        | Envelope Crypto   | No key zeroize    | Active zeroize    | Standard GC       | GMP zero-float   |
+------------------------+-------------------+-------------------+-------------------+-------------------+------------------+
| 3. Plugin Safety &     | PLANNED HIGH      | CRITICAL RISK     | N/A (CLOSED)      | HIGH              | LOW              |
|    Extensibility       | Extism Wasm       | Raw Node.js/JS    | Monolithic core   | Out-of-process    | Guile Scheme /   |
|                        | Linear memory     | Full OS rights    | No plugins        | Sandboxed workers | Native .so/.dll  |
+------------------------+-------------------+-------------------+-------------------+-------------------+------------------+
| 4. Update Resilience & | MEDIUM-HIGH       | HIGH              | VERY HIGH         | VERY HIGH         | LOW              |
|    Integrity           | Minisign Ed25519  | Code-signed       | Code-signed       | Background service| OS Package Mgr   |
|                        | Semver guard      | Dual-channel      | Server-enforced   | Dual-channel / Roll| Delegated       |
+------------------------+-------------------+-------------------+-------------------+-------------------+------------------+
| 5. Platform            | MEDIUM            | MEDIUM-HIGH       | VERY HIGH         | HIGH              | MEDIUM           |
|    Integration         | Single instance   | Deep-linking      | Biometrics / DPAPI| SecretStorage     | Native GTK       |
|                        | File associations | System tray       | System tray       | Terminal integration| Direct print   |
+=======================================================================================================================+
```

### Detailed Benchmarking Analysis

#### 1. Sandboxing & Process Isolation
- **Bitwarden (Rank 1)**: Outstanding renderer isolation. Completely disables Node.js integration, utilizes strict context bridges, and enforces zero-knowledge memory boundaries.
- **Finnca (Rank 2)**: Strong baseline with Tauri v2. Revokes `fs:default` and confines capabilities. Held back from Rank 1 by the `SEC-01` path validation gap.
- **VS Code (Rank 3)**: Isolates untrusted extensions in a separate Extension Host process; enforces Workspace Trust boundaries.
- **Obsidian (Rank 4)**: Renderer exposes native Node.js APIs to community plugins, leaving local files vulnerable.
- **GnuCash (Rank 5)**: Monolithic C/C++ binary with zero internal sandboxing.

#### 2. Memory Safety & Key Hygiene
- **Bitwarden & Finnca (Top Tier)**: Both leverage Rust for core cryptographic operations. Finnca employs Argon2id + ChaCha20-Poly1305 + SQLCipher. Bitwarden leads slightly due to integration with OS memory-locking primitives (`mlock`/DPAPI).
- **GnuCash**: Uses GMP rational numbers for exact zero-float calculations, but legacy C/C++ memory management exposes buffer overflow risks.

#### 3. Plugin Safety & Extensibility
- **Finnca (Extism Blueprint)**: Extism Wasm provides linear memory isolation, instruction metering, and an invariant gateway that mathematically guarantees third-party code cannot contaminate double-entry balance.
- **Obsidian**: Plugins execute raw Node.js code with unrestricted filesystem and network access, presenting significant supply-chain risks.
- **Bitwarden**: Closed architecture. Deliberately rejects plugins to safeguard credential vaults.

#### 4. Update Resilience & Integrity
- **VS Code & Bitwarden**: Gold standard. Background update services with cryptographic code signing, dual channels (Stable/Insiders), atomic binary swaps, and rollback safety.
- **Finnca**: Excellent cryptographic foundation with in-app Minisign verification. Needs distribution script hardening (`SEC-UP-01`) and a startup crash watchdog (`REL-UP-02`) to match tier-1 resilience.

#### 5. Platform Integration
- **Bitwarden**: Seamless desktop integration with Touch ID, Windows Hello, Linux FIDO2, and OS Keychain services.
- **Finnca**: Solid window state persistence and single-instance locks, but lacks system tray support and OS sleep auto-lock.

---

## 8. Prioritized, Phased Implementation Roadmap

```
PHASED IMPLEMENTATION TIMELINE
┌───────────────────────────────┬───────────────────────────────┬───────────────────────────────┐
│ PHASE 1: Quick Wins & Hardening│ PHASE 2: Core Extensions       │ PHASE 3: Plugins & Automation │
│ (0 – 30 Days)                 │ (30 – 90 Days)                │ (90 – 180 Days)               │
├───────────────────────────────┼───────────────────────────────┼───────────────────────────────┤
│ • Patch SEC-01 Path Traversal │ • OS Keychain Integration     │ • Extism Wasm Plugin Engine   │
│ • Add SQLCipher temp_store    │ • OS Sleep Auto-Lock Daemon   │ • Statement Parser PDKs       │
│ • Configure [profile.release] │ • Local Crash Reporting Hook  │ • Offline .finnca-pkg Support │
│ • Airgap Webview CSP          │ • Utilitarian A11y Upgrades   │ • GnuCash / OFX Interop       │
│ • Transactional DB Migrations │ • Multi-Channel Release Config│ • Mandatory CI/CD Signing Gate│
│ • Harden Shell Install Scripts│ • Startup Crash Watchdog      │ • Community Plugin Registry   │
│ • Resolve 10 NPM Advisories   │ • System Tray & Minimize      │                               │
└───────────────────────────────┴───────────────────────────────┴───────────────────────────────┘
```

---

### Phase 1: Quick Wins / Immediate Hardening (0–30 Days)
*Focus: Eliminate critical security exposures, harden release binaries, and secure database transactions.*

1. **Patch Arbitrary File Read (`SEC-01`)**:
   - Refactor `src-tauri/src/reconcile/commands.rs` to validate absolute paths, check symlink metadata on the raw uncanonicalized path before resolving, verify canonical targets, enforce cross-platform system directory blocks (Linux `/etc`, `/var`, `/proc`; Windows `C:\Windows`, `C:\Program Files`, `C:\ProgramData`), isolate sensitive credential roots (`~/.ssh`, `~/.gnupg`, `~/.aws`, `~/.config`) while allowing legitimate user vaults, and strictly whitelist statement formats (`.csv`, `.ofx`, `.qfx`, `.mt940`, `.sta`), removing broad `.txt`.
2. **Harden SQLCipher Storage (`SEC-02`)**:
   - Add `PRAGMA temp_store = MEMORY;` in `src-tauri/src/db/mod.rs` to ensure all temporary query b-trees and sorting indices remain strictly in volatile memory. Remove non-existent phantom pragma `PRAGMA cipher_default_temp_store`.
3. **Configure Cargo Release Profile (`SEC-04`)**:
   - Add `[profile.release]` to `src-tauri/Cargo.toml` (`opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`).
4. **Airgap CSP & Relocate FX Sync (`SEC-05`)**:
   - Strip `https://open.er-api.com` from `tauri.conf.json`. Move FX synchronization into a dedicated Rust IPC command.
5. **Transactional Database Migrations (`REL-DB-01`)**:
   - Wrap migration execution and `user_version` updates in atomic `rusqlite` transactions (`BEGIN IMMEDIATE ... COMMIT`) in `src-tauri/src/db/schema.rs` to prevent partial DDL application and split-brain corruption.
6. **Harden Distribution Installer Scripts (`SEC-UP-01`)**:
   - Update `scripts/finnca.sh` and `scripts/finnca.ps1` to require mandatory Minisign Ed25519 signature verification (`SHA256SUMS.minisig`) before computing SHA-256 hashes or installing packages, eliminating unauthenticated HTTP fallback.
7. **Resolve Frontend Vulnerabilities (`SEC-12`)**:
   - Add package overrides in `package.json` for `devalue >= 5.9.3` and `brace-expansion >= 5.0.12`. Run `pnpm update`.

---

### Phase 2: Core Platform Extensions & Reliability (30–90 Days)
*Focus: Operating system integration, automated crash diagnostics, accessibility, and update safety.*

1. **OS Keychain & Secret Service Integration (`SEC-08` / ADR 0008)**:
   - Integrate `keyring = "3"` in Rust backend with complete sensitive heap zeroization (`Zeroizing<String>`).
   - Implement Dual Envelope Architecture supporting biometric unlock via Apple Touch ID, Windows Hello, and Linux Secret Service, keyed by immutable `vault_id`.
2. **OS Sleep / Suspend Auto-Lock Daemon**:
   - Listen to Linux D-Bus `org.freedesktop.login1.Manager` and Windows `WM_POWERBROADCAST` to lock active vaults automatically when systems suspend.
3. **Privacy-Preserving Local Crash Reporting**:
   - Implement Rust `panic::set_hook` with PII/financial data scrubbing. Write sanitized JSON reports to `~/.local/share/finnca/crashes/`.
4. **Utilitarian-Brutalist Accessibility Upgrades**:
   - Upgrade `Tabs.svelte` (`role="tablist"`), `SelectDropdown.svelte` (`role="listbox"`), and `CommandPalette.svelte` (`role="combobox"` with focus trap).
5. **Startup Crash Watchdog & Multi-Channel Updater (ADR 0009)**:
   - Implement stateful crash counter (`update_pending.json`), executable backup, and automated rollback upon startup failure.
   - Overcome OS binary locking: unlink-before-write or atomic rename on Linux, `.old` rename on Windows, and outer `$APPIMAGE` path targeting on AppImage.
   - Hook clean exit lifecycle events to disarm watchdog on brief clean user sessions (< 15 seconds).
   - Support Stable, Beta, and Nightly release channels in settings.
6. **System Tray Integration**:
   - Add tray icon with quick actions and minimize-to-tray lifecycle handling.

---

### Phase 3: Plugin Ecosystem & Release Automation (90–180 Days)
*Focus: Polyglot Wasm plugin runtime, community parser ecosystem, and enterprise airgapped updates.*

1. **Extism WebAssembly Plugin Engine (ADR 0007)**:
   - Integrate `extism` crate in `src-tauri/src/plugins/`.
   - Implement capability gate, 32 MB linear memory caps, and instruction fuel budgeting.
2. **Zero-Float & Debit-First Gateway**:
   - Enforce unconditional IEEE-754 float rejection (`val.is_f64()` rejected unconditionally, no exceptions for whole floats) and string-disguised float detection (`"15000.50"`, `"NaN"`).
   - Enforce integer basis points in ABI structs (`ReportCard.trend_basis_points`).
   - Enforce host-level Debit-First validation (`validate_plugin_draft_entry`) on 2-leg draft postings (`postings[0] >= 0`, `postings[1] <= 0`).
3. **Statement Parser & Custom Report PDKs**:
   - Publish TypeScript, Go, and Rust SDKs for community plugin development.
   - Ship official plugins for major regional banks (BCA, Mandiri, Chase, Revolut).
4. **Offline / Airgapped Update Engine (`.finnca-pkg`)**:
   - Implement `.finnca-pkg` extraction, Minisign signature verification, and in-app manual update installer.
5. **Mandatory CI/CD Signing Gate**:
   - Update `.github/workflows/release.yml` to fail with a hard exit code if `TAURI_SIGNING_PRIVATE_KEY` is missing.
6. **Financial Data Interoperability**:
   - Build export mapping modules for GnuCash XML and import modules for OFX/QFX statements.

---

### Finnca System Axioms Adherence Checklist

All recommendations in this audit strictly uphold Finnca's foundational system axioms:

- [x] **Zero-Float Arithmetic Invariant**: All amounts are strictly modeled as signed integer minor units (`i64`/`i128`). Floating-point types are rejected at the plugin gateway.
- [x] **Debit-First Invariant**: All transaction drafting enforces Index 0 = Debit (`amount >= 0`) and Index 1 = Credit (`amount <= 0`).
- [x] **Utilitarian-Brutalist Aesthetic**: Zero CSS drift. All UI improvements adhere to semantic design tokens (`bg-bg-app`, `bg-bg-card`, `border-line`, `text-teal`) and font hierarchy (`Proto Mono` for metrics/headings, `Aux Mono` for text/inputs).
- [x] **Svelte 5 Runes Strictness**: Component upgrades utilize standard runes (`$state`, `$derived`, `$props`) strictly in `.svelte` and `.svelte.ts` files.
- [x] **Envelope Encryption & Local Vault Privacy**: Vault data remains completely offline in local SQLite databases protected by Argon2id, ChaCha20-Poly1305, and SQLCipher.

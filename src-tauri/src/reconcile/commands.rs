use super::dto::{
    CreateReconcileRuleInput, MatchStatementOutput, PostingReconcileView, ReconcileRuleWithAccount,
    ReconciliationStatusView, StatementRow,
};
use super::{matcher, rules};
use crate::audit::{self, models::AuditAction};
use crate::shared::AppError;
use crate::state::AppState;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, State};

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn get_actor(app: &AppHandle) -> String {
    crate::config::load(app)
        .ok()
        .and_then(|c| c.username)
        .unwrap_or_else(|| "user".to_string())
}

pub fn get_account_reconciliation_status(
    conn: &Connection,
    account_id: &str,
) -> Result<ReconciliationStatusView, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.entry_id, je.date, je.description, p.amount, p.memo, p.reconciled, p.reconciled_at
         FROM postings p
         JOIN journal_entries je ON je.id = p.entry_id
         WHERE p.account_id = ?1
         ORDER BY je.date ASC, je.id ASC;",
    )?;

    let rows = stmt.query_map(params![account_id], |row| {
        Ok(PostingReconcileView {
            posting_id: row.get(0)?,
            entry_id: row.get(1)?,
            date: row.get(2)?,
            description: row.get(3)?,
            amount: row.get(4)?,
            memo: row.get(5)?,
            reconciled: row.get(6)?,
            reconciled_at: row.get(7)?,
        })
    })?;

    let mut reconciled_balance: i64 = 0;
    let mut cleared_balance: i64 = 0;
    let mut uncleared_balance: i64 = 0;
    let mut total_balance: i64 = 0;
    let mut uncleared_postings = Vec::new();

    for r in rows {
        let p = r?;
        total_balance += p.amount;

        match p.reconciled.as_str() {
            "y" => {
                reconciled_balance += p.amount;
                cleared_balance += p.amount;
            }
            "c" => {
                cleared_balance += p.amount;
                uncleared_balance += p.amount;
                uncleared_postings.push(p);
            }
            _ => {
                uncleared_balance += p.amount;
                uncleared_postings.push(p);
            }
        }
    }

    Ok(ReconciliationStatusView {
        account_id: account_id.to_string(),
        reconciled_balance,
        cleared_balance,
        uncleared_balance,
        total_balance,
        uncleared_postings,
    })
}

#[tauri::command]
#[specta::specta]
pub fn get_reconciliation_status_cmd(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<ReconciliationStatusView, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    get_account_reconciliation_status(&conn, &account_id)
}

#[tauri::command]
#[specta::specta]
pub fn set_posting_reconciled_cmd(
    state: State<'_, AppState>,
    posting_id: String,
    status: String,
) -> Result<(), AppError> {
    if !matches!(status.as_str(), "n" | "c" | "y") {
        return Err(AppError::InvalidInput(
            "reconciled status must be 'n', 'c', or 'y'".into(),
        ));
    }

    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;

    let reconciled_at = if status == "y" {
        Some(now_unix())
    } else {
        None
    };

    let affected = conn.execute(
        "UPDATE postings SET reconciled = ?2, reconciled_at = ?3 WHERE id = ?1;",
        params![posting_id, status, reconciled_at],
    )?;

    if affected == 0 {
        return Err(AppError::NotFound(format!("posting id={posting_id}")));
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn bulk_set_postings_reconciled_cmd(
    state: State<'_, AppState>,
    posting_ids: Vec<String>,
    status: String,
) -> Result<(), AppError> {
    if !matches!(status.as_str(), "n" | "c" | "y") {
        return Err(AppError::InvalidInput(
            "reconciled status must be 'n', 'c', or 'y'".into(),
        ));
    }

    let db = state.get_db()?;
    let mut conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    let tx = conn.transaction()?;

    let reconciled_at = if status == "y" {
        Some(now_unix())
    } else {
        None
    };

    {
        let mut stmt =
            tx.prepare("UPDATE postings SET reconciled = ?2, reconciled_at = ?3 WHERE id = ?1;")?;
        for id in posting_ids {
            stmt.execute(params![id, status, reconciled_at])?;
        }
    }

    tx.commit()?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn finish_reconciliation_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    account_id: String,
    posting_ids: Vec<String>,
) -> Result<(), AppError> {
    let actor = get_actor(&app);
    let db = state.get_db()?;
    let mut conn = db.lock().map_err(|_| AppError::VaultLocked)?;

    let now = now_unix();
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "UPDATE postings SET reconciled = 'y', reconciled_at = ?3
             WHERE id = ?1 AND account_id = ?2;",
        )?;

        for id in &posting_ids {
            stmt.execute(params![id, account_id, now])?;
        }
    }

    let _ = audit::record(
        &tx,
        &actor,
        AuditAction::ReconcileAccount,
        "ACCOUNT",
        &account_id,
        Some(&format!(
            "Reconciled {} postings for account {}",
            posting_ids.len(),
            account_id
        )),
    );

    tx.commit()?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn match_statement_cmd(
    state: State<'_, AppState>,
    account_id: String,
    statements: Vec<StatementRow>,
    auto_clear: bool,
) -> Result<MatchStatementOutput, AppError> {
    let db = state.get_db()?;
    let mut conn = db.lock().map_err(|_| AppError::VaultLocked)?;

    let status = get_account_reconciliation_status(&conn, &account_id)?;
    let mut result =
        matcher::match_statements_against_postings(&statements, &status.uncleared_postings, 4);

    let active_rules = rules::list_rules(&conn)?;
    rules::apply_rules_to_unmatched_statements(&mut result.unmatched_statement_rows, &active_rules);

    if auto_clear && !result.matches.is_empty() {
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "UPDATE postings SET reconciled = 'c' WHERE id = ?1 AND reconciled = 'n';",
            )?;
            for m in &result.matches {
                stmt.execute(params![m.posting_id])?;
            }
        }
        tx.commit()?;
    }

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub fn list_reconcile_rules_cmd(
    state: State<'_, AppState>,
) -> Result<Vec<ReconcileRuleWithAccount>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    rules::list_rules(&conn)
}

#[tauri::command]
#[specta::specta]
pub fn create_reconcile_rule_cmd(
    state: State<'_, AppState>,
    input: CreateReconcileRuleInput,
) -> Result<ReconcileRuleWithAccount, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    rules::create_rule(&conn, &input)
}

#[tauri::command]
#[specta::specta]
pub fn delete_reconcile_rule_cmd(
    state: State<'_, AppState>,
    rule_id: String,
) -> Result<(), AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    rules::delete_rule(&conn, &rule_id)
}

#[tauri::command]
#[specta::specta]
pub fn evaluate_reconcile_rules_cmd(
    state: State<'_, AppState>,
    mut statements: Vec<StatementRow>,
) -> Result<Vec<StatementRow>, AppError> {
    let db = state.get_db()?;
    let conn = db.lock().map_err(|_| AppError::VaultLocked)?;
    let rules = rules::list_rules(&conn)?;
    rules::apply_rules_to_unmatched_statements(&mut statements, &rules);
    Ok(statements)
}

pub const MAX_STATEMENT_FILE_BYTES: u64 = 5 * 1024 * 1024;
pub const ALLOWED_STATEMENT_EXTENSIONS: &[&str] = &["csv", "ofx", "qif", "txt", "beancount"];

pub(crate) fn validate_statement_path(raw_path: &str) -> Result<PathBuf, AppError> {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput(
            "Statement file path cannot be empty".into(),
        ));
    }

    let p = Path::new(trimmed);
    if !p.is_absolute() {
        return Err(AppError::InvalidInput(
            "Statement file path must be absolute".into(),
        ));
    }

    if p.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(AppError::InvalidInput(
            "Path traversal components ('..') are strictly forbidden".into(),
        ));
    }

    // 1. Pre-canonicalization symlink check on raw uncanonicalized input path
    let input_meta = std::fs::symlink_metadata(p)
        .map_err(|e| AppError::NotFound(format!("Statement file at '{trimmed}': {e}")))?;
    if input_meta.file_type().is_symlink() {
        return Err(AppError::InvalidInput(
            "Reading symlinked statement files is strictly forbidden".into(),
        ));
    }
    if !input_meta.is_file() {
        return Err(AppError::NotFound(format!(
            "Statement file at '{trimmed}' is not a regular file"
        )));
    }

    // 2. Canonicalize path to resolve relative components ('..') and obtain normalized path
    let canonical = p.canonicalize().map_err(|e| {
        AppError::NotFound(format!("Failed to canonicalize statement file path: {e}"))
    })?;
    let canonical_str = canonical.to_string_lossy();

    // 3. Post-canonicalization symlink check (defense-in-depth against TOCTOU race)
    let meta = std::fs::symlink_metadata(&canonical)?;
    if meta.file_type().is_symlink() {
        return Err(AppError::InvalidInput(
            "Reading symlinked statement files is strictly forbidden".into(),
        ));
    }
    if !meta.is_file() {
        return Err(AppError::NotFound(
            "Statement target is not a regular file".into(),
        ));
    }

    // 4. Cross-platform system and sensitive directory blocks
    #[cfg(unix)]
    {
        let forbidden_system = [
            "/etc", "/root", "/boot", "/sys", "/proc", "/bin", "/sbin", "/usr", "/dev", "/var",
        ];
        if canonical_str == "/"
            || forbidden_system
                .iter()
                .any(|f| canonical_str == *f || canonical_str.starts_with(&format!("{f}/")))
        {
            return Err(AppError::InvalidInput(
                "Access to system directory is strictly prohibited".into(),
            ));
        }

        if let Some(home) = std::env::var_os("HOME") {
            let home_path = PathBuf::from(home);
            let forbidden_home = [
                home_path.join(".ssh"),
                home_path.join(".gnupg"),
                home_path.join(".aws"),
                home_path.join(".azure"),
                home_path.join(".kube"),
                home_path.join(".config").join("finnca"),
            ];
            if forbidden_home.iter().any(|f| canonical.starts_with(f)) {
                return Err(AppError::InvalidInput(
                    "Access to sensitive user configuration/credential directory is prohibited"
                        .into(),
                ));
            }
        }
    }

    #[cfg(windows)]
    {
        let canonical_str_norm = canonical_str.strip_prefix(r"\\?\").unwrap_or(&canonical_str);
        let canonical_upper = canonical_str_norm.to_uppercase();
        let forbidden_windows = [
            "\\WINDOWS",
            "\\PROGRAM FILES",
            "\\PROGRAM FILES (X86)",
            "\\PROGRAMDATA",
        ];
        if forbidden_windows
            .iter()
            .any(|w| canonical_upper.contains(w))
        {
            return Err(AppError::InvalidInput(
                "Access to Windows system directory is strictly prohibited".into(),
            ));
        }

        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let prof = user_profile.to_string_lossy();
            let prof_norm = prof.strip_prefix(r"\\?\").unwrap_or(&prof);
            let prof_upper = prof_norm.to_uppercase();
            let forbidden_win = [
                format!("{prof_upper}\\.SSH"),
                format!("{prof_upper}\\.AWS"),
                format!("{prof_upper}\\APPDATA\\LOCAL\\MICROSOFT\\CREDENTIALS"),
                format!("{prof_upper}\\APPDATA\\ROAMING\\MICROSOFT\\PROTECT"),
                format!("{prof_upper}\\APPDATA\\ROAMING\\FINNCA"),
            ];
            if forbidden_win
                .iter()
                .any(|f| canonical_upper == *f || canonical_upper.starts_with(&format!("{f}\\")))
            {
                return Err(AppError::InvalidInput(
                    "Access to sensitive user credential store is prohibited".into(),
                ));
            }
        }
    }

    // Block sensitive credential filenames/directories across any platform
    let sensitive_exact_names = [
        ".ssh",
        ".gnupg",
        ".aws",
        ".azure",
        ".kube",
        ".bash_history",
        ".zsh_history",
        ".env",
    ];
    let sensitive_name_prefixes = [
        "id_rsa",
        "id_ed25519",
        "id_ecdsa",
        "id_dsa",
        ".env",
        ".bash_history",
        ".zsh_history",
    ];

    for component in canonical.components() {
        let comp_str = component.as_os_str().to_string_lossy();
        if sensitive_exact_names
            .iter()
            .any(|s| comp_str.eq_ignore_ascii_case(s))
        {
            return Err(AppError::InvalidInput(
                "Access to sensitive configuration or credential directory is prohibited".into(),
            ));
        }
    }

    if let Some(file_name) = canonical.file_name().and_then(|s| s.to_str()) {
        let file_name_lower = file_name.to_lowercase();
        let file_stem_lower = canonical
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        if sensitive_exact_names.iter().any(|s| file_stem_lower == *s)
            || sensitive_name_prefixes
                .iter()
                .any(|p| file_name_lower.starts_with(p) || file_stem_lower.starts_with(p))
        {
            return Err(AppError::InvalidInput(
                "Access to sensitive configuration or credential file is prohibited".into(),
            ));
        }
    }

    // 5. Restrict allowed file extensions to statement files
    let ext = canonical
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !ALLOWED_STATEMENT_EXTENSIONS.contains(&ext.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "Unsupported statement format '.{ext}'. Allowed formats: {}",
            ALLOWED_STATEMENT_EXTENSIONS.join(", ")
        )));
    }

    // 6. Verify file size limit
    if meta.len() > MAX_STATEMENT_FILE_BYTES {
        return Err(AppError::InvalidInput(format!(
            "statement file size ({} bytes) exceeds 5 MB limit",
            meta.len()
        )));
    }

    Ok(canonical)
}

#[tauri::command]
#[specta::specta]
pub fn read_statement_file_cmd(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, AppError> {
    crate::vault::commands::require_session(&state).map_err(AppError::InvalidInput)?;
    let _ = state.get_db()?;

    let canonical = validate_statement_path(&path)?;
    use std::io::Read;
    let file = std::fs::File::open(&canonical)?;
    let mut handle = file.take(MAX_STATEMENT_FILE_BYTES + 1);
    let mut content = String::new();
    handle.read_to_string(&mut content)?;
    if content.len() as u64 > MAX_STATEMENT_FILE_BYTES {
        return Err(AppError::InvalidInput(format!(
            "statement file size exceeds {MAX_STATEMENT_FILE_BYTES} bytes limit"
        )));
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_empty_and_relative_paths_rejected() {
        assert!(validate_statement_path("").is_err());
        assert!(validate_statement_path("   ").is_err());
        assert!(validate_statement_path("relative/path/test.csv").is_err());
        assert!(validate_statement_path("./test.csv").is_err());
        assert!(validate_statement_path("../test.csv").is_err());
        assert!(validate_statement_path("/tmp/../test.csv").is_err());
    }

    #[test]
    fn test_valid_extensions_accepted() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        for ext in ALLOWED_STATEMENT_EXTENSIONS {
            let file_path = temp_dir.path().join(format!("statement.{ext}"));
            let mut f = File::create(&file_path).expect("create file");
            writeln!(f, "date,amount,desc\n2025-01-01,100,Test").expect("write file");

            let res = validate_statement_path(&file_path.to_string_lossy());
            assert!(
                res.is_ok(),
                "Extension .{ext} should be allowed, got: {:?}",
                res
            );
        }
    }

    #[test]
    fn test_disallowed_extension_rejected() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let disallowed = ["exe", "rs", "json", "pdf", "sh", "png", "db"];
        for ext in disallowed {
            let file_path = temp_dir.path().join(format!("file.{ext}"));
            let mut f = File::create(&file_path).expect("create file");
            writeln!(f, "dummy content").expect("write file");

            let res = validate_statement_path(&file_path.to_string_lossy());
            assert!(res.is_err(), "Extension .{ext} must be rejected");
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_forbidden_system_directories_rejected() {
        let forbidden = ["/etc/passwd", "/etc/hosts", "/sys", "/proc", "/dev/null"];
        for path in forbidden {
            if Path::new(path).exists() {
                let res = validate_statement_path(path);
                assert!(res.is_err(), "System path {path} must be rejected");
            }
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_symlinks_rejected() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let target_file = temp_dir.path().join("real.csv");
        let mut f = File::create(&target_file).expect("create file");
        writeln!(f, "data").expect("write");

        let symlink_file = temp_dir.path().join("link.csv");
        std::os::unix::fs::symlink(&target_file, &symlink_file).expect("create symlink");

        let res = validate_statement_path(&symlink_file.to_string_lossy());
        assert!(res.is_err(), "Symlinked files must be rejected");
        if let Err(AppError::InvalidInput(msg)) = res {
            assert!(msg.contains("symlink"));
        } else {
            panic!("Expected InvalidInput error for symlink");
        }
    }

    #[test]
    fn test_file_size_limit_exceeded() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let large_file = temp_dir.path().join("large.csv");
        let f = File::create(&large_file).expect("create file");
        f.set_len(MAX_STATEMENT_FILE_BYTES + 1).expect("set_len");

        let res = validate_statement_path(&large_file.to_string_lossy());
        assert!(res.is_err(), "File > 5 MB must be rejected");
        if let Err(AppError::InvalidInput(msg)) = res {
            assert!(msg.contains("exceeds 5 MB limit"));
        } else {
            panic!("Expected InvalidInput error for oversize file");
        }
    }

    #[test]
    fn test_sensitive_credential_paths_rejected() {
        let temp_dir = tempfile::tempdir().expect("tempdir");

        // 1. Inside .ssh folder
        let ssh_dir = temp_dir.path().join(".ssh");
        std::fs::create_dir_all(&ssh_dir).expect("mkdir .ssh");
        let key_file = ssh_dir.join("id_rsa.txt");
        let mut f = File::create(&key_file).expect("create file");
        writeln!(f, "dummy private key").expect("write");
        let res = validate_statement_path(&key_file.to_string_lossy());
        assert!(
            res.is_err(),
            "Path containing .ssh directory must be rejected"
        );

        // 2. Sensitive credential filenames directly in a non-sensitive folder with allowed extensions
        let sensitive_filenames = [
            "id_rsa.txt",
            "id_rsa.csv",
            "id_ed25519.csv",
            "id_ecdsa.ofx",
            "id_dsa.beancount",
            ".env.txt",
            ".bash_history.txt",
            "id_rsa_backup.csv",
        ];
        for filename in sensitive_filenames {
            let file_path = temp_dir.path().join(filename);
            let mut f = File::create(&file_path).expect("create file");
            writeln!(f, "secret content").expect("write");

            let res = validate_statement_path(&file_path.to_string_lossy());
            assert!(
                res.is_err(),
                "Sensitive credential file '{filename}' must be rejected even outside .ssh"
            );
        }
    }
}

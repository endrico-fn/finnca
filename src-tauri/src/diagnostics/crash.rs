use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Once, OnceLock};

static RE_PATH_UNIX: OnceLock<Regex> = OnceLock::new();
static RE_PATH_WIN: OnceLock<Regex> = OnceLock::new();
static RE_CURRENCY: OnceLock<Regex> = OnceLock::new();
static RE_MONEY: OnceLock<Regex> = OnceLock::new();
static RE_ACCOUNT: OnceLock<Regex> = OnceLock::new();
static RE_ACCOUNT_ID: OnceLock<Regex> = OnceLock::new();
static CRASH_INIT: Once = Once::new();

pub fn sanitize_panic_message(raw: &str) -> String {
    let re_unix = RE_PATH_UNIX.get_or_init(|| {
        Regex::new(r"(/home/[^/:\s\)'\x22\x3c\x3e]+|/Users/[^/:\s\)'\x22\x3c\x3e]+|/root)").unwrap()
    });
    let re_win = RE_PATH_WIN.get_or_init(|| {
        Regex::new(r"(?i)([a-zA-Z]:\\Users\\[^\\/:\s\)'\x22\x3c\x3e]+)").unwrap()
    });
    let re_curr = RE_CURRENCY.get_or_init(|| {
        Regex::new(r"(?i)\b(USD|IDR|EUR|GBP|SGD|JPY|CAD|AUD|CHF|CNY|HKD|NZD|KRW|Rp)\b|[$€£¥]").unwrap()
    });
    let re_money = RE_MONEY.get_or_init(|| {
        Regex::new(r"-?\b\d{1,3}(?:[.,]\d{3})*(?:[.,]\d{2,})?\b|-?\b\d+([.,]\d{2,})?\b").unwrap()
    });
    let re_acc = RE_ACCOUNT.get_or_init(|| {
        Regex::new(r"\b(Assets|Liabilities|Income|Expenses|Equity|Aset|Liabilitas|Kewajiban|Ekuitas|Pendapatan|Pengeluaran|Beban)(?:(?::[a-zA-Z0-9_\-]+)+|(?:\s*>\s*[A-Z0-9][a-zA-Z0-9_\-&]*(?:\s+(?:&|on|of|[A-Z0-9][a-zA-Z0-9_\-&]*))*)+)").unwrap()
    });
    let re_acc_id =
        RE_ACCOUNT_ID.get_or_init(|| Regex::new(r"\bacc_[0-9A-Za-z]{10,32}\b").unwrap());

    // 1. Redact home_dir directly first (including Windows/Unix variants)
    let mut scrubbed = raw.to_string();
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy().to_string();
        if !home_str.is_empty() {
            scrubbed = scrubbed.replace(&home_str, "[REDACTED_PATH]");
            let win_sep = home_str.replace('/', "\\");
            if !win_sep.is_empty() {
                scrubbed = scrubbed.replace(&win_sep, "[REDACTED_PATH]");
            }
            let unix_sep = home_str.replace('\\', "/");
            if !unix_sep.is_empty() {
                scrubbed = scrubbed.replace(&unix_sep, "[REDACTED_PATH]");
            }
        }
    }

    // Generic path redacting for other users or system paths
    scrubbed = re_unix.replace_all(&scrubbed, "[REDACTED_PATH]").to_string();
    scrubbed = re_win.replace_all(&scrubbed, "[REDACTED_PATH]").to_string();

    // 2. Scrub sensitive account IDs first, then hierarchical account paths
    scrubbed = re_acc_id
        .replace_all(&scrubbed, "[ACCOUNT_ID_MASKED]")
        .to_string();
    scrubbed = re_acc.replace_all(&scrubbed, "[ACCOUNT_MASKED]").to_string();

    // 3. Redact currency codes and symbols
    scrubbed = re_curr
        .replace_all(&scrubbed, "[CURRENCY_MASKED]")
        .to_string();

    // 4. Scrub numeric monetary figures
    scrubbed = re_money
        .replace_all(&scrubbed, "[NUMERIC_MASKED]")
        .to_string();

    scrubbed
}

pub fn get_crash_dir() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .map(|d| d.join("finnca").join("crashes"))
        .unwrap_or_else(|| PathBuf::from("crashes"))
}

pub fn rotate_crash_dumps(dir: &Path, max_keep: usize) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    let mut entries: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
    let read_dir = fs::read_dir(dir).map_err(|e| format!("Failed to read crash dir: {e}"))?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    let mtime = entry
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::UNIX_EPOCH);
                    entries.push((path, mtime));
                }
            }
        }
    }

    // Sort by modified time descending (newest first), tie-breaking by filename
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));

    // Remove older files beyond max_keep
    if entries.len() > max_keep {
        for (old_path, _) in &entries[max_keep..] {
            let _ = fs::remove_file(old_path);
        }
    }

    Ok(())
}

pub fn write_sanitized_crash_report(report: &serde_json::Value) -> Result<PathBuf, String> {
    let crash_dir = get_crash_dir();
    fs::create_dir_all(&crash_dir)
        .map_err(|e| format!("Failed to create crash directory: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&crash_dir, fs::Permissions::from_mode(0o700));
    }

    let timestamp_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let filename = format!("crash_{timestamp_millis}_{}.json", std::process::id());
    let file_path = crash_dir.join(filename);

    let formatted = serde_json::to_string_pretty(report)
        .map_err(|e| format!("Failed to serialize crash report: {e}"))?;

    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&file_path)
            .map_err(|e| format!("Failed to open crash dump with 0600 permissions: {e}"))?;
        file.write_all(formatted.as_bytes())
            .map_err(|e| format!("Failed to write crash dump: {e}"))?;
    }

    #[cfg(not(unix))]
    {
        fs::write(&file_path, formatted).map_err(|e| format!("Failed to write crash dump: {e}"))?;
    }

    // Keep at most 5 latest crash dumps
    let _ = rotate_crash_dumps(&crash_dir, 5);

    Ok(file_path)
}

pub fn init_crash_reporter() {
    CRASH_INIT.call_once(|| {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let backtrace = std::backtrace::Backtrace::capture();
            let timestamp = chrono::Utc::now().to_rfc3339();

            let raw_info = format!("{panic_info}");
            let sanitized_msg = sanitize_panic_message(&raw_info);
            let raw_bt = format!("{backtrace}");
            let sanitized_bt = sanitize_panic_message(&raw_bt);

            let report = serde_json::json!({
                "timestamp": timestamp,
                "version": env!("CARGO_PKG_VERSION"),
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "panic": sanitized_msg,
                "backtrace": sanitized_bt
            });

            let _ = write_sanitized_crash_report(&report);

            prev_hook(panic_info);
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_paths() {
        let msg = "Fatal at /home/endrico/Projects/finnca/src/main.rs and C:\\Users\\Administrator\\AppData";
        let cleaned = sanitize_panic_message(msg);
        assert!(!cleaned.contains("/home/endrico"));
        assert!(!cleaned.contains("C:\\Users\\Administrator"));
        assert!(cleaned.contains("[REDACTED_PATH]"));
    }

    #[test]
    fn test_sanitize_currencies_and_monetary_figures() {
        let msg = "Balance mismatch: 1500000.50 IDR does not balance with -100.00 USD, Rp 50.000 or $250.00";
        let cleaned = sanitize_panic_message(msg);
        assert!(!cleaned.contains("1500000.50"));
        assert!(!cleaned.contains("-100.00"));
        assert!(!cleaned.contains("IDR"));
        assert!(!cleaned.contains("USD"));
        assert!(!cleaned.contains("Rp"));
        assert!(!cleaned.contains("$"));
        assert!(cleaned.contains("[NUMERIC_MASKED]"));
        assert!(cleaned.contains("[CURRENCY_MASKED]"));
    }

    #[test]
    fn test_sanitize_account_names() {
        let msg = "Posting to Assets:Bank:BCA_Main and Liabilities:CreditCard:Mega failed with acc_01HN7V9J6Q1111111111111111";
        let cleaned = sanitize_panic_message(msg);
        assert!(!cleaned.contains("Assets:Bank:BCA_Main"));
        assert!(!cleaned.contains("Liabilities:CreditCard:Mega"));
        assert!(!cleaned.contains("acc_01HN7V9J6Q1111111111111111"));
        assert!(cleaned.contains("[ACCOUNT_MASKED]"));
        assert!(cleaned.contains("[ACCOUNT_ID_MASKED]"));
        assert_eq!(
            cleaned,
            "Posting to [ACCOUNT_MASKED] and [ACCOUNT_MASKED] failed with [ACCOUNT_ID_MASKED]"
        );

        // Bilingual and breadcrumb flat path testing
        let bilingual_msg = "Imbalance in Aset > Kas & Bank > Dompet Tunai and Beban > Makanan & Minuman with acc_01HN7V9J6Q1111111111111111";
        let bilingual_cleaned = sanitize_panic_message(bilingual_msg);
        assert!(!bilingual_cleaned.contains("Aset > Kas & Bank > Dompet Tunai"));
        assert!(!bilingual_cleaned.contains("Beban > Makanan & Minuman"));
        assert!(!bilingual_cleaned.contains("acc_01HN7V9J6Q1111111111111111"));
        assert!(bilingual_cleaned.contains("[ACCOUNT_MASKED]"));
        assert!(bilingual_cleaned.contains("[ACCOUNT_ID_MASKED]"));
        assert_eq!(
            bilingual_cleaned,
            "Imbalance in [ACCOUNT_MASKED] and [ACCOUNT_MASKED] with [ACCOUNT_ID_MASKED]"
        );

        // Conjunction preservation and variable name acc_* preservation
        let conj_msg = "Error in Assets > Banking > Checking in transfer for account with note and acc_cash";
        let conj_cleaned = sanitize_panic_message(conj_msg);
        assert_eq!(
            conj_cleaned,
            "Error in [ACCOUNT_MASKED] in transfer for account with note and acc_cash"
        );
    }

    #[test]
    fn test_rotate_crash_dumps() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path();

        for i in 0..8 {
            let file = path.join(format!("crash_{i:02}.json"));
            fs::write(&file, "{}").unwrap();
        }

        rotate_crash_dumps(path, 5).unwrap();

        let remaining: Vec<_> = fs::read_dir(path)
            .unwrap()
            .flatten()
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
            .collect();

        assert_eq!(remaining.len(), 5);
    }
}

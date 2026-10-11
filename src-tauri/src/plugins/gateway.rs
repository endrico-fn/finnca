use crate::shared::AppError;
use serde::{Deserialize, Serialize};

/// Trait representing a transaction posting leg with a minor unit integer amount.
pub trait DraftPostingLeg {
    fn amount(&self) -> i64;
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DraftPosting {
    pub account_id: String,
    pub amount: i64,
    #[serde(default)]
    pub memo: Option<String>,
}

impl DraftPostingLeg for DraftPosting {
    fn amount(&self) -> i64 {
        self.amount
    }
}

impl DraftPostingLeg for &DraftPosting {
    fn amount(&self) -> i64 {
        self.amount
    }
}

impl DraftPostingLeg for crate::ledger::dto::PostingInput {
    fn amount(&self) -> i64 {
        self.amount
    }
}

impl DraftPostingLeg for &crate::ledger::dto::PostingInput {
    fn amount(&self) -> i64 {
        self.amount
    }
}

impl DraftPostingLeg for (String, i64) {
    fn amount(&self) -> i64 {
        self.1
    }
}

impl DraftPostingLeg for i64 {
    fn amount(&self) -> i64 {
        *self
    }
}

impl DraftPostingLeg for &i64 {
    fn amount(&self) -> i64 {
        **self
    }
}

/// Recursively inspects JSON/binary bytes across host-plugin boundary.
/// Any IEEE-754 floating point number (including string-disguised floats "150.50", "NaN", "Infinity")
/// triggers an immediate `ERR_PLUGIN_FLOAT_VIOLATION`.
/// All monetary values must strictly be integer minor units (`i64`/`u64`).
pub fn validate_no_raw_floats(bytes: &[u8]) -> Result<(), AppError> {
    let val: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| AppError::InvalidInput(format!("Payload plugin bukan JSON valid: {e}")))?;

    fn walk_json(v: &serde_json::Value, path: &str) -> Result<(), AppError> {
        match v {
            serde_json::Value::Number(num) => {
                // Aksioma Zero-Float Mutlak: Seluruh tipe floating-point ditolak tanpa syarat!
                // Tidak ada toleransi untuk bilangan bulat berkoma (misal 15000.0, 1e2, 1e20).
                if num.is_f64() || (!num.is_i64() && !num.is_u64()) {
                    return Err(AppError::InvalidInput(format!(
                        "ERR_PLUGIN_FLOAT_VIOLATION: Nilai floating-point IEEE-754 terdeteksi pada '{path}': {num}. Seluruh nominal moneter wajib berupa integer minor units murni (i64/u64)."
                    )));
                }
            }
            serde_json::Value::String(s) => {
                let trimmed = s.trim();
                let lower = trimmed.to_lowercase();
                if lower == "nan"
                    || lower == "+nan"
                    || lower == "-nan"
                    || lower == "infinity"
                    || lower == "+infinity"
                    || lower == "-infinity"
                    || lower == "inf"
                    || lower == "-inf"
                    || lower == "+inf"
                {
                    return Err(AppError::InvalidInput(format!(
                        "ERR_PLUGIN_FLOAT_VIOLATION: Nilai non-finit (NaN/Infinity) terdeteksi pada string '{path}': \"{s}\"."
                    )));
                }

                if let Ok(f) = trimmed.parse::<f64>() {
                    if !f.is_finite() {
                        return Err(AppError::InvalidInput(format!(
                            "ERR_PLUGIN_FLOAT_VIOLATION: Nilai non-finit (NaN/Infinity) terdeteksi pada string '{path}': \"{s}\"."
                        )));
                    }
                }

                // Deteksi penyelundupan angka desimal pecahan / eksponensial dalam string (misal "15000.50", "1e3", "-0.5")
                if (trimmed.contains('.') || trimmed.contains('e') || trimmed.contains('E'))
                    && trimmed.parse::<f64>().is_ok()
                {
                    return Err(AppError::InvalidInput(format!(
                        "ERR_PLUGIN_FLOAT_VIOLATION: Representasi desimal/eksponensial pecahan terdeteksi pada string '{path}': \"{s}\". Seluruh data finansial wajib beroperasi pada integer minor units."
                    )));
                }
            }
            serde_json::Value::Array(items) => {
                for (idx, item) in items.iter().enumerate() {
                    walk_json(item, &format!("{path}[{idx}]"))?;
                }
            }
            serde_json::Value::Object(map) => {
                for (key, item) in map.iter() {
                    walk_json(item, &format!("{path}.{key}"))?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    walk_json(&val, "root")
}

/// Alias for validate_no_raw_floats as referenced in ADR 0007 § 4
pub fn validate_zero_float_integrity(payload: &[u8]) -> Result<(), AppError> {
    validate_no_raw_floats(payload)
}

/// Memvalidasi integritas invarian Debit-First dan keseimbangan double-entry
/// untuk draf transaksi yang diusulkan oleh plugin pihak ketiga sebelum disajikan ke pengguna:
/// - Minimum 2 kaki.
/// - Enforces Index 0 = Debit (amount >= 0) and Index 1 = Credit (amount <= 0).
/// - Enforces sum(Debit) + sum(Credit) == 0.
/// - Forbids zero amounts.
pub fn validate_plugin_draft_entry<T: DraftPostingLeg>(postings: &[T]) -> Result<(), AppError> {
    if postings.len() < 2 {
        return Err(AppError::InvalidInput(
            "ERR_PLUGIN_INVALID_ENTRY: Jurnal minimal harus memiliki 2 kaki transaksi.".into(),
        ));
    }

    // Penegakan Standar Invarian Debit-First untuk transaksi 2-kaki (Transfer / Simple Entry)
    if postings.len() == 2 {
        let debit_leg = &postings[0];
        let credit_leg = &postings[1];

        if debit_leg.amount() < 0 || credit_leg.amount() > 0 {
            return Err(AppError::InvalidInput(format!(
                "ERR_PLUGIN_INVARIANT_DEBIT_FIRST: Urutan kaki transaksi melanggar standar Debit-First. Kaki [0] wajib Debit (amount >= 0, ditemukan {}), Kaki [1] wajib Kredit (amount <= 0, ditemukan {}).",
                debit_leg.amount(),
                credit_leg.amount()
            )));
        }
    }

    // Pastikan tidak ada kaki transaksi dengan nominal 0
    for (idx, p) in postings.iter().enumerate() {
        if p.amount() == 0 {
            return Err(AppError::InvalidInput(format!(
                "ERR_PLUGIN_ZERO_AMOUNT: Kaki transaksi [{idx}] memiliki nominal nol."
            )));
        }
    }

    // Validasi keseimbangan Double-Entry (sum == 0)
    let total_balance: i128 = postings.iter().map(|p| p.amount() as i128).sum();
    if total_balance != 0 {
        return Err(AppError::InvalidInput(format!(
            "ERR_PLUGIN_UNBALANCED_ENTRY: Jurnal tidak seimbang. Total selisih: {total_balance} minor units."
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_float_gateway_rejects_floats() {
        // Fractional numbers
        assert!(validate_no_raw_floats(br#"{"amount": 15000.50}"#).is_err());
        // Whole floats (fract() == 0.0) must be unconditionally rejected
        assert!(validate_no_raw_floats(br#"{"amount": 15000.0}"#).is_err());
        // Scientific notation
        assert!(validate_no_raw_floats(br#"{"amount": 1e2}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": 1e20}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": 1.5e3}"#).is_err());

        // String-disguised floats
        assert!(validate_no_raw_floats(br#"{"amount": "15000.50"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "15000.0"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "-15000.75"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "1e3"}"#).is_err());

        // NaN and Infinity literals (case-insensitive)
        assert!(validate_no_raw_floats(br#"{"amount": "NaN"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "nan"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "+nan"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "-nan"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "Infinity"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "-Infinity"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "+Infinity"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "inf"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "-inf"}"#).is_err());
        assert!(validate_no_raw_floats(br#"{"amount": "+inf"}"#).is_err());

        // Nested inside array or object
        assert!(validate_no_raw_floats(br#"{"rows": [{"balance": 12.34}]}"#).is_err());
        assert!(validate_no_raw_floats(br#"[1, 2, 3.14]"#).is_err());
    }

    #[test]
    fn test_zero_float_gateway_accepts_integers() {
        // Pure integer minor units
        assert!(
            validate_no_raw_floats(br#"{"amount": 1500000, "trend_basis_points": 520}"#).is_ok()
        );
        assert!(validate_no_raw_floats(br#"{"negative": -1500000, "zero": 0}"#).is_ok());
        // Safe string text (not numeric floats)
        assert!(
            validate_no_raw_floats(br#"{"name": "Bank Central Asia", "date": "2026-10-06"}"#)
                .is_ok()
        );
        assert!(validate_no_raw_floats(
            br#"{"description": "Transfer to John.Doe", "version": "1.0.0"}"#
        )
        .is_ok());
        assert!(
            validate_no_raw_floats(br#"{"status": "ok", "tags": ["salary", "fixed_income"]}"#)
                .is_ok()
        );
    }

    #[test]
    fn test_debit_first_validation_enforced() {
        // Valid 2-leg transfer: Index 0 = Debit (>= 0), Index 1 = Credit (<= 0)
        let valid_postings = vec![
            DraftPosting {
                account_id: "bank".into(),
                amount: 100_000,
                memo: None,
            },
            DraftPosting {
                account_id: "cash".into(),
                amount: -100_000,
                memo: None,
            },
        ];
        assert!(validate_plugin_draft_entry(&valid_postings).is_ok());

        // Inverted order: Index 0 = Credit (< 0), Index 1 = Debit (> 0) -> MUST FAIL
        let inverted_postings = vec![
            DraftPosting {
                account_id: "cash".into(),
                amount: -100_000,
                memo: None,
            },
            DraftPosting {
                account_id: "bank".into(),
                amount: 100_000,
                memo: None,
            },
        ];
        let err = validate_plugin_draft_entry(&inverted_postings).unwrap_err();
        assert!(err.to_string().contains("ERR_PLUGIN_INVARIANT_DEBIT_FIRST"));

        // Unbalanced entry: sum != 0 -> MUST FAIL
        let unbalanced_postings = vec![
            DraftPosting {
                account_id: "bank".into(),
                amount: 100_000,
                memo: None,
            },
            DraftPosting {
                account_id: "cash".into(),
                amount: -90_000,
                memo: None,
            },
        ];
        let err = validate_plugin_draft_entry(&unbalanced_postings).unwrap_err();
        assert!(err.to_string().contains("ERR_PLUGIN_UNBALANCED_ENTRY"));

        // Zero amount leg -> MUST FAIL
        let zero_amount_postings = vec![
            DraftPosting {
                account_id: "bank".into(),
                amount: 0,
                memo: None,
            },
            DraftPosting {
                account_id: "cash".into(),
                amount: 0,
                memo: None,
            },
        ];
        let err = validate_plugin_draft_entry(&zero_amount_postings).unwrap_err();
        assert!(err.to_string().contains("ERR_PLUGIN_ZERO_AMOUNT"));

        // Less than 2 legs -> MUST FAIL
        let single_leg = vec![DraftPosting {
            account_id: "bank".into(),
            amount: 100_000,
            memo: None,
        }];
        let err = validate_plugin_draft_entry(&single_leg).unwrap_err();
        assert!(err.to_string().contains("ERR_PLUGIN_INVALID_ENTRY"));

        // Multi-leg balanced entry (>2 legs)
        let multi_leg = vec![
            DraftPosting {
                account_id: "salary_expense".into(),
                amount: 150_000,
                memo: None,
            },
            DraftPosting {
                account_id: "tax_withholding".into(),
                amount: -20_000,
                memo: None,
            },
            DraftPosting {
                account_id: "bank".into(),
                amount: -130_000,
                memo: None,
            },
        ];
        assert!(validate_plugin_draft_entry(&multi_leg).is_ok());
    }
}

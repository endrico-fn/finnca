use super::dto::PostingInput;
use crate::ledger::currency::{convert_minor_units, normalize_fx_rate, DEFAULT_FX_RATE};
use crate::shared::AppError;
use std::collections::HashMap;

pub fn validate_postings_balance(postings: &[PostingInput]) -> Result<(), AppError> {
    validate_postings_balance_with_context(postings, "IDR", DEFAULT_FX_RATE)
}

pub fn validate_postings_balance_with_context(
    postings: &[PostingInput],
    default_currency: &str,
    default_fx_rate: i64,
) -> Result<(), AppError> {
    if postings.len() < 2 {
        return Err(AppError::InvalidInput(
            "A double-entry transaction must contain at least 2 postings".into(),
        ));
    }

    for (i, p) in postings.iter().enumerate() {
        if p.amount == 0 {
            return Err(AppError::InvalidInput(format!(
                "Posting at index {i} has amount 0, which is not allowed"
            )));
        }
    }

    // 1. Check per-commodity balance
    let mut commodity_sums: HashMap<String, i64> = HashMap::new();
    for p in postings {
        let curr = p
            .currency
            .as_deref()
            .unwrap_or(default_currency)
            .trim()
            .to_uppercase();
        let entry = commodity_sums.entry(curr).or_insert(0);
        *entry = entry.checked_add(p.amount).ok_or_else(|| {
            AppError::InvalidInput("Integer overflow detected during postings sum".into())
        })?;
    }

    let is_per_commodity_balanced = commodity_sums.values().all(|&sum| sum == 0);
    if is_per_commodity_balanced {
        return Ok(());
    }

    // 2. If single-currency, it cannot be balanced via FX conversion
    if commodity_sums.len() <= 1 {
        let first_sum = commodity_sums.values().next().copied().unwrap_or(0);
        return Err(AppError::InvalidInput(format!(
            "Transaction is unbalanced: sum of postings is {first_sum}, expected 0"
        )));
    }

    // 3. Multi-currency: check functional FX conversion / cost balance
    let mut base_sum: i64 = 0;
    for (i, p) in postings.iter().enumerate() {
        let curr = p
            .currency
            .as_deref()
            .unwrap_or(default_currency)
            .trim()
            .to_uppercase();

        let base_amount = if let Some(cost) = p.cost_amount {
            if cost == 0 {
                return Err(AppError::InvalidInput(format!(
                    "Posting at index {i} has cost_amount 0, which is not allowed"
                )));
            }
            p.amount.signum() * cost.abs()
        } else if curr == "IDR" {
            p.amount
        } else {
            let fx = p
                .fx_rate
                .map(|r| normalize_fx_rate(r, default_fx_rate))
                .unwrap_or(default_fx_rate);
            convert_minor_units(p.amount, &curr, "IDR", fx)
        };

        base_sum = base_sum.checked_add(base_amount).ok_or_else(|| {
            AppError::InvalidInput(
                "Integer overflow detected during postings FX conversion sum".into(),
            )
        })?;
    }

    if base_sum != 0 {
        return Err(AppError::InvalidInput(format!(
            "Multi-currency transaction is unbalanced: net functional value is {base_sum} IDR, expected 0"
        )));
    }

    Ok(())
}

pub fn validate_date_format(date: &str) -> Result<(), AppError> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| {
        AppError::InvalidInput(format!("Invalid date format '{date}': expected YYYY-MM-DD"))
    })?;
    Ok(())
}

pub fn validate_not_in_locked_period(
    date: &str,
    closing_date: Option<&str>,
) -> Result<(), AppError> {
    if let Some(lock_date) = closing_date {
        let lock_date = lock_date.trim();
        if !lock_date.is_empty() && date <= lock_date {
            return Err(AppError::PeriodLocked(format!(
                "Accounting period on or before {lock_date} is closed and locked"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_double_entry_balance() {
        let postings = vec![
            PostingInput {
                id: None,
                account_id: "acc_1".into(),
                amount: 100000,
                memo: None,
                action: None,
                reconcile: None,
                ..Default::default()
            },
            PostingInput {
                id: None,
                account_id: "acc_2".into(),
                amount: -100000,
                memo: None,
                action: None,
                reconcile: None,
                ..Default::default()
            },
        ];

        assert!(validate_postings_balance(&postings).is_ok());
    }

    #[test]
    fn test_unbalanced_postings_rejected() {
        let postings = vec![
            PostingInput {
                id: None,
                account_id: "acc_1".into(),
                amount: 100000,
                memo: None,
                action: None,
                reconcile: None,
                ..Default::default()
            },
            PostingInput {
                id: None,
                account_id: "acc_2".into(),
                amount: -99000,
                memo: None,
                action: None,
                reconcile: None,
                ..Default::default()
            },
        ];

        let err = validate_postings_balance(&postings).unwrap_err();
        assert_eq!(err.code(), "ERR_INVALID_INPUT");
    }

    #[test]
    fn test_single_posting_rejected() {
        let postings = vec![PostingInput {
            id: None,
            account_id: "acc_1".into(),
            amount: 0,
            memo: None,
            action: None,
            reconcile: None,
            ..Default::default()
        }];

        let err = validate_postings_balance(&postings).unwrap_err();
        assert_eq!(err.code(), "ERR_INVALID_INPUT");
    }

    #[test]
    fn test_zero_amount_posting_rejected() {
        let postings = vec![
            PostingInput {
                id: None,
                account_id: "acc_1".into(),
                amount: 100,
                memo: None,
                action: None,
                reconcile: None,
                ..Default::default()
            },
            PostingInput {
                id: None,
                account_id: "acc_2".into(),
                amount: 0,
                memo: None,
                action: None,
                reconcile: None,
                ..Default::default()
            },
            PostingInput {
                id: None,
                account_id: "acc_3".into(),
                amount: -100,
                memo: None,
                action: None,
                reconcile: None,
                ..Default::default()
            },
        ];

        let err = validate_postings_balance(&postings).unwrap_err();
        assert_eq!(err.code(), "ERR_INVALID_INPUT");
    }

    #[test]
    fn test_date_validation() {
        assert!(validate_date_format("2026-09-14").is_ok());
        assert!(validate_date_format("2026-02-30").is_err());
        assert!(validate_date_format("14-09-2026").is_err());
        assert!(validate_date_format("not-a-date").is_err());
    }

    #[test]
    fn test_locked_period_validation() {
        assert!(validate_not_in_locked_period("2026-01-01", None).is_ok());
        assert!(validate_not_in_locked_period("2026-01-01", Some("")).is_ok());
        assert!(validate_not_in_locked_period("2026-01-01", Some("2025-12-31")).is_ok());
        assert!(validate_not_in_locked_period("2025-12-31", Some("2025-12-31")).is_err());
        assert!(validate_not_in_locked_period("2025-11-15", Some("2025-12-31")).is_err());
        let err = validate_not_in_locked_period("2025-12-31", Some("2025-12-31")).unwrap_err();
        assert_eq!(err.code(), "ERR_PERIOD_LOCKED");
    }

    #[test]
    fn test_multicurrency_per_commodity_balance() {
        let postings = vec![
            PostingInput {
                id: None,
                account_id: "usd_1".into(),
                amount: 100,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("USD".into()),
                fx_rate: None,
                cost_amount: None,
            },
            PostingInput {
                id: None,
                account_id: "usd_2".into(),
                amount: -100,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("USD".into()),
                fx_rate: None,
                cost_amount: None,
            },
            PostingInput {
                id: None,
                account_id: "idr_1".into(),
                amount: 50000,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                cost_amount: None,
            },
            PostingInput {
                id: None,
                account_id: "idr_2".into(),
                amount: -50000,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                cost_amount: None,
            },
        ];
        assert!(validate_postings_balance(&postings).is_ok());
    }

    #[test]
    fn test_multicurrency_fx_conversion_balance() {
        let postings = vec![
            PostingInput {
                id: None,
                account_id: "usd_acc".into(),
                amount: 100, // 100 cents = $1.00 USD
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("USD".into()),
                fx_rate: Some(16000), // $1.00 * 16000 = 16000 IDR
                cost_amount: None,
            },
            PostingInput {
                id: None,
                account_id: "idr_acc".into(),
                amount: -16000,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                cost_amount: None,
            },
        ];
        assert!(validate_postings_balance(&postings).is_ok());
    }

    #[test]
    fn test_multicurrency_cost_amount_balance() {
        let postings = vec![
            PostingInput {
                id: None,
                account_id: "foreign_asset".into(),
                amount: 500,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("EUR".into()),
                fx_rate: None,
                cost_amount: Some(85000),
            },
            PostingInput {
                id: None,
                account_id: "bank_idr".into(),
                amount: -85000,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                cost_amount: None,
            },
        ];
        assert!(validate_postings_balance(&postings).is_ok());
    }

    #[test]
    fn test_multicurrency_unbalanced_rejected() {
        let postings = vec![
            PostingInput {
                id: None,
                account_id: "usd_acc".into(),
                amount: 100,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("USD".into()),
                fx_rate: Some(16000), // converts to 16000 IDR
                cost_amount: None,
            },
            PostingInput {
                id: None,
                account_id: "idr_acc".into(),
                amount: -15000, // mismatch
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                cost_amount: None,
            },
        ];
        let err = validate_postings_balance(&postings).unwrap_err();
        assert_eq!(err.code(), "ERR_INVALID_INPUT");
    }

    #[test]
    fn test_multicurrency_negative_cost_amount_balance() {
        let postings = vec![
            PostingInput {
                id: None,
                account_id: "foreign_asset".into(),
                amount: -500, // selling EUR
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("EUR".into()),
                fx_rate: None,
                cost_amount: Some(85000), // positive cost magnitude correctly signed to negative
            },
            PostingInput {
                id: None,
                account_id: "bank_idr".into(),
                amount: 85000,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                cost_amount: None,
            },
        ];
        assert!(validate_postings_balance(&postings).is_ok());
    }

    #[test]
    fn test_multicurrency_eur_fx_conversion_balance() {
        let postings = vec![
            PostingInput {
                id: None,
                account_id: "eur_acc".into(),
                amount: 1000, // 10.00 EUR
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("EUR".into()),
                fx_rate: Some(17000), // 10 * 17000 = 170000 IDR
                cost_amount: None,
            },
            PostingInput {
                id: None,
                account_id: "idr_acc".into(),
                amount: -170000,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                cost_amount: None,
            },
        ];
        assert!(validate_postings_balance(&postings).is_ok());
    }

    #[test]
    fn test_multicurrency_zero_cost_amount_rejected() {
        let postings = vec![
            PostingInput {
                id: None,
                account_id: "usd_acc".into(),
                amount: 100,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("USD".into()),
                fx_rate: None,
                cost_amount: Some(0),
            },
            PostingInput {
                id: None,
                account_id: "idr_acc".into(),
                amount: 0,
                memo: None,
                action: None,
                reconcile: None,
                currency: Some("IDR".into()),
                fx_rate: None,
                cost_amount: None,
            },
        ];
        assert!(validate_postings_balance(&postings).is_err());
    }
}

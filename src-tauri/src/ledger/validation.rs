use super::dto::PostingInput;
use crate::shared::AppError;

pub fn validate_postings_balance(postings: &[PostingInput]) -> Result<(), AppError> {
    if postings.len() < 2 {
        return Err(AppError::InvalidInput(
            "A double-entry transaction must contain at least 2 postings".into(),
        ));
    }

    let mut sum: i64 = 0;
    for (i, p) in postings.iter().enumerate() {
        if p.amount == 0 {
            return Err(AppError::InvalidInput(format!(
                "Posting at index {i} has amount 0, which is not allowed"
            )));
        }

        sum = sum.checked_add(p.amount).ok_or_else(|| {
            AppError::InvalidInput("Integer overflow detected during postings sum".into())
        })?;
    }

    if sum != 0 {
        return Err(AppError::InvalidInput(format!(
            "Transaction is unbalanced: sum of postings is {sum}, expected 0"
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
            },
            PostingInput {
                id: None,
                account_id: "acc_2".into(),
                amount: -100000,
                memo: None,
                action: None,
                reconcile: None,
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
            },
            PostingInput {
                id: None,
                account_id: "acc_2".into(),
                amount: -99000,
                memo: None,
                action: None,
                reconcile: None,
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
            },
            PostingInput {
                id: None,
                account_id: "acc_2".into(),
                amount: 0,
                memo: None,
                action: None,
                reconcile: None,
            },
            PostingInput {
                id: None,
                account_id: "acc_3".into(),
                amount: -100,
                memo: None,
                action: None,
                reconcile: None,
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
}

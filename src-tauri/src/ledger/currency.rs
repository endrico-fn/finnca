pub const DEFAULT_FX_RATE: i64 = 16000;

pub fn normalize_fx_rate(raw: i64, fallback: i64) -> i64 {
    if raw == 1_000_000 || raw <= 0 {
        if fallback > 0 && fallback != 1_000_000 {
            fallback
        } else {
            DEFAULT_FX_RATE
        }
    } else {
        raw
    }
}

pub fn usd_minor_to_idr(amount_cents: i64, fx_rate: i64) -> i64 {
    let rate = normalize_fx_rate(fx_rate, DEFAULT_FX_RATE);
    let prod = amount_cents as i128 * rate as i128;
    let rounded = if prod >= 0 { prod + 50 } else { prod - 50 } / 100;
    rounded.clamp(i64::MIN as i128, i64::MAX as i128) as i64
}

pub fn idr_to_usd_minor(amount_idr: i64, fx_rate: i64) -> i64 {
    let rate = normalize_fx_rate(fx_rate, DEFAULT_FX_RATE);
    if rate == 0 {
        return 0;
    }
    let prod = amount_idr as i128 * 100;
    let rounded = if prod >= 0 {
        (prod + (rate as i128 / 2)) / rate as i128
    } else {
        (prod - (rate as i128 / 2)) / rate as i128
    };
    rounded.clamp(i64::MIN as i128, i64::MAX as i128) as i64
}

pub fn convert_minor_units(
    amount: i64,
    from_curr: &str,
    to_curr: &str,
    fx_rate: i64,
) -> i64 {
    if from_curr.eq_ignore_ascii_case(to_curr) {
        return amount;
    }
    if from_curr.eq_ignore_ascii_case("USD") && to_curr.eq_ignore_ascii_case("IDR") {
        return usd_minor_to_idr(amount, fx_rate);
    }
    if from_curr.eq_ignore_ascii_case("IDR") && to_curr.eq_ignore_ascii_case("USD") {
        return idr_to_usd_minor(amount, fx_rate);
    }
    amount
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_conversions() {
        assert_eq!(usd_minor_to_idr(100, 16000), 16000);
        assert_eq!(usd_minor_to_idr(250, 16000), 40000);
        assert_eq!(idr_to_usd_minor(16000, 16000), 100);
        assert_eq!(idr_to_usd_minor(40000, 16000), 250);
    }

    #[test]
    fn test_convert_minor_units() {
        assert_eq!(convert_minor_units(100, "USD", "IDR", 16500), 16500);
        assert_eq!(convert_minor_units(16500, "IDR", "USD", 16500), 100);
        assert_eq!(convert_minor_units(5000, "IDR", "IDR", 16500), 5000);
    }
}

pub const DEFAULT_FX_RATE: i64 = 16000;
const LEGACY_FX_SENTINEL: i64 = 1_000_000;

pub fn normalize_fx_rate(raw: i64, fallback: i64) -> i64 {
    if raw == LEGACY_FX_SENTINEL || raw <= 0 {
        if fallback > 0 && fallback != LEGACY_FX_SENTINEL {
            fallback
        } else {
            DEFAULT_FX_RATE
        }
    } else {
        raw
    }
}

fn div_half_up(num: i128, denom: i128) -> i128 {
    if denom == 0 {
        return 0;
    }
    let half = denom.abs() / 2;
    if num >= 0 {
        (num + half) / denom
    } else {
        (num - half) / denom
    }
}

fn clamp_i64(v: i128) -> i64 {
    v.clamp(i64::MIN as i128, i64::MAX as i128) as i64
}

pub fn usd_minor_to_idr(amount_cents: i64, fx_rate: i64) -> i64 {
    let rate = normalize_fx_rate(fx_rate, DEFAULT_FX_RATE);
    let prod = amount_cents as i128 * rate as i128;
    clamp_i64(div_half_up(prod, 100))
}

pub fn idr_to_usd_minor(amount_idr: i64, fx_rate: i64) -> i64 {
    let rate = normalize_fx_rate(fx_rate, DEFAULT_FX_RATE);
    if rate == 0 {
        return 0;
    }
    let prod = amount_idr as i128 * 100;
    clamp_i64(div_half_up(prod, rate as i128))
}

pub fn currency_minor_factor(curr: &str) -> i128 {
    let c = curr.trim().to_uppercase();
    match c.as_str() {
        "IDR" | "JPY" | "KRW" | "VND" | "CLP" | "HUF" | "PYG" | "RWF" | "UGX" | "BIF" | "DJF"
        | "GNF" | "KMF" | "XAF" | "XOF" | "XPF" => 1,
        "BHD" | "KWD" | "OMR" => 1000,
        _ => 100,
    }
}

pub fn convert_minor_units(amount: i64, from_curr: &str, to_curr: &str, fx_rate: i64) -> i64 {
    let from = from_curr.trim().to_uppercase();
    let to = to_curr.trim().to_uppercase();

    if from == to {
        return amount;
    }

    let rate = normalize_fx_rate(fx_rate, DEFAULT_FX_RATE);

    if to == "IDR" {
        let factor = currency_minor_factor(&from);
        let prod = amount as i128 * rate as i128;
        return clamp_i64(div_half_up(prod, factor));
    }

    if from == "IDR" {
        let factor = currency_minor_factor(&to);
        if rate == 0 {
            return 0;
        }
        let prod = amount as i128 * factor;
        return clamp_i64(div_half_up(prod, rate as i128));
    }

    let in_idr = convert_minor_units(amount, &from, "IDR", rate);
    convert_minor_units(in_idr, "IDR", &to, DEFAULT_FX_RATE)
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
    fn test_half_up_symmetry() {
        assert_eq!(usd_minor_to_idr(1, 16000), 160);
        assert_eq!(usd_minor_to_idr(-1, 16000), -160);
        assert_eq!(usd_minor_to_idr(1, 16001), 160);
        assert_eq!(idr_to_usd_minor(80, 16000), 1);
        assert_eq!(idr_to_usd_minor(-80, 16000), -1);
        assert_eq!(usd_minor_to_idr(0, 16000), 0);
        assert_eq!(usd_minor_to_idr(100, 0), 16000);
        assert_eq!(usd_minor_to_idr(100, 1_000_000), 16000);
    }

    #[test]
    fn test_overflow_clamp() {
        assert_eq!(usd_minor_to_idr(i64::MAX, 16000), i64::MAX);
        assert_eq!(usd_minor_to_idr(i64::MIN, 16000), i64::MIN);
        assert_eq!(idr_to_usd_minor(i64::MAX, 1), i64::MAX);
    }

    #[test]
    fn test_convert_minor_units() {
        assert_eq!(convert_minor_units(100, "USD", "IDR", 16500), 16500);
        assert_eq!(convert_minor_units(16500, "IDR", "USD", 16500), 100);
        assert_eq!(convert_minor_units(5000, "IDR", "IDR", 16500), 5000);
        assert_eq!(convert_minor_units(500, "EUR", "IDR", 17000), 85000);
        assert_eq!(convert_minor_units(85000, "IDR", "EUR", 17000), 500);
        assert_eq!(convert_minor_units(1000, "JPY", "IDR", 105), 105000);
    }
}

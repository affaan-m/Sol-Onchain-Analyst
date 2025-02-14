use bigdecimal::FromPrimitive;
use bigdecimal::{BigDecimal, ToPrimitive};

/// Converts an f64 value to a BigDecimal. Returns 0 if the conversion fails.
pub fn f64_to_decimal(value: f64) -> BigDecimal {
    BigDecimal::from_f64(value).unwrap_or_else(|| BigDecimal::from(0))
}

/// Converts a reference to a BigDecimal to an f64. Returns 0.0 if the conversion fails.
pub fn decimal_to_f64(value: &BigDecimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;

    #[test]
    fn test_f64_to_decimal() {
        assert_eq!(f64_to_decimal(1.0), BigDecimal::from(1));
        assert_eq!(f64_to_decimal(0.0), BigDecimal::from(0));
        assert_eq!(f64_to_decimal(3.14), BigDecimal::from_f64(3.14).unwrap());
    }

    #[test]
    fn test_decimal_to_f64() {
        let big_decimal_one = BigDecimal::from(1);
        let big_decimal_zero = BigDecimal::from(0);
        let big_decimal_pi = BigDecimal::from_f64(3.14).unwrap();

        assert_eq!(decimal_to_f64(&big_decimal_one), 1.0);
        assert_eq!(decimal_to_f64(&big_decimal_zero), 0.0);
        assert_eq!(decimal_to_f64(&big_decimal_pi), 3.14);
    }
}
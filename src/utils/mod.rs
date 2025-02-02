use bigdecimal::{BigDecimal, ToPrimitive};
use bigdecimal::FromPrimitive;

pub fn f64_to_decimal(value: f64) -> BigDecimal {
    BigDecimal::from_f64(value).unwrap_or_else(|| BigDecimal::from(0))
}

pub fn decimal_to_f64(value: &BigDecimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
}
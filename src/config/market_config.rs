use crate::error::{AgentError, AgentResult};
use std::env;
use crate::utils::f64_to_decimal;
use bigdecimal::BigDecimal;

#[derive(Debug, Clone)]
pub struct MarketConfig {
    pub price_change_threshold: BigDecimal,
    pub volume_surge_threshold: BigDecimal,
    pub base_confidence: BigDecimal,
    pub price_weight: BigDecimal,
    pub volume_weight: BigDecimal,
}

impl MarketConfig {
    pub fn new_from_env() -> AgentResult<Self> {
        Ok(Self {
            price_change_threshold: parse_decimal_env("PRICE_CHANGE_THRESHOLD", 0.05)?,
            volume_surge_threshold: parse_decimal_env("VOLUME_SURGE_THRESHOLD", 1.0)?,
            base_confidence: parse_decimal_env("BASE_CONFIDENCE", 0.5)?,
            price_weight: parse_decimal_env("PRICE_WEIGHT", 0.3)?,
            volume_weight: parse_decimal_env("VOLUME_WEIGHT", 0.2)?,
        })
    }

    pub fn validate(&self) -> AgentResult<()> {
        // Validate thresholds are positive
        if self.price_change_threshold <= BigDecimal::from(0) {
            return Err(AgentError::InvalidConfig(
                "price_change_threshold".into(),
                "must be greater than 0".into(),
            ));
        }
        if self.volume_surge_threshold <= BigDecimal::from(0) {
            return Err(AgentError::InvalidConfig(
                "volume_surge_threshold".into(),
                "must be greater than 0".into(),
            ));
        }

        // Validate weights sum to less than or equal to 1
        let total_weight = &self.price_weight + &self.volume_weight;
        if total_weight > BigDecimal::from(1) {
            return Err(AgentError::InvalidConfig(
                "weights".into(),
                "sum of weights must not exceed 1.0".into(),
            ));
        }

        Ok(())
    }
}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            price_change_threshold: f64_to_decimal(0.05),
            volume_surge_threshold: f64_to_decimal(1.0),
            base_confidence: f64_to_decimal(0.5),
            price_weight: f64_to_decimal(0.3),
            volume_weight: f64_to_decimal(0.2),
        }
    }
}

fn parse_decimal_env(key: &str, default: f64) -> AgentResult<BigDecimal> {
    match env::var(key) {
        Ok(val) => val.parse::<f64>()
            .map_err(|_| AgentError::InvalidConfig(
                key.to_string(),
                "must be a valid decimal number".to_string(),
            ))
            .map(f64_to_decimal),
        Err(_) => Ok(f64_to_decimal(default)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_config_defaults() {
        let config = MarketConfig::default();
        assert_eq!(config.price_change_threshold, f64_to_decimal(0.05));
        assert_eq!(config.volume_surge_threshold, f64_to_decimal(1.0));
        assert_eq!(config.base_confidence, f64_to_decimal(0.5));
    }

    #[test]
    fn test_market_config_validation() {
        // Valid config
        let config = MarketConfig::default();
        assert!(config.validate().is_ok());

        // Invalid: negative threshold
        let mut invalid_config = MarketConfig::default();
        invalid_config.price_change_threshold = f64_to_decimal(-0.1);
        assert!(invalid_config.validate().is_err());

        // Invalid: weights sum > 1
        let mut invalid_weights = MarketConfig::default();
        invalid_weights.price_weight = f64_to_decimal(0.6);
        invalid_weights.volume_weight = f64_to_decimal(0.5);
        assert!(invalid_weights.validate().is_err());
    }
}
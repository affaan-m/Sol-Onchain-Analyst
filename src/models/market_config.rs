use bigdecimal::BigDecimal;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConfig {
    pub price_change_threshold: BigDecimal,
    pub volume_surge_threshold: BigDecimal,
    pub base_confidence: BigDecimal,
    pub price_weight: BigDecimal,
    pub volume_weight: BigDecimal,
}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            price_change_threshold: BigDecimal::from(0.05),
            volume_surge_threshold: BigDecimal::from(0.2),
            base_confidence: BigDecimal::from(0.5),
            price_weight: BigDecimal::from(0.3),
            volume_weight: BigDecimal::from(0.2),
        }
    }
}
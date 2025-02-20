use serde::Serialize;
use uuid::Uuid;
use bson::{doc, DateTime};
use bigdecimal::ToPrimitive;

use crate::models::market_signal::MarketSignal;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSignalLog {
    pub id: Uuid,
    pub timestamp: DateTime,
    pub token_address: String,
    pub token_symbol: String,
    pub signal_type: String,
    pub price: f64,
    pub price_change_24h: Option<f64>,
    pub volume_change_24h: Option<f64>,
    pub confidence: f64,
    pub risk_score: f64,
    pub created_at: DateTime,
}

impl From<MarketSignal> for MarketSignalLog {
    fn from(signal: MarketSignal) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: DateTime::now(),
            token_address: signal.asset_address.clone(),
            token_symbol: signal
                .metadata
                .expect("Failed to get token symbol from metadata")
                .get("token_symbol")
                .and_then(|v| v.as_str())
                .unwrap_or(&signal.asset_address)
                .to_string(),
            signal_type: signal.signal_type.to_string(),
            price: signal.price.to_f64().unwrap_or_default(),
            price_change_24h: Some(
                signal
                    .price_change_24h
                    .and_then(|p| p.to_f64())
                    .unwrap_or_default(),
            ),
            volume_change_24h: signal.volume_change_24h.and_then(|v| v.to_f64()),
            confidence: signal.confidence.to_f64().unwrap_or_default(),
            risk_score: signal.risk_score.to_f64().unwrap_or_default(),
            created_at: signal.created_at.unwrap_or_else(DateTime::now),
        }
    }
}

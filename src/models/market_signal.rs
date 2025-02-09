use bigdecimal::BigDecimal;
// use bson::{Document, oid::ObjectId};
// use chrono::DateTime;
use serde::{Serialize, Deserialize};
use crate::utils::f64_to_decimal;
use std::fmt;
use chrono::Utc;
use bson::{self,DateTime, Document};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SignalType {
    Buy,
    Sell,
    Hold,
    StrongBuy,
    StrongSell,
    PriceSpike,
    PriceDrop,
    VolumeSurge,
}

impl fmt::Display for SignalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignalType::Buy => write!(f, "buy"),
            SignalType::Sell => write!(f, "sell"),
            SignalType::Hold => write!(f, "hold"),
            _ => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSignal {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub asset_address: String,
    pub signal_type: SignalType,
    pub price: BigDecimal,
    pub confidence: BigDecimal,
    pub risk_score: BigDecimal,
    pub sentiment_score: Option<BigDecimal>,
    pub price_change_24h: Option<BigDecimal>,
    pub volume_change_24h: Option<BigDecimal>,
    pub volume_change: BigDecimal,
    pub created_at: Option<DateTime>,
    pub timestamp: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Document>,
}

pub struct MarketSignalBuilder {
    asset_address: String,
    signal_type: SignalType,
    confidence: Option<BigDecimal>,
    risk_score: Option<BigDecimal>,
    sentiment_score: Option<BigDecimal>,
    volume_change_24h: Option<BigDecimal>,
    price_change_24h: Option<BigDecimal>,
    price: BigDecimal,
    volume_change: Option<BigDecimal>,
    timestamp: Option<DateTime>,
    metadata: Option<JsonValue>,
}

impl MarketSignalBuilder {
    pub fn new(asset_address: String, signal_type: SignalType, price: BigDecimal) -> Self {
        Self {
            asset_address,
            signal_type,
            confidence: None,
            risk_score: None,
            sentiment_score: None,
            volume_change_24h: None,
            price_change_24h: None,
            price,
            volume_change: None,
            timestamp: None,
            metadata: None,
        }
    }

    pub fn confidence(mut self, confidence: BigDecimal) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn risk_score(mut self, risk_score: BigDecimal) -> Self {
        self.risk_score = Some(risk_score);
        self
    }

    pub fn sentiment_score(mut self, sentiment_score: BigDecimal) -> Self {
        self.sentiment_score = Some(sentiment_score);
        self
    }

    pub fn volume_change_24h(mut self, volume_change: BigDecimal) -> Self {
        self.volume_change_24h = Some(volume_change);
        self
    }

    pub fn price_change_24h(mut self, price_change: BigDecimal) -> Self {
        self.price_change_24h = Some(price_change);
        self
    }

    pub fn volume_change(mut self, volume_change: BigDecimal) -> Self {
        self.volume_change = Some(volume_change);
        self
    }

    pub fn timestamp(mut self, timestamp: DateTime) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn metadata(mut self, metadata: JsonValue) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn build(self) -> MarketSignal {
        MarketSignal {
            id: None,
            asset_address: self.asset_address,
            signal_type: self.signal_type,
            confidence: self.confidence.unwrap_or_else(|| f64_to_decimal(0.5)),
            risk_score: self.risk_score.unwrap_or_else(|| f64_to_decimal(0.5)),
            sentiment_score: self.sentiment_score,
            volume_change_24h: self.volume_change_24h,
            price_change_24h: self.price_change_24h,
            price: self.price,
            volume_change: self.volume_change.unwrap_or_else(|| BigDecimal::from(0)),
            timestamp: DateTime::from(self.timestamp.unwrap_or_else(DateTime::now)),
            metadata: self.metadata.map(|v| bson::to_document(&v).unwrap()),
            created_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_market_signal_builder() {
        let price = f64_to_decimal(100.0);
        let signal = MarketSignalBuilder::new(
            "test_address".to_string(),
            SignalType::PriceSpike,
            price.clone(),
        )
        .confidence(f64_to_decimal(0.8))
        .risk_score(f64_to_decimal(0.3))
        .volume_change_24h(f64_to_decimal(0.15))
        .price_change_24h(f64_to_decimal(0.05))
        .metadata(json!({"source": "test"}))
        .build();

        assert_eq!(signal.asset_address, "test_address");
        assert_eq!(signal.price, price);
        assert_eq!(signal.confidence, f64_to_decimal(0.8));
        assert_eq!(signal.risk_score, f64_to_decimal(0.3));
        assert!(signal.metadata.is_some());
    }

    #[test]
    fn test_market_signal_builder_defaults() {
        let price = f64_to_decimal(100.0);
        let signal = MarketSignalBuilder::new(
            "test_address".to_string(),
            SignalType::Hold,
            price.clone(),
        )
        .build();

        assert_eq!(signal.confidence, f64_to_decimal(0.5)); // Default confidence
        assert_eq!(signal.risk_score, f64_to_decimal(0.5)); // Default risk score
        assert_eq!(signal.volume_change, BigDecimal::from(0)); // Default volume change
        assert!(signal.metadata.is_none());
    }
}
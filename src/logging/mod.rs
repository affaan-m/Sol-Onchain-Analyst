use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct MarketMetrics {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: Option<f64>,
    pub signal_type: Option<String>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketSignalLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub token_address: String,
    pub token_symbol: String,
    pub signal_type: String,
    pub price: f64,
    pub price_change_24h: Option<f64>,
    pub volume_change_24h: Option<f64>,
    pub confidence: f64,
    pub risk_score: f64,
    pub created_at: DateTime<Utc>,
}

pub struct RequestLogger {
    module: String,
    action: String,
}

impl RequestLogger {
    pub fn new(module: &str, action: &str) -> Self {
        Self {
            module: module.to_string(),
            action: action.to_string(),
        }
    }

    pub fn info(&self, message: &str) {
        info!(module = %self.module, action = %self.action, "{}", message);
    }

    pub fn warn(&self, message: &str) {
        warn!(module = %self.module, action = %self.action, "{}", message);
    }

    pub fn error(&self, message: &str) {
        error!(module = %self.module, action = %self.action, "{}", message);
    }
}

pub fn log_market_metrics(metrics: MarketMetrics) {
    info!(
        symbol = %metrics.symbol,
        price = %metrics.price,
        volume_24h = ?metrics.volume_24h,
        signal_type = ?metrics.signal_type,
        confidence = ?metrics.confidence,
        "Market metrics recorded"
    );
}

pub fn log_market_signal(signal: MarketSignalLog) {
    info!(
        token = %signal.token_symbol,
        signal_type = %signal.signal_type,
        price_change = ?signal.price_change_24h,
        volume_change = ?signal.volume_change_24h,
        confidence = %signal.confidence,
        risk_score = %signal.risk_score,
        "Market signal generated"
    );
}
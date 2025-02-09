use serde::Serialize;
use std::time::Instant;
use tracing::{info, warn, error};
use tracing_subscriber::{fmt, EnvFilter};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;

#[derive(Debug, Serialize)]
pub struct RequestLog {
    pub request_id: String,
    pub service: String,
    pub operation: String,
    pub start_time: DateTime<Utc>,
    pub duration_ms: u64,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MarketMetrics {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: Option<f64>,
    pub signal_type: Option<String>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct MarketSignalLog {
    pub id: Uuid,
    pub token_address: String,
    pub token_symbol: String,
    pub signal_type: String,
    pub price: f64,
    pub price_change_24h: Option<f64>,
    pub volume_change_24h: Option<f64>,
    pub confidence: f64,
    pub risk_score: f64,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub duration_ms: u64,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
}

pub struct RequestLogger {
    start_time: Instant,
    request_id: String,
    service: String,
    operation: String,
}

impl RequestLogger {
    pub fn new(service: &str, operation: &str) -> Self {
        Self {
            start_time: Instant::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
            service: service.to_string(),
            operation: operation.to_string(),
        }
    }

    pub fn success(self) {
        let duration = self.start_time.elapsed();
        let log = RequestLog {
            request_id: self.request_id,
            service: self.service,
            operation: self.operation,
            start_time: Utc::now() - chrono::Duration::from_std(duration).unwrap(),
            duration_ms: duration.as_millis() as u64,
            status: "success".to_string(),
            error: None,
        };
        info!(target: "request", "{}", serde_json::to_string(&log).unwrap());
    }

    pub fn error(self, error_msg: &str) {
        let duration = self.start_time.elapsed();
        let log = RequestLog {
            request_id: self.request_id,
            service: self.service,
            operation: self.operation,
            start_time: Utc::now() - chrono::Duration::from_std(duration).unwrap(),
            duration_ms: duration.as_millis() as u64,
            status: "error".to_string(),
            error: Some(error_msg.to_string()),
        };
        error!(target: "request", "{}", serde_json::to_string(&log).unwrap());
    }
}

pub fn log_market_metrics(metrics: MarketMetrics) {
    info!(
        target: "market_metrics",
        "{}",
        serde_json::to_string(&metrics).unwrap()
    );
}

pub fn log_market_signal(signal: MarketSignalLog) {
    info!(
        target: "market_signal",
        "Market signal detected: {}",
        serde_json::to_string(&signal).unwrap()
    );
}

pub fn log_performance(metrics: PerformanceMetrics) {
    if metrics.success {
        info!(
            target = "performance",
            "{}",
            serde_json::to_string(&metrics).unwrap()
        );
    } else {
        warn!(
            target = "performance",
            "{}",
            serde_json::to_string(&metrics).unwrap()
        );
    }
}

pub fn init_logging() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .with_ansi(true)
        .compact()
        .init();

    info!("Logging initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_request_logger() {
        let logger = RequestLogger::new("test_service", "test_operation");
        logger.success();
        // Verify log format would be tested in integration tests
    }

    #[test]
    fn test_market_metrics_serialization() {
        let metrics = MarketMetrics {
            symbol: "SOL".to_string(),
            price: 100.0,
            volume_24h: Some(1000000.0),
            signal_type: Some("BUY".to_string()),
            confidence: Some(0.8),
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["symbol"], "SOL");
        assert_eq!(parsed["price"], 100.0);
        assert_eq!(parsed["volume_24h"], 1000000.0);
        assert_eq!(parsed["signal_type"], "BUY");
        assert_eq!(parsed["confidence"], 0.8);
    }

    #[test]
    fn test_performance_metrics_serialization() {
        let metrics = PerformanceMetrics {
            operation: "market_analysis".to_string(),
            duration_ms: 100,
            success: true,
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["operation"], "market_analysis");
        assert_eq!(parsed["duration_ms"], 100);
        assert_eq!(parsed["success"], true);
        assert!(parsed["timestamp"].is_string());
    }
}

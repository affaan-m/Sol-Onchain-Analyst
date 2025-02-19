use crate::birdeye::api::{BirdeyeApi, BirdeyeClient};
use crate::config::mongodb::MongoDbPool;
use crate::config::birdeye_config::BirdeyeConfig;
use crate::models::market_signal::MarketSignal;
use crate::services::token_analytics::TokenAnalyticsService;
use anyhow::Result;
use bson::DateTime;
use chrono::{Duration, TimeZone, Utc};
use std::sync::Arc;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum Error {
    #[error("MongoDB error: {0}")]
    Mongo(#[from] mongodb::error::Error),
    #[error("Token analysis error: {0}")]
    Analysis(String),
    #[error("Data error: {0}")]
    Data(String),
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

pub struct AnalystAgent {
    analytics_service: Arc<TokenAnalyticsService>,
    db: Arc<MongoDbPool>,
}

impl AnalystAgent {
    pub async fn new(db_pool: Arc<MongoDbPool>, birdeye_api_key: String) -> Result<Self> {
        let birdeye_client: Arc<dyn BirdeyeApi> = Arc::new(BirdeyeClient::new(birdeye_api_key));
        let analytics_service =
            Arc::new(TokenAnalyticsService::new(db_pool.clone(), birdeye_client, None).await?);

        Ok(Self {
            analytics_service,
            db: db_pool,
        })
    }

    pub async fn analyze_token(&self, symbol: &str, address: &str) -> Result<Option<MarketSignal>> {
        info!("Starting analysis for token: {} ({})", symbol, address);

        // First fetch and store current token info
        let analytics = self
            .analytics_service
            .fetch_and_store_token_info(symbol, address)
            .await?;

        // Get historical data for analysis
        let now = DateTime::now();
        let timestamp_millis = now.timestamp_millis();
        let chrono_now = Utc.timestamp_millis_opt(timestamp_millis).unwrap();
        let start_time_chrono = chrono_now - Duration::days(7);
        let new_timestamp_millis = start_time_chrono.timestamp_millis();
        let start_time = DateTime::from_millis(new_timestamp_millis);
        let end_time = now;

        let history = self
            .analytics_service
            .get_token_history(address, start_time, end_time)
            .await?;

        info!(
            "Retrieved {} historical data points for analysis",
            history.len()
        );

        // Get previous analytics for comparison
        if self
            .analytics_service
            .get_previous_analytics(address)
            .await?
            .is_some()
        {
            info!("Generating market signals based on analysis");

            // Generate market signals based on the analysis
            return self
                .analytics_service
                .generate_market_signals(&analytics)
                .await
                .map_err(|e| Error::Analysis(e.to_string()).into());
        }

        info!("No previous analytics found for comparison");
        Ok(None)
    }
}
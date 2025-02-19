use crate::birdeye::api::{BirdeyeApi, BirdeyeClient};
use crate::config::mongodb::MongoDbPool;
use crate::config::mongodb::{MongoConfig, MongoPoolConfig};
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
            .await
            .map_err(|e| Error::Analysis(e.to_string()))?;

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
            .get_token_history(address, start_time, end_time, 100, 0)
            .await
            .map_err(|e| Error::Data(e.to_string()))?;

        info!(
            "Retrieved {} historical data points for analysis",
            history.len()
        );

        // Get previous analytics for comparison
        let previous = self
            .analytics_service
            .get_previous_analytics(address)
            .await
            .map_err(|e| Error::Data(e.to_string()))?;

        if let Some(prev_analytics) = previous {
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

    // async fn store_analysis(&self, analysis: &Analysis) -> Result<(), Error> {
    //     let collection = self.db.database("cainam").collection("market_analysis");

    //     collection
    //         .insert_one(analysis, None)
    //         .await
    //         .map_err(|e| Error::Mongo(e))?;

    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::mongodb::{MongoConfig, MongoPoolConfig};
    use crate::error::AgentError;

    #[tokio::test]
    async fn test_token_analysis() -> Result<()> {
        // Setup MongoDB connection
        let mongodb_uri = "mongodb://localhost:27017".to_string();
        let mongodb_database = "test_db".to_string();
        let config = MongoConfig {
            uri: mongodb_uri.clone(),
            database: mongodb_database.clone(),
            app_name: Some("test".to_string()),
            pool_config: MongoPoolConfig::default(),
        };
        let db_pool = Arc::new(MongoDbPool::create_pool(config).await?);

        // Create AnalystAgent
        let analyst = AnalystAgent::new(db_pool, "test_key".to_string()).await?;

        // Test token analysis
        let symbol = "SOL";
        let address = "So11111111111111111111111111111111111111112";
        let result = analyst.analyze_token(symbol, address).await;
        assert!(result.is_ok());

        Ok(())
    }
}

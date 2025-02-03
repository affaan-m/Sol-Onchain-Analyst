use anyhow::Result;
use crate::models::market_signal::MarketSignal;
use crate::services::token_analytics::TokenAnalyticsService;
use std::sync::Arc;
use chrono::Utc;

pub struct AnalystAgent {
    analytics_service: Arc<TokenAnalyticsService>,
}

impl AnalystAgent {
    pub fn new(analytics_service: Arc<TokenAnalyticsService>) -> Self {
        Self {
            analytics_service,
        }
    }

    pub async fn analyze_token(&self, symbol: &str, address: &str) -> Result<Option<MarketSignal>> {
        // First fetch and store current token info
        let analytics = self.analytics_service.fetch_and_store_token_info(symbol, address).await.map_err(|e| anyhow::anyhow!(e))?;
        
        // Get historical data for analysis
        let start_time = Utc::now() - chrono::Duration::days(7);
        let end_time = Utc::now();
        let _history = self.analytics_service.get_token_history(
            address,
            start_time,
            end_time,
            100,
            0
        ).await.map_err(|e| anyhow::anyhow!(e))?;

        // Get latest analytics for comparison
        let latest = self.analytics_service.get_latest_token_analytics(address).await.map_err(|e| anyhow::anyhow!(e))?;
        
        if let Some(latest) = latest {
            // Calculate volume change
            if let Some(current_volume) = analytics.volume_24h.clone() {
                if let Some(_volume_change) = self.analytics_service.calculate_volume_change(&current_volume, &latest) {
                    // Generate market signals based on the analysis
                    return self.analytics_service.generate_market_signals(&analytics).await.map_err(|e| anyhow::anyhow!(e));
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PgPool;
    use crate::config::MarketConfig;
    use crate::birdeye::{BirdeyeApi, MockBirdeyeApi, TokenInfo};
    use cainam_birdeye::BirdeyeClient;
    use sqlx::postgres::PgPoolOptions;
    use futures::future::FutureExt;

    async fn setup_test_db() -> Arc<PgPool> {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for tests");
            
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create database pool");
            
        Arc::new(pool)
    }

    fn setup_mock_birdeye() -> (Arc<dyn BirdeyeApi>, Arc<BirdeyeClient>) {
        let mut mock = MockBirdeyeApi::new();
        mock.expect_get_token_info()
            .returning(|_| Ok(TokenInfo {
                price: 100.0,
                volume24h: 1000000.0,
                price_change_24h: 5.0,
                liquidity: 500000.0,
                trade24h: 1000,
            }));

        (
            Arc::new(mock),
            Arc::new(BirdeyeClient::new("test_key".to_string()))
        )
    }

    #[tokio::test]
    async fn test_analyze_token() -> Result<()> {
        let db = setup_test_db().await;
        let (birdeye, birdeye_extended) = setup_mock_birdeye();
        let market_config = MarketConfig::default();
        
        let analytics_service = Arc::new(TokenAnalyticsService::new(
            db,
            birdeye,
            birdeye_extended,
            Some(market_config),
        ));

        let analyst = AnalystAgent::new(analytics_service);
        let signal = analyst.analyze_token("SOL", "test_address").await?;
        
        assert!(signal.is_some());
        Ok(())
    }
}
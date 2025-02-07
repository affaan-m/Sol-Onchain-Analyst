use crate::services::token_analytics::TokenAnalyticsService;
use crate::models::token_analytics::TokenAnalytics;
use crate::config::MarketConfig;
use crate::{
    birdeye::{MockBirdeyeApi, TokenInfo},
    error::AgentError,
    models::market_signal::SignalType,
};
use mockall::predicate::*;
use rig_mongodb::MongoDbPool;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_db, cleanup_test_db};
    use rig_mongodb::MongoDbPool;
    use std::sync::Arc;

    async fn setup_test_environment() -> (Arc<MongoDbPool>, Arc<MockBirdeyeApi>, Arc<BirdeyeExtendedClient>) {
        let db = setup_test_db("cainam_test")
            .await
            .expect("Failed to setup test database");
        
        let (birdeye, birdeye_extended) = setup_mock_birdeye();
        
        (db, birdeye, birdeye_extended)
    }

    async fn cleanup_test_environment(pool: &MongoDbPool) {
        cleanup_test_db(pool, "cainam_test")
            .await
            .expect("Failed to cleanup test database");
    }

    async fn setup_test_db() -> Arc<MongoDbPool> {
        let connection_string = "mongodb://localhost:27017";
        MongoDbPool::new_from_uri(connection_string, "cainam_test")
            .await
            .expect("Failed to create test database pool")
            .into()
    }

    fn setup_mock_birdeye() -> (Arc<MockBirdeyeApi>, Arc<cainam_birdeye::BirdeyeClient>) {
        let mut mock = MockBirdeyeApi::new();
        mock.expect_get_token_info()
            .returning(|_| {
                Ok(TokenInfo {
                    price: 100.0,
                    volume24h: 1000000.0,
                    price_change_24h: 5.0,
                    liquidity: 500000.0,
                    trade24h: 1000,
                })
            });

        (Arc::new(mock), Arc::new(cainam_birdeye::BirdeyeClient::new("test_key")))
    }

    #[tokio::test]
    async fn test_fetch_token_info_success() -> AgentResult<()> {
        let db = setup_test_db().await;
        let (birdeye, birdeye_extended) = setup_mock_birdeye();
        let market_config = MarketConfig::default();
        
        let service = TokenAnalyticsService::new(
            db,
            birdeye,
            birdeye_extended,
            Some(market_config),
        );

        let analytics = service.fetch_and_store_token_info("SOL", "test_address").await?;
        assert_eq!(analytics.token_symbol, "SOL");
        assert_eq!(analytics.price, f64_to_decimal(100.0));
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_token_price() -> AgentResult<()> {
        let db = setup_test_db().await;
        let mut mock = MockBirdeyeApi::new();
        mock.expect_get_token_info()
            .returning(|_| {
                Ok(TokenInfo {
                    price: -1.0, // Invalid price
                    volume24h: 1000000.0,
                    price_change_24h: 5.0,
                    liquidity: 500000.0,
                    trade24h: 1000,
                })
            });

        let service = TokenAnalyticsService::new(
            db,
            Arc::new(mock),
            Arc::new(cainam_birdeye::BirdeyeClient::new("test_key")),
            Some(MarketConfig::default()),
        );

        let result = service.fetch_and_store_token_info("SOL", "test_address").await;
        assert!(matches!(result, Err(AgentError::Validation(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_signal_confidence() -> AgentResult<()> {
        let db = setup_test_db().await;
        let (birdeye, birdeye_extended) = setup_mock_birdeye();
        let mut market_config = MarketConfig::default();
        
        // Set up config to generate invalid confidence
        market_config.base_confidence = f64_to_decimal(2.0); // Will result in confidence > 1
        
        let service = TokenAnalyticsService::new(
            db,
            birdeye,
            birdeye_extended,
            Some(market_config),
        );

        let result = service.fetch_and_store_token_info("SOL", "test_address").await;
        assert!(matches!(result, Err(AgentError::Validation(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_market_signal_generation() -> AgentResult<()> {
        let db = setup_test_db().await;
        let (birdeye, birdeye_extended) = setup_mock_birdeye();
        let market_config = MarketConfig::default();
        
        let service = TokenAnalyticsService::new(
            db.clone(),
            birdeye,
            birdeye_extended,
            Some(market_config),
        );

        // First store some historical data
        let mut tx = db.begin().await?;
        let analytics = TokenAnalytics {
            id: None,
            token_address: "test_address".to_string(),
            token_name: "Test Token".to_string(),
            token_symbol: "TEST".to_string(),
            price: f64_to_decimal(90.0), // Lower price to trigger price spike
            volume_24h: Some(f64_to_decimal(500000.0)),
            market_cap: Some(f64_to_decimal(1000000.0)),
            total_supply: Some(f64_to_decimal(10000.0)),
            holder_count: None,
            timestamp: Utc::now() - chrono::Duration::hours(1),
            created_at: None,
        };
        service.store_token_analytics_tx(&mut tx, &analytics).await?;
        tx.commit().await?;

        // Now fetch current data which should generate a signal
        let result = service.fetch_and_store_token_info("TEST", "test_address").await?;
        let signal = service.generate_market_signals(&result).await?;
        
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.signal_type, SignalType::PriceSpike);
        assert!(signal.confidence > f64_to_decimal(0.0));
        assert!(signal.confidence <= f64_to_decimal(1.0));
        Ok(())
    }

    #[tokio::test]
    async fn test_transaction_rollback() -> AgentResult<()> {
        let db = setup_test_db().await;
        let (birdeye, birdeye_extended) = setup_mock_birdeye();
        let market_config = MarketConfig::default();
        
        let service = TokenAnalyticsService::new(
            db.clone(),
            birdeye,
            birdeye_extended,
            Some(market_config),
        );

        // Start a transaction
        let mut tx = db.begin().await?;

        // Store valid analytics
        let analytics = TokenAnalytics {
            id: None,
            token_address: "test_address".to_string(),
            token_name: "Test Token".to_string(),
            token_symbol: "TEST".to_string(),
            price: f64_to_decimal(100.0),
            volume_24h: Some(f64_to_decimal(1000000.0)),
            market_cap: Some(f64_to_decimal(10000000.0)),
            total_supply: Some(f64_to_decimal(100000.0)),
            holder_count: None,
            timestamp: Utc::now(),
            created_at: None,
        };

        service.store_token_analytics_tx(&mut tx, &analytics).await?;

        // Rollback the transaction
        tx.rollback().await?;

        // Verify the data wasn't stored
        let result = service.get_latest_token_analytics("test_address").await?;
        assert!(result.is_none());
        
        Ok(())
    }
}
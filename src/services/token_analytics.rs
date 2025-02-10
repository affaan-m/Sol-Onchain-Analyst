use bigdecimal::{BigDecimal, ToPrimitive};
use futures::StreamExt;
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::Collection;
use rig::providers::openai::{self, EmbeddingModel};
use rig_mongodb::MongoDbVectorIndex;
use std::sync::Arc;
use crate::logging::RequestLogger;
use crate::birdeye::{BirdeyeApi, TokenInfo};
use crate::config::mongodb::MongoDbPool;
use crate::config::MarketConfig;
use crate::models::market_signal::{MarketSignal, SignalType, MarketSignalBuilder};
use crate::models::token_analytics::TokenAnalytics;
use crate::utils::f64_to_decimal;
use crate::error::{AgentError, AgentResult};
use bson::{doc, DateTime};
use uuid::Uuid;
use tracing::info;

const TEXT_EMBEDDING_ADA_002: &str = "text-embedding-ada-002";

#[derive(Debug, thiserror::Error)]
pub enum TokenAnalyticsError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Birdeye API error: {0}")]
    BirdeyeApi(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

// impl From<MongoDbError> for TokenAnalyticsError {
//     fn from(err: MongoDbError) -> Self {
//         Self::Database(err.to_string())
//     }
// }

// Remove the conflicting From implementation and use map_err where needed

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketMetrics {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: Option<f64>,
    pub signal_type: Option<String>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize)]
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

// Helper functions for logging with proper references
fn log_market_metrics(metrics: &MarketMetrics) {
    info!(
        symbol = %metrics.symbol,
        price = %metrics.price,
        volume_24h = ?metrics.volume_24h,
        signal_type = ?metrics.signal_type,
        confidence = ?metrics.confidence,
        "Market metrics recorded"
    );
}

fn log_market_signal(signal: &MarketSignalLog) {
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

pub struct TokenAnalyticsService {
    pool: Arc<MongoDbPool>,
    collection: Collection<TokenAnalytics>,
    signals_collection: Collection<MarketSignal>,
    vector_index: MongoDbVectorIndex<EmbeddingModel, TokenAnalytics>,
    birdeye: Arc<dyn BirdeyeApi>,
    market_config: MarketConfig,
}

impl TokenAnalyticsService {
    pub async fn new(
        pool: Arc<MongoDbPool>,
        birdeye: Arc<dyn BirdeyeApi>,
        market_config: Option<MarketConfig>,
    ) -> AgentResult<Self> {  // Change return type to AgentResult
        let db = pool.database("analytics");
        let collection = db.collection("token_analytics");
        println!(">> token_analytics collections {:?}", collection);

        let signals_collection = db.collection("market_signals");
        println!(">> market_signals collections {:?}", signals_collection);

        let openai_client = openai::Client::from_env();
        let model = openai_client.embedding_model(openai::TEXT_EMBEDDING_ADA_002);

        // Check if vector search index exists
        let list_indexes_command = doc! {
            "listSearchIndexes": "token_analytics"
        };
        
        let index_exists = match db.run_command(list_indexes_command).await {
            Ok(result) => {
                let indexes = result.get_document("cursor")
                    .and_then(|cursor| cursor.get_array("firstBatch"))
                    .map(|batch| !batch.is_empty())
                    .unwrap_or(false);
                if indexes {
                    info!("Vector search index already exists for token_analytics");
                }
                indexes
            }
            Err(_) => false
        };

        // Create vector search index if it doesn't exist
        if !index_exists {
            info!("Creating vector search index for token_analytics");
            let command = doc! {
                "createSearchIndexes": "token_analytics",
                "indexes": [{
                    "name": "vector_index",
                    "definition": {
                        "mappings": {
                            "dynamic": true,
                            "fields": {
                                "embedding": {
                                    "type": "knnVector",
                                    "dimensions": 1536,
                                    "similarity": "cosine"
                                }
                            }
                        }
                    }
                }]
            };
            
            match db.run_command(command).await {
                Ok(_) => info!("Created vector index for token_analytics"),
                Err(e) => {
                    info!("Failed to create vector index: {}", e);
                    return Err(AgentError::Database(e));
                }
            }
        }

        let vector_index = MongoDbVectorIndex::new(
            collection.clone(),
            model,
            "vector_index",  // Use the same name as created above
            Default::default()
        ).await.map_err(|e| AgentError::VectorStore(e.to_string()))?;
        
        Ok(Self {
            pool: pool,
            collection,
            signals_collection,
            vector_index,
            birdeye,
            market_config: market_config.unwrap_or_default(),
        })
    }

    pub async fn fetch_and_store_token_info(&self, symbol: &str, address: &str) -> AgentResult<TokenAnalytics> {
        let logger = RequestLogger::new("token_analytics", "fetch_and_store_token_info");

        // Fetch basic token info from Birdeye using address
        let token_info = match self.birdeye.get_token_info_by_address(address).await {
            Ok(info) => info,
            Err(e) => {
                let err = AgentError::BirdeyeApi(format!("Failed to fetch token info: {}", e));
                logger.error(&err.to_string());
                return Err(err);
            }
        };
        
        // Fetch extended token info using the comprehensive client
        let token_overview = match self.birdeye.get_token_info(address).await {
            Ok(overview) => overview,
            Err(e) => {
                let err = AgentError::BirdeyeApi(format!("Failed to fetch token overview: {}", e));
                logger.error(&err.to_string());
                return Err(err);
            }
        };
        
        // Validate token data and log metrics
        if token_info.price <= 0.0 {
            let err = AgentError::validation("Token price must be positive");
            logger.error(&err.to_string());
            return Err(err);
        }
        if token_info.volume_24h < 0.0 {
            let err = AgentError::validation("Token volume cannot be negative");
            logger.error(&err.to_string());
            return Err(err);
        }

        // Log market metrics
        let metrics = MarketMetrics {
            symbol: symbol.to_string(),
            price: token_info.price,
            volume_24h: Some(token_info.volume_24h),
            signal_type: None,
            confidence: None,
        };
        log_market_metrics(&metrics);

        // Convert to TokenAnalytics
        let analytics = match self.convert_to_analytics(address, symbol, token_info, token_overview).await {
            Ok(analytics) => analytics,
            Err(e) => {
                logger.error(&e.to_string());
                return Err(e);
            }
        };
        
        // Store in database
        let stored = self.store_token_analytics(&analytics).await?;
        
        // Generate and process market signals
        let signal = self.generate_market_signals(&stored).await?;
        
        // Store the signal if present
        if let Some(ref signal) = signal {
            let zero = BigDecimal::from(0);
            let one = BigDecimal::from(1);
            
            if signal.confidence < zero || signal.confidence > one {
                return Err(AgentError::validation("Signal confidence must be between 0 and 1"));
            }
            if signal.risk_score < zero || signal.risk_score > one {
                return Err(AgentError::validation("Risk score must be between 0 and 1"));
            }
            
            self.store_market_signal(signal).await?;
        }
        
        Ok(stored)
    }

    // TODO: zTgx hardcoded
    async fn convert_to_analytics(
        &self,
        address: &str,
        symbol: &str,
        info: TokenInfo,
        overview: TokenInfo,
    ) -> AgentResult<TokenAnalytics> {
        Ok(TokenAnalytics {
            id: None,
            token_address: address.to_string(),
            token_name: "overview.name".to_string(),
            token_symbol: symbol.to_string(),
            price: f64_to_decimal(info.price),
            volume_24h: Some(f64_to_decimal(info.volume_24h)),
            market_cap: Some(f64_to_decimal(11.0)),
            // market_cap: Some(f64_to_decimal(overview.market_cap)),
            total_supply: Some(f64_to_decimal(11.1)),
            // total_supply: Some(f64_to_decimal(overview.total_supply)),
            holder_count: None,
            timestamp: DateTime::now(),
            created_at: None,
            metadata: Some(doc!{}),
        })
    }

    pub async fn generate_market_signals(&self, analytics: &TokenAnalytics) -> AgentResult<Option<MarketSignal>> {
        let logger = RequestLogger::new("token_analytics", "generate_market_signals");

        // Get previous analytics for comparison
        let previous = match self.get_previous_analytics(&analytics.token_address).await {
            Ok(prev) => prev,
            Err(e) => {
                logger.error(&e.to_string());
                return Err(e);
            }
        };
        
        if let Some(prev) = previous {
            let price_change = (analytics.price.clone() - prev.price.clone()) / prev.price.clone();
            let volume_change = analytics.volume_24h.as_ref().map(|current| {
                let binding = BigDecimal::from(0);
                let prev = prev.volume_24h.as_ref().unwrap_or(&binding);
                (current.clone() - prev.clone()) / prev.clone()
            });

            if price_change > self.market_config.price_change_threshold.clone() {
                let signal = self.create_market_signal(
                    analytics,
                    SignalType::PriceSpike,
                    price_change.clone(),
                    volume_change.clone(),
                );
                self.log_signal(&signal, analytics);
                return Ok(Some(signal));
            } else if price_change < -self.market_config.price_change_threshold.clone() {
                let signal = self.create_market_signal(
                    analytics,
                    SignalType::PriceDrop,
                    price_change.abs(),
                    volume_change.clone(),
                );
                self.log_signal(&signal, analytics);
                return Ok(Some(signal));
            }

            if let Some(vol_change) = volume_change {
                if vol_change > self.market_config.volume_surge_threshold {
                    let signal = self.create_market_signal(
                        analytics,
                        SignalType::VolumeSurge,
                        price_change,
                        Some(vol_change),
                    );
                    self.log_signal(&signal, analytics);
                    return Ok(Some(signal));
                }
            }
        }
        
        Ok(None)
    }

    fn create_market_signal(
        &self,
        analytics: &TokenAnalytics,
        signal_type: SignalType,
        price_change: BigDecimal,
        volume_change: Option<BigDecimal>,
    ) -> MarketSignal {
        let confidence = self.calculate_confidence(
            price_change.clone(),
            volume_change.clone().unwrap_or_else(|| BigDecimal::from(0)),
        );

        MarketSignalBuilder::new(
            analytics.token_address.clone(),
            signal_type,
            analytics.price.clone(),
        )
        .confidence(confidence)
        .risk_score(f64_to_decimal(0.5))
        .price_change_24h(price_change)
        .volume_change_24h(volume_change.clone().unwrap_or_else(|| BigDecimal::from(0)))
        .volume_change(volume_change.unwrap_or_else(|| BigDecimal::from(0)))
        .timestamp(analytics.timestamp)
        .build()
    }

    async fn store_market_signal(&self, signal: &MarketSignal) -> AgentResult<()> {
        self.signals_collection
            .insert_one(signal)
            .await
            .map_err(|e| AgentError::Database(e))?;
            
        Ok(())
    }

    pub async fn get_previous_analytics(&self, address: &str) -> AgentResult<Option<TokenAnalytics>> {
        let filter = doc! {
            "token_address": address,
            "timestamp": { "$lt": DateTime::now() }
        };

        let options = FindOneOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .build();

        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AgentError::Database(e))
    }

    async fn store_token_analytics(&self, analytics: &TokenAnalytics) -> AgentResult<TokenAnalytics> {
        let result = self.collection
            .insert_one(analytics)
            .await
            .map_err(|e| AgentError::Database(e))?;
            
        let mut stored = analytics.clone();
        stored.id = result.inserted_id.as_object_id();
        Ok(stored)
    }

    pub async fn get_token_history(
        &self,
        address: &str,
        start_time: DateTime,
        end_time: DateTime,
        limit: i64,
        offset: i64,
    ) -> AgentResult<Vec<TokenAnalytics>> {
        let filter = doc! {
            "token_address": address,
            "timestamp": {
                "$gte": start_time,
                "$lte": end_time
            }
        };

        let options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .skip(Some(offset as u64))
            .limit(Some(limit))
            .build();

        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|e| AgentError::Database(e))?;

        let mut results = Vec::new();
        while let Some(doc) = cursor.next().await {
            results.push(doc?);
        }

        Ok(results)
    }

    pub async fn get_latest_token_analytics(&self, address: &str) -> AgentResult<Option<TokenAnalytics>> {
        let filter = doc! { "token_address": address };

        let analytics = self.collection
            .find_one(filter)
            .await
            .map_err(|e| AgentError::Database(e))?;

        Ok(analytics)
    }

    pub fn calculate_volume_change(&self, current: &BigDecimal, prev: &TokenAnalytics) -> Option<BigDecimal> {
        prev.volume_24h.as_ref().map(|prev_vol| {
            let zero = BigDecimal::from(0);
            let prev_value = if prev_vol == &zero { 
                BigDecimal::from(1) 
            } else { 
                prev_vol.clone() 
            };
            (current - prev_vol) / prev_value
        })
    }
}

impl TokenAnalyticsService {
    fn log_signal(&self, signal: &MarketSignal, analytics: &TokenAnalytics) {
        let signal_log = MarketSignalLog {
            id: Uuid::new_v4(),
            timestamp: DateTime::now(),
            token_address: signal.asset_address.clone(),
            token_symbol: analytics.token_symbol.clone(),
            signal_type: signal.signal_type.to_string(),
            price: analytics.price.to_f64().unwrap_or_default(),
            price_change_24h: Some(signal.price_change_24h.as_ref()
                .and_then(|p| p.to_f64())
                .unwrap_or_default()),
            volume_change_24h: signal.volume_change_24h.as_ref()
                .and_then(|v| v.to_f64()),
            confidence: signal.confidence.to_f64().unwrap_or_default(),
            risk_score: signal.risk_score.to_f64().unwrap_or_default(),
            created_at: DateTime::now(),
        };

        log_market_signal(&signal_log);
    }

    fn calculate_confidence(&self, price_change: BigDecimal, volume_change: BigDecimal) -> BigDecimal {
        self.market_config.base_confidence.clone() +
        (price_change * self.market_config.price_weight.clone()) +
        (volume_change * self.market_config.volume_weight.clone())
    }

    pub async fn process_market_signal(&self, signal: MarketSignal) -> AgentResult<()> {
        let _logger = RequestLogger::new("token_analytics", "process_market_signal");
        
        let signal_log = MarketSignalLog {
            id: Uuid::new_v4(),
            timestamp: DateTime::now(),
            token_address: signal.asset_address.clone(),
            token_symbol: signal.metadata.expect("Failed to get token symbol from metadata").get("token_symbol")
                .and_then(|v| v.as_str())
                
                .unwrap_or(&signal.asset_address)
                .to_string(),
            signal_type: signal.signal_type.to_string(),
            price: signal.price.to_f64().unwrap_or_default(),
            price_change_24h: signal.price_change_24h.map(|p| p.to_f64().unwrap_or_default()),
            volume_change_24h: signal.volume_change_24h.map(|v| v.to_f64().unwrap_or_default()),
            confidence: signal.confidence.to_f64().unwrap_or_default(),
            risk_score: signal.risk_score.to_f64().unwrap_or_default(),
            created_at: signal.created_at.unwrap_or_else(|| DateTime::now()),
        };

        log_market_signal(&signal_log);
        Ok(())
    }
}

impl From<MarketSignal> for MarketSignalLog {
    fn from(signal: MarketSignal) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: DateTime::now(),
            token_address: signal.asset_address.clone(),
            token_symbol: signal.metadata.expect("Failed to get token symbol from metadata").get("token_symbol")
                .and_then(|v| v.as_str())
                .unwrap_or(&signal.asset_address)
                .to_string(),
            signal_type: signal.signal_type.to_string(),
            price: signal.price.to_f64().unwrap_or_default(),
            price_change_24h: Some(signal.price_change_24h
                .and_then(|p| p.to_f64())
                .unwrap_or_default()),
            volume_change_24h: signal.volume_change_24h
                .and_then(|v| v.to_f64()),
            confidence: signal.confidence.to_f64().unwrap_or_default(),
            risk_score: signal.risk_score.to_f64().unwrap_or_default(),
            created_at: signal.created_at.unwrap_or_else(|| DateTime::now()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;
    use mongodb;  // Add explicit mongodb dependency

    mock! {
        pub BirdeyeApi {
            async fn get_token_info(&self, symbol: &str) -> AgentResult<TokenInfo>;
            async fn get_token_info_by_address(&self, address: &str) -> AgentResult<TokenInfo>;
        }
    }

    // async fn setup_test_db() -> MongoDbPool {
    //     let client_options = mongodb::options::ClientOptions::parse("mongodb://localhost:32770")
    //         .await
    //         .expect("Failed to parse MongoDB URI");
        
    //     MongoDbPool::create_pool(client_options)
    //         .await
    //         .expect("Failed to create MongoDB pool")
    // }

    fn setup_mock_birdeye() -> Arc<MockBirdeyeApi> {
        let mut mock = MockBirdeyeApi::new();
        mock.expect_get_token_info_by_address()
            .returning(|_| Ok(TokenInfo {
                price: 100.0,
                volume_24h: 1000000.0,
                price_change_24h: 5.0,
                liquidity: 500000.0,
                trade_24h: 1000,
            }));

        Arc::new(mock)
    }

    // #[tokio::test]
    // async fn test_market_signal_generation() -> AgentResult<()> {
    //     let pool = setup_test_db().await;
    //     let birdeye = setup_mock_birdeye();
        
    //     let market_config = MarketConfig {
    //         price_change_threshold: f64_to_decimal(0.05),
    //         volume_surge_threshold: f64_to_decimal(0.2),
    //         base_confidence: f64_to_decimal(0.5),
    //         price_weight: f64_to_decimal(0.3),
    //         volume_weight: f64_to_decimal(0.2),
    //     };
        
    //     let service = TokenAnalyticsService::new(
    //         pool,
    //         birdeye,
    //         Some(market_config),
    //     ).await?;

    //     // Create test data
    //     let analytics = TokenAnalytics {
    //         id: None,
    //         token_address: "test_address".to_string(),
    //         token_name: "Test Token".to_string(),
    //         token_symbol: "TEST".to_string(),
    //         price: f64_to_decimal(100.0),
    //         volume_24h: Some(f64_to_decimal(1000000.0)),
    //         market_cap: Some(f64_to_decimal(1000000.0)),
    //         total_supply: Some(f64_to_decimal(10000.0)),
    //         holder_count: Some(1000),
    //         timestamp: Utc::now(),
    //         created_at: Some(Utc::now()),
    //         metadata: Some(serde_json::json!({
    //             "network": "solana",
    //             "decimals": 9
    //         })),
    //     };

    //     let result = service.store_token_analytics(&analytics).await?;
    //     assert!(result.id.is_some(), "Should have assigned an ID");

    //     let history = service.get_token_history(
    //         "test_address",
    //         Utc::now() - chrono::Duration::hours(24),
    //         Utc::now(),
    //         10,
    //         0
    //     ).await?;

    //     assert!(!history.is_empty(), "Should have historical data");

    //     Ok(())
    // }
}

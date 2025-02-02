use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{DateTime, Utc, Datelike, Timelike};
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use crate::birdeye::{BirdeyeApi, TokenInfo};
use crate::models::market_signal::{MarketSignal, SignalType, MarketSignalBuilder};
use crate::models::token_analytics::TokenAnalytics;
use crate::utils::f64_to_decimal;
use crate::error::{AgentError, AgentResult};
use cainam_birdeye::BirdeyeClient as BirdeyeExtendedClient;
use cainam_birdeye::types::api::TokenOverview;
use crate::logging::{RequestLogger, MarketMetrics, log_market_metrics, MarketSignalLog, log_market_signal};
use uuid::Uuid;
use crate::config::MarketConfig;
use time::OffsetDateTime;
use serde_json::{self, Value};
use time::PrimitiveDateTime;

pub struct TokenAnalyticsService {
    db: Arc<PgPool>,
    birdeye: Arc<dyn BirdeyeApi>,
    birdeye_extended: Arc<BirdeyeExtendedClient>,
    market_config: MarketConfig,
}

impl TokenAnalyticsService {
    pub fn new(
        db: Arc<PgPool>,
        birdeye: Arc<dyn BirdeyeApi>,
        birdeye_extended: Arc<BirdeyeExtendedClient>,
        market_config: Option<MarketConfig>,
    ) -> Self {
        Self {
            db,
            birdeye,
            birdeye_extended,
            market_config: market_config.unwrap_or_default(),
        }
    }

    pub async fn fetch_and_store_token_info(&self, symbol: &str, address: &str) -> AgentResult<TokenAnalytics> {
        let logger = RequestLogger::new("token_analytics", "fetch_and_store_token_info");

        // Start a transaction
        let mut tx = match self.db.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                let err = AgentError::transaction(format!("Failed to begin transaction: {}", e));
                logger.error(&err.to_string());
                return Err(err);
            }
        };

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
        let token_overview = match self.birdeye_extended.get_token_overview(address.to_string()).await {
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
        if token_info.volume24h < 0.0 {
            let err = AgentError::validation("Token volume cannot be negative");
            logger.error(&err.to_string());
            return Err(err);
        }

        // Log market metrics
        log_market_metrics(MarketMetrics {
            symbol: symbol.to_string(),
            price: token_info.price,
            volume_24h: Some(token_info.volume24h),
            signal_type: None,
            confidence: None,
        });

        // Convert to TokenAnalytics
        let analytics = match self.convert_to_analytics(address, symbol, token_info, token_overview).await {
            Ok(analytics) => analytics,
            Err(e) => {
                logger.error(&e.to_string());
                return Err(e);
            }
        };
        
        // Store in database within transaction
        let stored = self.store_token_analytics_tx(&mut tx, &analytics).await?;
        
        // Generate and process market signals within the same transaction
        let signal = self.generate_market_signals(&stored).await?;
        
        // Validate market signal before committing
        if let Some(ref signal) = signal {
            let zero = BigDecimal::from(0);
            let one = BigDecimal::from(1);
            
            if signal.confidence < zero || signal.confidence > one {
                return Err(AgentError::validation("Signal confidence must be between 0 and 1"));
            }
            if signal.risk_score < zero || signal.risk_score > one {
                return Err(AgentError::validation("Risk score must be between 0 and 1"));
            }
            
            // Store the signal if validation passes
            self.store_market_signal_tx(&mut tx, signal).await?;
        }

        // Commit the transaction
        tx.commit().await
            .map_err(|e| AgentError::transaction(format!("Failed to commit transaction: {}", e)))?;
        
        Ok(stored)
    }

    async fn convert_to_analytics(
        &self,
        address: &str,
        symbol: &str,
        info: TokenInfo,
        overview: TokenOverview,
    ) -> AgentResult<TokenAnalytics> {
        Ok(TokenAnalytics {
            id: None,
            token_address: address.to_string(),
            token_name: overview.name,
            token_symbol: symbol.to_string(),
            price: f64_to_decimal(info.price),
            volume_24h: Some(f64_to_decimal(info.volume24h)),
            market_cap: Some(f64_to_decimal(overview.market_cap)),
            total_supply: Some(f64_to_decimal(overview.total_supply)),
            holder_count: None,
            timestamp: Utc::now(),
            created_at: None,
            metadata: Some(serde_json::json!({})),
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

    async fn store_market_signal_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        signal: &MarketSignal,
    ) -> AgentResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO market_signals (
                asset_address, signal_type, confidence, risk_score,
                sentiment_score, volume_change_24h, price_change_24h,
                timestamp, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            signal.asset_address,
            signal.signal_type.to_string(),
            signal.confidence,
            signal.risk_score,
            signal.sentiment_score,
            signal.volume_change_24h,
            signal.price_change_24h,
            datetime_to_offset(signal.timestamp),
            signal.metadata
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| AgentError::Database(e))?;

        Ok(())
    }

    pub async fn get_previous_analytics(&self, address: &str) -> AgentResult<Option<TokenAnalytics>> {
        let analytics = sqlx::query_as!(
            TokenAnalytics,
            r#"
            SELECT 
                id as "id?: Uuid",
                token_address,
                token_name,
                token_symbol,
                price as "price!: BigDecimal",
                volume_24h as "volume_24h?: BigDecimal",
                market_cap as "market_cap?: BigDecimal",
                total_supply as "total_supply?: BigDecimal",
                holder_count as "holder_count?: i32",
                timestamp as "timestamp!: DateTime<Utc>",
                created_at as "created_at?: DateTime<Utc>",
                metadata as "metadata?: Value"
            FROM token_analytics
            WHERE token_address = $1
              AND timestamp < NOW()
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
            address,
        )
        .fetch_optional(&*self.db)
        .await
        .map_err(|e| AgentError::Database(e))?;

        Ok(analytics)
    }

    pub async fn store_token_analytics_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        analytics: &TokenAnalytics,
    ) -> AgentResult<TokenAnalytics> {
        let default_metadata = serde_json::json!({});
        let metadata = analytics.metadata.as_ref().unwrap_or(&default_metadata);
        
        let stored = sqlx::query_as!(
            TokenAnalytics,
            r#"
            INSERT INTO token_analytics (
                token_address, token_name, token_symbol,
                price, volume_24h, market_cap, total_supply,
                holder_count, timestamp, created_at, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING 
                id as "id?: Uuid",
                token_address,
                token_name,
                token_symbol,
                price as "price!: BigDecimal",
                volume_24h as "volume_24h?: BigDecimal",
                market_cap as "market_cap?: BigDecimal",
                total_supply as "total_supply?: BigDecimal",
                holder_count as "holder_count?: i32",
                timestamp as "timestamp!: DateTime<Utc>",
                created_at as "created_at?: DateTime<Utc>",
                metadata as "metadata?: Value"
            "#,
            analytics.token_address,
            analytics.token_name,
            analytics.token_symbol,
            analytics.price,
            analytics.volume_24h,
            analytics.market_cap,
            analytics.total_supply,
            analytics.holder_count,
            datetime_to_offset(analytics.timestamp),
            analytics.created_at.map(datetime_to_offset),
            metadata
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AgentError::Database(e))?;

        Ok(stored)
    }

    pub async fn get_token_history(
        &self,
        address: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: i64,
        offset: i64,
    ) -> AgentResult<Vec<TokenAnalytics>> {
        let analytics = sqlx::query_as!(
            TokenAnalytics,
            r#"
            SELECT 
                id as "id?: Uuid",
                token_address,
                token_name,
                token_symbol,
                price as "price!: BigDecimal",
                volume_24h as "volume_24h?: BigDecimal",
                market_cap as "market_cap?: BigDecimal",
                total_supply as "total_supply?: BigDecimal",
                holder_count as "holder_count?: i32",
                timestamp as "timestamp!: DateTime<Utc>",
                created_at as "created_at?: DateTime<Utc>",
                metadata as "metadata?: Value"
            FROM token_analytics
            WHERE token_address = $1
              AND timestamp BETWEEN $2 AND $3
            ORDER BY timestamp DESC
            LIMIT $4
            OFFSET $5
            "#,
            address,
            datetime_to_offset(start_time),
            datetime_to_offset(end_time),
            limit,
            offset,
        )
        .fetch_all(&*self.db)
        .await
        .map_err(|e| AgentError::Database(e))?;

        Ok(analytics)
    }

    pub async fn get_latest_token_analytics(&self, address: &str) -> AgentResult<Option<TokenAnalytics>> {
        let analytics = sqlx::query_as!(
            TokenAnalytics,
            r#"
            SELECT 
                id as "id?: Uuid",
                token_address,
                token_name,
                token_symbol,
                price as "price!: BigDecimal",
                volume_24h as "volume_24h?: BigDecimal",
                market_cap as "market_cap?: BigDecimal",
                total_supply as "total_supply?: BigDecimal",
                holder_count as "holder_count?: i32",
                timestamp as "timestamp!: DateTime<Utc>",
                created_at as "created_at?: DateTime<Utc>",
                metadata as "metadata?: Value"
            FROM token_analytics
            WHERE token_address = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
            address,
        )
        .fetch_optional(&*self.db)
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
            timestamp: Utc::now(),
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
            created_at: Utc::now(),
        };

        log_market_signal(signal_log);
    }

    fn calculate_confidence(&self, price_change: BigDecimal, volume_change: BigDecimal) -> BigDecimal {
        self.market_config.base_confidence.clone() +
        (price_change * self.market_config.price_weight.clone()) +
        (volume_change * self.market_config.volume_weight.clone())
    }
}

impl From<MarketSignal> for MarketSignalLog {
    fn from(signal: MarketSignal) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            token_address: signal.asset_address.clone(),
            token_symbol: signal.asset_address.clone(), // Using address as symbol since we don't have symbol
            signal_type: signal.signal_type.to_string(),
            price: signal.price.to_f64().unwrap_or_default(),
            price_change_24h: Some(signal.price_change_24h
                .and_then(|p| p.to_f64())
                .unwrap_or_default()),
            volume_change_24h: signal.volume_change_24h
                .and_then(|v| v.to_f64()),
            confidence: signal.confidence.to_f64().unwrap_or_default(),
            risk_score: signal.risk_score.to_f64().unwrap_or_default(),
            created_at: signal.created_at.unwrap_or_else(|| Utc::now()),
        }
    }
}

fn datetime_to_offset(dt: DateTime<Utc>) -> OffsetDateTime {
    let month: u8 = dt.month() as u8;
    PrimitiveDateTime::new(
        time::Date::from_calendar_date(
            dt.year(),
            time::Month::try_from(month).unwrap(),
            dt.day() as u8
        ).unwrap(),
        time::Time::from_hms_nano(
            dt.hour() as u8,
            dt.minute() as u8,
            dt.second() as u8,
            dt.nanosecond()
        ).unwrap()
    ).assume_utc()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::birdeye::MockBirdeyeApi;
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

    fn setup_mock_birdeye() -> (Arc<dyn BirdeyeApi>, Arc<BirdeyeExtendedClient>) {
        let mut mock = MockBirdeyeApi::new();
        mock.expect_get_token_info()
            .returning(|_| {
                futures::future::ready(Ok(TokenInfo {
                    price: 100.0,
                    volume24h: 1000000.0,
                    price_change_24h: 5.0,
                    liquidity: 500000.0,
                    trade24h: 1000,
                })).boxed()
            });

        (
            Arc::new(mock),
            Arc::new(BirdeyeExtendedClient::new("test_key".to_string()))
        )
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
            metadata: Some(serde_json::json!({})),
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
            metadata: Some(serde_json::json!({})),
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

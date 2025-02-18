use crate::birdeye::BirdeyeApi;
use crate::models::token_info::TokenInfo;
use crate::config::mongodb::MongoDbPool;
use crate::config::MarketConfig;
use crate::error::{AgentError, AgentResult};
use crate::logging::{log_market_metrics, log_market_signal, RequestLogger};
use crate::models::market_signal::{MarketSignal, MarketSignalBuilder, SignalType};
use crate::models::token_analytics::TokenAnalytics;
use crate::utils::f64_to_decimal;
use crate::birdeye::api::{TokenMarketResponse, TrendingToken, OnchainMetrics};
use bigdecimal::{BigDecimal, ToPrimitive};
use bson::{doc, DateTime};
use futures::StreamExt;
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::Collection;
use std::sync::Arc;
use uuid::Uuid;

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

pub struct TokenAnalyticsService {
    pool: Arc<MongoDbPool>,
    collection: Collection<TokenAnalytics>,
    signals_collection: Collection<MarketSignal>,
    birdeye: Arc<dyn BirdeyeApi>,
    market_config: MarketConfig,
}

impl TokenAnalyticsService {
    pub async fn new(
        pool: Arc<MongoDbPool>,
        birdeye: Arc<dyn BirdeyeApi>,
        market_config: Option<MarketConfig>,
    ) -> AgentResult<Self> {
        let db = pool.database(&pool.get_config().database);
        let collection = db.collection("token_analytics");
        println!(">> token_analytics collections {:?}", collection);

        let signals_collection = db.collection("market_signals");
        println!(">> market_signals collections {:?}", signals_collection);

        Ok(Self {
            pool,
            collection,
            signals_collection,
            birdeye,
            market_config: market_config.unwrap_or_default(),
        })
    }

    pub async fn fetch_and_store_token_info(
        &self,
        symbol: &str,
        address: &str,
    ) -> AgentResult<TokenAnalytics> {
        let logger = RequestLogger::new("token_analytics", "fetch_and_store_token_info");

        // Fetch basic token info from Birdeye using address with retry logic
        let token_info = match self.fetch_with_retry(|| self.birdeye.get_token_info_by_address(address), 3).await {
            Ok(info) => info,
            Err(e) => {
                let err = AgentError::BirdeyeApi(format!("Failed to fetch token info after retries: {}", e));
                logger.error(&err.to_string());
                return Err(err);
            }
        };

        // Fetch market data with retry logic
        let market_data = match self.fetch_with_retry(|| self.birdeye.get_market_data(address), 3).await {
            Ok(data) => data,
            Err(e) => {
                let err = AgentError::BirdeyeApi(format!("Failed to fetch market data after retries: {}", e));
                logger.error(&err.to_string());
                return Err(err);
            }
        };

        // Validate token data
        self.validate_token_data(&token_info)?;

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
        let analytics = match self
            .convert_to_analytics(address, symbol, token_info, market_data)
            .await
        {
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
            self.validate_signal(signal)?;
            self.store_market_signal(signal).await?;
        }

        Ok(stored)
    }

    async fn convert_to_analytics(
        &self,
        address: &str,
        symbol: &str,
        info: TokenInfo,
        market_data: TokenMarketResponse,
    ) -> AgentResult<TokenAnalytics> {
        // Get on-chain metrics
        let onchain_metrics = match self.birdeye.get_onchain_metrics(address).await {
            Ok(metrics) => Some(metrics),
            Err(_) => None,
        };

        // Calculate technical indicators
        let price_history = match self.get_token_history(address, 
            DateTime::from(std::time::SystemTime::now() - std::time::Duration::from_secs(14 * 24 * 60 * 60)), 
            DateTime::now(), 
            100, 0).await {
            Ok(history) => history,
            Err(_) => vec![],
        };

        let (rsi, macd, macd_signal, bollinger_upper, bollinger_lower) = 
            if !price_history.is_empty() {
                let prices: Vec<f64> = price_history.iter()
                    .map(|h| h.price.to_f64().unwrap_or_default())
                    .collect();
                
                // Calculate RSI (14 periods)
                let rsi = self.calculate_rsi(&prices, 14);
                
                // Calculate MACD (12, 26, 9)
                let (macd, signal) = self.calculate_macd(&prices, 12, 26, 9);
                
                // Calculate Bollinger Bands (20 periods, 2 standard deviations)
                let (upper, lower) = self.calculate_bollinger_bands(&prices, 20, 2.0);
                
                (
                    Some(f64_to_decimal(rsi)),
                    Some(f64_to_decimal(macd)),
                    Some(f64_to_decimal(signal)),
                    Some(f64_to_decimal(upper)),
                    Some(f64_to_decimal(lower))
                )
            } else {
                (None, None, None, None, None)
            };

        Ok(TokenAnalytics {
            id: None,
            // Base token data
            token_address: address.to_string(),
            token_name: info.name,
            token_symbol: symbol.to_string(),
            decimals: info.decimals,
            logo_uri: info.logo_uri,
            
            // Price metrics
            price: f64_to_decimal(info.price),
            price_change_24h: info.price_change_24h.map(f64_to_decimal),
            price_change_7d: None, // Need to calculate from historical data
            
            // Volume metrics
            volume_24h: Some(f64_to_decimal(info.volume_24h)),
            volume_change_24h: info.volume_change_24h.map(f64_to_decimal),
            volume_by_price_24h: Some(f64_to_decimal(info.volume_24h * info.price)),
            
            // Market metrics
            market_cap: info.market_cap.map(f64_to_decimal),
            fully_diluted_market_cap: Some(f64_to_decimal(market_data.fdv)),
            circulating_supply: Some(f64_to_decimal(market_data.supply)),
            total_supply: Some(f64_to_decimal(market_data.total_supply)),
            
            // Liquidity metrics
            liquidity: Some(f64_to_decimal(info.liquidity)),
            liquidity_change_24h: None, // Need historical data
            
            // Trading metrics
            trades_24h: info.trade_24h,
            average_trade_size: info.trade_24h.map(|trades| 
                f64_to_decimal(info.volume_24h / trades as f64)
            ),
            
            // Holder metrics
            holder_count: onchain_metrics.as_ref().map(|m| m.unique_holders as i32),
            active_wallets_24h: onchain_metrics.as_ref().map(|m| m.active_wallets_24h as i32),
            whale_transactions_24h: onchain_metrics.as_ref().map(|m| m.whale_transactions_24h as i32),
            
            // Technical indicators
            rsi_14: rsi,
            macd,
            macd_signal,
            bollinger_upper,
            bollinger_lower,
            
            // Social metrics - Not available from Birdeye
            social_score: None,
            social_volume: None,
            social_sentiment: None,
            dev_activity: None,
            
            // Timestamps and metadata
            timestamp: DateTime::now(),
            created_at: None,
            last_trade_time: Some(info.timestamp),
            
            // Extensions and metadata
            metadata: Some(doc! {
                "source": "birdeye",
                "version": "1.0"
            }),
            
            // Vector embedding will be added in a separate process
            embedding: None
        })
    }

    fn calculate_rsi(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period + 1 {
            return 50.0; // Default value if not enough data
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..prices.len() {
            let diff = prices[i] - prices[i-1];
            if diff >= 0.0 {
                gains.push(diff);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-diff);
            }
        }

        let avg_gain = gains.iter().take(period).sum::<f64>() / period as f64;
        let avg_loss = losses.iter().take(period).sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    fn calculate_macd(&self, prices: &[f64], fast_period: usize, slow_period: usize, signal_period: usize) -> (f64, f64) {
        if prices.len() < slow_period {
            return (0.0, 0.0);
        }

        let fast_ema = self.calculate_ema(prices, fast_period);
        let slow_ema = self.calculate_ema(prices, slow_period);
        let macd_line = fast_ema - slow_ema;
        
        let signal_line = self.calculate_ema(&vec![macd_line], signal_period);
        
        (macd_line, signal_line)
    }

    fn calculate_ema(&self, prices: &[f64], period: usize) -> f64 {
        if prices.is_empty() {
            return 0.0;
        }
        
        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];
        
        for price in prices.iter().skip(1) {
            ema = (price - ema) * multiplier + ema;
        }
        
        ema
    }

    fn calculate_bollinger_bands(&self, prices: &[f64], period: usize, num_std_dev: f64) -> (f64, f64) {
        if prices.len() < period {
            return (prices[prices.len()-1], prices[prices.len()-1]);
        }

        let sma = prices.iter().take(period).sum::<f64>() / period as f64;
        
        let variance = prices.iter()
            .take(period)
            .map(|price| {
                let diff = price - sma;
                diff * diff
            })
            .sum::<f64>() / period as f64;
        
        let std_dev = variance.sqrt();
        
        let upper_band = sma + (std_dev * num_std_dev);
        let lower_band = sma - (std_dev * num_std_dev);
        
        (upper_band, lower_band)
    }

    pub async fn generate_market_signals(
        &self,
        analytics: &TokenAnalytics,
    ) -> AgentResult<Option<MarketSignal>> {
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

            let mut signal_opt = None;

            if price_change > self.market_config.price_change_threshold.clone() {
                let signal = self.create_market_signal(
                    analytics,
                    SignalType::PriceSpike,
                    price_change.clone(),
                    volume_change.clone(),
                );
                self.log_signal(&signal, analytics);
                signal_opt = Some(signal);
            } else if price_change < -self.market_config.price_change_threshold.clone() {
                let signal = self.create_market_signal(
                    analytics,
                    SignalType::PriceDrop,
                    price_change.abs(),
                    volume_change.clone(),
                );
                self.log_signal(&signal, analytics);
                signal_opt = Some(signal);
            } else if let Some(vol_change) = volume_change {
                if vol_change > self.market_config.volume_surge_threshold {
                    let signal = self.create_market_signal(
                        analytics,
                        SignalType::VolumeSurge,
                        price_change,
                        Some(vol_change),
                    );
                    self.log_signal(&signal, analytics);
                    signal_opt = Some(signal);
                }
            }

            // Process the signal if one was generated
            if let Some(signal) = signal_opt.clone() {
                if let Err(e) = self.process_market_signal(signal).await {
                    logger.error(&format!("Failed to process market signal: {}", e));
                    // Continue execution - don't fail if signal processing fails
                }
            }

            Ok(signal_opt)
        } else {
            Ok(None)
        }
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
            .map_err(AgentError::Database)?;

        Ok(())
    }

    pub async fn get_previous_analytics(
        &self,
        address: &str,
    ) -> AgentResult<Option<TokenAnalytics>> {
        let filter = doc! {
            "token_address": address,
            "timestamp": { "$lt": DateTime::now() }
        };

        let _options = FindOneOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .build();

        self.collection
            .find_one(filter)
            .await
            .map_err(AgentError::Database)
    }

    async fn store_token_analytics(
        &self,
        analytics: &TokenAnalytics,
    ) -> AgentResult<TokenAnalytics> {
        let result = self
            .collection
            .insert_one(analytics)
            .await
            .map_err(AgentError::Database)?;

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

        let mut cursor = self
            .collection
            .find(filter)
            .await
            .map_err(AgentError::Database)?;

        let mut results = Vec::new();
        while let Some(doc) = cursor.next().await {
            results.push(doc.map_err(AgentError::Database)?);
        }

        Ok(results)
    }

    pub async fn get_latest_token_analytics(
        &self,
        address: &str,
    ) -> AgentResult<Option<TokenAnalytics>> {
        let filter = doc! { "token_address": address };

        let analytics = self
            .collection
            .find_one(filter)
            .await
            .map_err(AgentError::Database)?;

        Ok(analytics)
    }

    pub fn calculate_volume_change(
        &self,
        current: &BigDecimal,
        prev: &TokenAnalytics,
    ) -> Option<BigDecimal> {
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
            price_change_24h: Some(
                signal
                    .price_change_24h
                    .as_ref()
                    .and_then(|p| p.to_f64())
                    .unwrap_or_default(),
            ),
            volume_change_24h: signal.volume_change_24h.as_ref().and_then(|v| v.to_f64()),
            confidence: signal.confidence.to_f64().unwrap_or_default(),
            risk_score: signal.risk_score.to_f64().unwrap_or_default(),
            created_at: DateTime::now(),
        };

        log_market_signal(&signal_log);
    }

    fn calculate_confidence(
        &self,
        price_change: BigDecimal,
        volume_change: BigDecimal,
    ) -> BigDecimal {
        self.market_config.base_confidence.clone()
            + (price_change * self.market_config.price_weight.clone())
            + (volume_change * self.market_config.volume_weight.clone())
    }

    pub async fn process_market_signal(&self, signal: MarketSignal) -> AgentResult<()> {
        let _logger = RequestLogger::new("token_analytics", "process_market_signal");

        let signal_log = MarketSignalLog {
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
            price_change_24h: signal
                .price_change_24h
                .map(|p| p.to_f64().unwrap_or_default()),
            volume_change_24h: signal
                .volume_change_24h
                .map(|v| v.to_f64().unwrap_or_default()),
            confidence: signal.confidence.to_f64().unwrap_or_default(),
            risk_score: signal.risk_score.to_f64().unwrap_or_default(),
            created_at: signal.created_at.unwrap_or_else(DateTime::now),
        };

        log_market_signal(&signal_log);
        Ok(())
    }

    // Helper method for validation
    fn validate_token_data(&self, token_info: &TokenInfo) -> AgentResult<()> {
        if token_info.price <= 0.0 {
            return Err(AgentError::validation("Token price must be positive"));
        }
        if token_info.volume_24h < 0.0 {
            return Err(AgentError::validation("Token volume cannot be negative"));
        }
        Ok(())
    }

    // Helper method for signal validation
    fn validate_signal(&self, signal: &MarketSignal) -> AgentResult<()> {
        let zero = BigDecimal::from(0);
        let one = BigDecimal::from(1);

        if signal.confidence < zero || signal.confidence > one {
            return Err(AgentError::validation("Signal confidence must be between 0 and 1"));
        }
        if signal.risk_score < zero || signal.risk_score > one {
            return Err(AgentError::validation("Risk score must be between 0 and 1"));
        }
        Ok(())
    }

    // Helper method for retrying API calls
    async fn fetch_with_retry<T, F, Fut>(&self, f: F, retries: u32) -> Result<T, anyhow::Error>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < retries {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);
                    if attempts < retries {
                        tokio::time::sleep(std::time::Duration::from_millis(500 * 2u64.pow(attempts))).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error during retry")))
    }
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
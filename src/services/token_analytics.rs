use crate::birdeye::BirdeyeApi;
use crate::config::mongodb::MongoDbPool;
use crate::config::market_config::MarketConfig;
use crate::error::{AgentError, AgentResult};
use crate::logging::market_metrics::MarketSignalLog;
use crate::logging::{log_market_metrics, log_market_signal, RequestLogger};
use crate::models::market_signal::{MarketSignal, MarketSignalBuilder, SignalType};
use crate::models::token_analytics::TokenAnalytics;
use crate::utils::f64_to_decimal;
use crate::birdeye::api::TokenOverviewResponse;
use bigdecimal::{BigDecimal, ToPrimitive};
use bson::{doc, DateTime};
use futures::StreamExt;
use mongodb::{options::{FindOptions, FindOneOptions}, Collection};
use std::sync::Arc;
use tracing::{debug, info};
use chrono::{Utc, Duration as ChronoDuration};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum TokenAnalyticsError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Birdeye API error: {0}")]
    BirdeyeApi(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketMetrics {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: Option<f64>,
    pub signal_type: Option<String>,    
    pub confidence: Option<f64>,
}

pub struct TokenAnalyticsService {
    collection: Collection<TokenAnalytics>,
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
        debug!("Initializing TokenAnalyticsService with collection: {:?}", collection);

        Ok(Self {
            collection,
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

        // Fetch token overview with retry logic
        let overview = match self
            .fetch_with_retry(|| self.birdeye.get_token_overview(address), 3)
            .await
        {
            Ok(data) => data,
            Err(e) => {
                let err = AgentError::BirdeyeApi(format!(
                    "Failed to fetch token overview after retries: {}",
                    e
                ));
                logger.error(&err.to_string());
                return Err(err);
            }
        };

        // Validate token data
        if overview.price <= 0.0 {
            let err = AgentError::validation("Token price must be positive");
            logger.error(&err.to_string());
            return Err(err);
        }

        // Log market metrics
        let metrics = MarketMetrics {
            symbol: symbol.to_string(),
            price: overview.price,
            volume_24h: Some(overview.v24h),
            signal_type: None,
            confidence: None,
        };
        log_market_metrics(&metrics);

        // Convert to TokenAnalytics
        let analytics = self.convert_to_analytics(address, symbol, overview).await?;

        // Store in database
        self.collection
            .insert_one(&analytics)
            .await
            .map_err(AgentError::Database)?;

        Ok(analytics)
    }

    async fn convert_to_analytics(
        &self,
        address: &str,
        symbol: &str,
        overview: TokenOverviewResponse,
    ) -> AgentResult<TokenAnalytics> {
        // Calculate technical indicators
        let price_history = match self
            .get_token_history(
                address,
                DateTime::from(
                    std::time::SystemTime::now()
                        - std::time::Duration::from_secs(14 * 24 * 60 * 60),
                ),
                DateTime::now(),
            )
            .await
        {
            Ok(history) => history,
            Err(_) => vec![],
        };

        let (rsi, macd, macd_signal, bollinger_upper, bollinger_lower) =
            if !price_history.is_empty() {
                let prices: Vec<f64> = price_history
                    .iter()
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
                    Some(f64_to_decimal(lower)),
                )
            } else {
                (None, None, None, None, None)
            };

        Ok(TokenAnalytics {
            id: None,
            // Base token data
            token_address: address.to_string(),
            token_name: overview.name,
            token_symbol: symbol.to_string(),
            decimals: overview.decimals as u8,
            logo_uri: Some(overview.logo_uri),

            // Price metrics
            price: f64_to_decimal(overview.price),
            price_change_24h: Some(f64_to_decimal(overview.price_change_24h_percent)),
            price_change_7d: Some(f64_to_decimal(
                (overview.price - overview.history_24h_price) / overview.history_24h_price * 100.0,
            )),

            // Volume metrics
            volume_24h: Some(f64_to_decimal(overview.v24h)),
            volume_change_24h: Some(f64_to_decimal(overview.v24h_change_percent)),
            volume_by_price_24h: Some(f64_to_decimal(overview.v24h_usd)),

            // Market metrics
            market_cap: Some(f64_to_decimal(overview.real_mc)),
            fully_diluted_market_cap: Some(f64_to_decimal(overview.fdv)),
            circulating_supply: Some(f64_to_decimal(overview.circulating_supply)),
            total_supply: Some(f64_to_decimal(overview.total_supply)),

            // Liquidity metrics
            liquidity: Some(f64_to_decimal(overview.liquidity)),
            liquidity_change_24h: Some(f64_to_decimal(overview.v24h_change_percent)),

            // Trading metrics
            trades_24h: Some(overview.trade24h),
            average_trade_size: Some(f64_to_decimal(
                overview.v24h_usd / overview.trade24h as f64
            )),

            // Holder metrics
            holder_count: Some(overview.holder as i32),
            active_wallets_24h: Some(overview.unique_wallet_24h as i32),
            whale_transactions_24h: Some((overview.trade24h / 100) as i32),

            // Technical indicators
            rsi_14: rsi,
            macd,
            macd_signal,
            bollinger_upper,
            bollinger_lower,

            // Timestamps and metadata
            timestamp: DateTime::now(),
            created_at: Some(DateTime::now()),
            last_trade_time: Some(DateTime::from_millis(overview.last_trade_unix_time * 1000)),

            // Extensions and metadata
            metadata: Some(doc! {
                "source": "birdeye",
                "version": "1.0",
                "number_markets": overview.number_markets,
                "unique_wallets_30m": overview.unique_wallet_30m,
                "unique_wallets_1h": overview.unique_wallet_1h,
                "unique_wallets_24h": overview.unique_wallet_24h,
                "trade_30m": overview.trade30m,
                "trade_24h": overview.trade24h,
                "buy_24h": overview.buy24h,
                "sell_24h": overview.sell24h,
                "v24h_usd": overview.v24h_usd,
                "v24h_change_percent": overview.v24h_change_percent,
                "price_change_24h_percent": overview.price_change_24h_percent,
            }),
            embedding: None,
        })
    }

    fn calculate_rsi(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period + 1 {
            return 50.0; // Default value if not enough data
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..prices.len() {
            let diff = prices[i] - prices[i - 1];
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

    fn calculate_macd(
        &self,
        prices: &[f64],
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> (f64, f64) {
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

    fn calculate_bollinger_bands(
        &self,
        prices: &[f64],
        period: usize,
        num_std_dev: f64,
    ) -> (f64, f64) {
        if prices.len() < period {
            return (prices[prices.len() - 1], prices[prices.len() - 1]);
        }

        let sma = prices.iter().take(period).sum::<f64>() / period as f64;

        let variance = prices
            .iter()
            .take(period)
            .map(|price| {
                let diff = price - sma;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;

        let std_dev = variance.sqrt();

        let upper_band = sma + (std_dev * num_std_dev);
        let lower_band = sma - (std_dev * num_std_dev);

        (upper_band, lower_band)
    }

    pub async fn get_previous_analytics(
        &self,
        address: &str,
    ) -> AgentResult<Option<TokenAnalytics>> {
        let filter = doc! {
            "token_address": address,
            "timestamp": { "$lt": DateTime::now() }
        };

        let options = FindOneOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .build();

        self.collection
            .find_one(filter)
            .with_options(options)
            .await
            .map_err(AgentError::Database)
    }

    pub async fn get_token_history(
        &self,
        address: &str,
        start_time: DateTime,
        end_time: DateTime,
    ) -> AgentResult<Vec<TokenAnalytics>> {
        let filter = doc! {
            "token_address": address,
            "timestamp": {
                "$gte": start_time,
                "$lte": end_time
            }
        };

        let options = FindOptions::builder()
            .sort(doc! { "timestamp": 1 })
            .build();

        let mut cursor = self
            .collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(AgentError::Database)?;

        let mut results = Vec::new();
        while let Some(result) = cursor.next().await {
            results.push(result.map_err(AgentError::Database)?);
        }

        Ok(results)
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
                        tokio::time::sleep(std::time::Duration::from_millis(
                            500 * 2u64.pow(attempts),
                        ))
                        .await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error during retry")))
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
                let prev_volume = prev.volume_24h.as_ref().unwrap_or(&binding);
                (current.clone() - prev_volume.clone()) / prev_volume.clone()
            });

            let mut signal_opt = None;

            if price_change.abs() > self.market_config.price_change_threshold {
                info!(
                    "Price spike detected: change={:.2}%, volume_change={:?}",
                    price_change.abs(),
                    volume_change.clone(),
                );
                let signal = self.create_market_signal(
                    analytics,
                    SignalType::PriceSpike,
                    price_change,
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
        let vol_change = volume_change.unwrap_or_else(|| BigDecimal::from(0));
        let confidence = self.calculate_confidence(
            price_change.clone(),
            vol_change.clone(),
        );

        let metadata = json!({
            "token_symbol": analytics.token_symbol.clone(),
            "token_name": analytics.token_name.clone()
        });

        MarketSignalBuilder::new(
            analytics.token_address.clone(),
            signal_type,
            analytics.price.clone(),
        )
        .confidence(confidence)
        .risk_score(f64_to_decimal(0.5))
        .sentiment_score(f64_to_decimal(0.5))
        .price_change_24h(price_change)
        .volume_change_24h(vol_change.clone())
        .volume_change(vol_change)
        .timestamp(analytics.timestamp)
        .metadata(metadata)
        .build()
    }

    fn log_signal(&self, signal: &MarketSignal, analytics: &TokenAnalytics) {
        let mut signal_log: MarketSignalLog = signal.clone().into();
        signal_log.token_symbol = analytics.token_symbol.clone(); // Override with actual token symbol
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

    pub async fn get_relevant_analytics(&self, query: &str) -> AgentResult<Vec<TokenAnalytics>> {
        debug!("Finding relevant analytics for query: {}", query);
        
        let twenty_four_hours_ago = Utc::now() - ChronoDuration::hours(24);
        let filter = doc! {
            "timestamp": {
                "$gte": DateTime::from_millis(twenty_four_hours_ago.timestamp_millis())
            }
        };
        
        let options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(10)
            .build();

        let mut cursor = self
            .collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(AgentError::Database)?;

        let mut results = Vec::new();
        while let Some(result) = cursor.next().await {
            results.push(result.map_err(AgentError::Database)?);
        }

        Ok(results)
    }

    pub async fn get_trending_tokens(&self, limit: i64) -> AgentResult<Vec<TokenAnalytics>> {
        debug!("Getting top {} trending tokens", limit);
        
        let twenty_four_hours_ago = Utc::now() - ChronoDuration::hours(24);
        let filter = doc! {
            "timestamp": {
                "$gte": DateTime::from_millis(twenty_four_hours_ago.timestamp_millis())
            }
        };
        
        let options = FindOptions::builder()
            .sort(doc! { 
                "volume_24h": -1,
                "price_change_24h": -1
            })
            .limit(limit)
            .build();

        let mut cursor = self
            .collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(AgentError::Database)?;

        let mut results = Vec::new();
        while let Some(result) = cursor.next().await {
            results.push(result.map_err(AgentError::Database)?);
        }

        Ok(results)
    }

    pub async fn get_token_analytics(&self, address: &str) -> AgentResult<Option<TokenAnalytics>> {
        debug!("Getting analytics for token: {}", address);
        
        let twenty_four_hours_ago = Utc::now() - ChronoDuration::hours(24);
        let filter = doc! {
            "token_address": address,
            "timestamp": {
                "$gte": DateTime::from_millis(twenty_four_hours_ago.timestamp_millis())
            }
        };
        
        let options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(1)
            .build();

        let mut cursor = self
            .collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(AgentError::Database)?;

        if let Some(result) = cursor.next().await {
            Ok(Some(result.map_err(AgentError::Database)?))
        } else {
            Ok(None)
        }
    }
}
use crate::birdeye::api::TokenOverviewResponse;
use crate::birdeye::BirdeyeApi;
use crate::config::market_config::MarketConfig;
use crate::config::mongodb::MongoDbPool;
use crate::error::{AgentError, AgentResult};
use crate::logging::market_metrics::MarketSignalLog;
use crate::logging::performance_metrics::PerformanceMetrics;
use crate::logging::{log_market_metrics, log_market_signal, log_performance, RequestLogger};
use crate::models::market_signal::{MarketSignal, MarketSignalBuilder, SignalType};
use crate::models::token_analytics::TokenAnalytics;
use crate::utils::f64_to_decimal;
use bigdecimal::BigDecimal;
use bson::{doc, DateTime};
use chrono::{Duration as ChronoDuration, Utc};
use fin_primitives::signals::indicators::{BollingerB, Ema, Macd, Rsi};
use fin_primitives::signals::{BarInput, Signal, SignalValue};
use futures::StreamExt;
use mongodb::{
    options::{FindOneOptions, FindOptions},
    Collection,
};
use rust_decimal::Decimal;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

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
        debug!(
            "Initializing TokenAnalyticsService with collection: {:?}",
            collection
        );

        Ok(Self {
            collection,
            birdeye,
            market_config: market_config.unwrap_or_default(),
        })
    }

    fn log_operation_performance(&self, operation: &str, start_time: Instant, success: bool) {
        log_performance(PerformanceMetrics {
            operation: operation.to_string(),
            duration_ms: start_time.elapsed().as_millis() as u64,
            success,
            timestamp: Utc::now(),
        });
    }

    async fn collect_cursor_results(
        &self,
        cursor: mongodb::Cursor<TokenAnalytics>,
    ) -> AgentResult<Vec<TokenAnalytics>> {
        let mut results = Vec::new();
        let mut cursor = cursor;
        while let Some(result) = cursor.next().await {
            results.push(result.map_err(AgentError::Database)?);
        }
        Ok(results)
    }

    pub async fn fetch_and_store_token_info(
        &self,
        symbol: &str,
        address: &str,
    ) -> AgentResult<TokenAnalytics> {
        let start_time = Instant::now();
        let operation = format!("fetch_token_info_{}", symbol);
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
                self.log_operation_performance(&operation, start_time, false);
                return Err(err);
            }
        };

        // Validate token data
        if overview.price <= 0.0 {
            let err = AgentError::validation("Token price must be positive");
            logger.error(&err.to_string());
            self.log_operation_performance(&operation, start_time, false);
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

        self.log_operation_performance(&operation, start_time, true);
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
                let prices: Vec<Decimal> = price_history
                    .iter()
                    .filter_map(|h| bigdecimal_to_decimal(&h.price))
                    .collect();

                // Calculate RSI (14 periods)
                let rsi = Self::calculate_rsi(&prices, 14);

                // Calculate MACD (12, 26, 9)
                let macd = Self::calculate_macd(&prices, 12, 26, 9);

                // Calculate Bollinger Bands (20 periods, 2 standard deviations)
                let bands = Self::calculate_bollinger_bands(&prices, 20, Decimal::TWO);

                // Each indicator is None until there is enough history to compute it
                (
                    rsi.map(decimal_to_bigdecimal),
                    macd.map(|(line, _)| decimal_to_bigdecimal(line)),
                    macd.map(|(_, signal)| decimal_to_bigdecimal(signal)),
                    bands.map(|(upper, _)| decimal_to_bigdecimal(upper)),
                    bands.map(|(_, lower)| decimal_to_bigdecimal(lower)),
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
            decimals: overview.decimals,
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
            average_trade_size: Some(f64_to_decimal(overview.v24h_usd / overview.trade24h as f64)),

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

    fn calculate_rsi(prices: &[Decimal], period: usize) -> Option<Decimal> {
        // Wilder-smoothed RSI over the whole series
        let mut rsi = Rsi::new("rsi", period).ok()?;
        last_value(&mut rsi, prices)
    }

    /// Returns `(macd_line, signal_line)`, or None until the signal EMA has warmed up.
    fn calculate_macd(
        prices: &[Decimal],
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> Option<(Decimal, Decimal)> {
        let mut fast_ema = Ema::new("ema_fast", fast_period).ok()?;
        let mut slow_ema = Ema::new("ema_slow", slow_period).ok()?;
        let mut macd = Macd::new("macd", fast_period, slow_period, signal_period).ok()?;

        let fast = last_value(&mut fast_ema, prices)?;
        let slow = last_value(&mut slow_ema, prices)?;
        let histogram = last_value(&mut macd, prices)?;

        // Macd yields the histogram (macd_line - signal_line), so recover the signal line from it
        let macd_line = fast - slow;
        Some((macd_line, macd_line - histogram))
    }

    /// Returns `(upper, lower)` over the most recent `period` prices.
    fn calculate_bollinger_bands(
        prices: &[Decimal],
        period: usize,
        num_std_dev: Decimal,
    ) -> Option<(Decimal, Decimal)> {
        let mut bollinger = BollingerB::new("bollinger", period, num_std_dev).ok()?;
        last_value(&mut bollinger, prices)?;
        let (upper, _middle, lower) = bollinger.bands()?;
        Some((upper, lower))
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

        let options = FindOptions::builder().sort(doc! { "timestamp": 1 }).build();

        let cursor = self
            .collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(AgentError::Database)?;

        self.collect_cursor_results(cursor).await
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
        let confidence = self.calculate_confidence(price_change.clone(), vol_change.clone());

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

    /// Get analytics data relevant to a specific query.
    /// This method is used by the LLM service for semantic analysis and by the CLI for showing recent trading activity.
    /// 
    /// # Arguments
    /// * `query` - The search query to find relevant analytics
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

        let cursor = self
            .collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(AgentError::Database)?;

        self.collect_cursor_results(cursor).await
    }

    /// Get the top trending tokens based on volume and price changes.
    /// Used by both the LLM service for market insights and the CLI for monitoring.
    /// 
    /// # Arguments
    /// * `limit` - Maximum number of trending tokens to return
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

        let cursor = self
            .collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(AgentError::Database)?;

        self.collect_cursor_results(cursor).await
    }

    /// Get the most recent analytics data for a specific token.
    /// Used for token comparison in the LLM service and for monitoring in the CLI.
    /// 
    /// # Arguments
    /// * `address` - The token's address
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

/// Feeds every price into `signal` and returns its final value, or None while it is still warming up.
fn last_value<S: Signal>(signal: &mut S, prices: &[Decimal]) -> Option<Decimal> {
    let mut value = SignalValue::Unavailable;
    for price in prices {
        value = signal.update(&BarInput::from_close(*price)).ok()?;
    }
    value.as_decimal()
}

fn bigdecimal_to_decimal(value: &BigDecimal) -> Option<Decimal> {
    let s = value.to_string();
    Decimal::from_str(&s)
        .or_else(|_| Decimal::from_scientific(&s))
        .ok()
}

fn decimal_to_bigdecimal(value: Decimal) -> BigDecimal {
    BigDecimal::from_str(&value.to_string()).unwrap_or_else(|_| BigDecimal::from(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimals(values: &[&str]) -> Vec<Decimal> {
        values
            .iter()
            .map(|v| Decimal::from_str(v).unwrap())
            .collect()
    }

    #[test]
    fn test_rsi_uses_wilder_smoothing() {
        // Wilder's worked example: average gain / loss over the first 14 changes
        let prices = decimals(&[
            "44.34", "44.09", "44.15", "43.61", "44.33", "44.83", "45.10", "45.42", "45.84",
            "46.08", "45.89", "46.03", "45.61", "46.28", "46.28", "46.00", "46.03", "46.41",
        ]);
        let rsi = TokenAnalyticsService::calculate_rsi(&prices[..15], 14).unwrap();
        assert_eq!(rsi.round_dp(2), Decimal::from_str("70.46").unwrap());

        // Later changes are folded in with Wilder smoothing
        let rsi = TokenAnalyticsService::calculate_rsi(&prices, 14).unwrap();
        assert_eq!(rsi.round_dp(2), Decimal::from_str("69.35").unwrap());

        // Not enough history yet
        assert!(TokenAnalyticsService::calculate_rsi(&prices[..14], 14).is_none());
    }

    #[test]
    fn test_macd_signal_line_is_ema_of_macd_line() {
        let prices: Vec<Decimal> = (0..40).map(|i| Decimal::from(100 + i * i % 17)).collect();
        let (macd_line, signal_line) =
            TokenAnalyticsService::calculate_macd(&prices, 12, 26, 9).unwrap();
        assert_eq!(
            macd_line.round_dp(6),
            Decimal::from_str("0.043907").unwrap()
        );
        assert_eq!(
            signal_line.round_dp(6),
            Decimal::from_str("-0.174441").unwrap()
        );

        // Needs slow_period + signal_period - 1 prices before the signal line exists
        assert!(TokenAnalyticsService::calculate_macd(&prices[..33], 12, 26, 9).is_none());
        assert!(TokenAnalyticsService::calculate_macd(&prices[..34], 12, 26, 9).is_some());
    }

    #[test]
    fn test_bollinger_bands_use_latest_window() {
        let mut prices = vec![Decimal::from(1000); 20];
        prices.extend(vec![Decimal::from(10); 20]);
        let (upper, lower) =
            TokenAnalyticsService::calculate_bollinger_bands(&prices, 20, Decimal::TWO).unwrap();
        assert_eq!(upper, Decimal::from(10));
        assert_eq!(lower, Decimal::from(10));
    }

    #[test]
    fn test_decimal_round_trip() {
        let price = BigDecimal::from_str("0.000012345678901234").unwrap();
        let decimal = bigdecimal_to_decimal(&price).unwrap();
        assert_eq!(decimal_to_bigdecimal(decimal), price);
    }
}

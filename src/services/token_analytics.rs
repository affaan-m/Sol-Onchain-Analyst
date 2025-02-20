use crate::birdeye::BirdeyeApi;
use crate::models::market_data::TokenMarketResponse;
use crate::config::mongodb::MongoDbPool;
use crate::config::market_config::MarketConfig;
use crate::error::{AgentError, AgentResult};
use crate::logging::{log_market_metrics, log_market_signal, RequestLogger};
use crate::models::market_signal::{MarketSignal, MarketSignalBuilder, SignalType};
use crate::models::token_analytics::TokenAnalytics;
use crate::models::token_info::TokenInfo;
use crate::utils::f64_to_decimal;
use crate::birdeye::api::TokenOverviewResponse;
use bigdecimal::{BigDecimal, ToPrimitive};
use bson::{doc, DateTime};
use futures::StreamExt;
use mongodb::options::FindOneOptions;
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
    birdeye: Arc<dyn BirdeyeApi>,
}

impl TokenAnalyticsService {
    pub async fn new(
        pool: Arc<MongoDbPool>,
        birdeye: Arc<dyn BirdeyeApi>,
        _market_config: Option<MarketConfig>,
    ) -> AgentResult<Self> {
        let db = pool.database(&pool.get_config().database);
        let collection = db.collection("token_analytics");
        println!(">> token_analytics collections {:?}", collection);

        Ok(Self {
            pool,
            collection,
            birdeye,
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
        let analytics = match self
            .convert_to_analytics(address, symbol, overview)
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

        Ok(stored)
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
            liquidity_change_24h: Some(f64_to_decimal(overview.v24h_change_percent)), // Using volume change as proxy for liquidity change

            // Trading metrics
            trades_24h: Some(overview.trade24h),
            average_trade_size: Some(f64_to_decimal(
                overview.v24h_usd / overview.trade24h as f64
            )),

            // Holder metrics
            holder_count: Some(overview.holder as i32),
            active_wallets_24h: Some(overview.unique_wallet_24h as i32),
            whale_transactions_24h: Some((overview.trade24h / 100) as i32), // Estimating whale transactions as 1% of total trades

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
    ) -> AgentResult<Vec<TokenAnalytics>> {
        let filter = doc! {
            "token_address": address,
            "timestamp": {
                "$gte": start_time,
                "$lte": end_time
            }
        };

        let options = mongodb::options::FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .build();

        let mut cursor = self
            .collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(AgentError::Database)?;

        let mut results = Vec::new();
        while let Some(doc) = cursor.next().await {
            results.push(doc.map_err(AgentError::Database)?);
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
}

// Move From implementation outside of TokenAnalyticsService impl block
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

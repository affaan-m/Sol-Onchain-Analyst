use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const BIRDEYE_API_URL: &str = "https://public-api.birdeye.so";

#[derive(Debug, Clone)]
pub struct BirdeyeProvider {
    client: Arc<Client>,
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price_usd: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDepth {
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBookEntry {
    pub price: f64,
    pub size: f64,
}

#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    async fn get_token_info(&self, address: &str) -> Result<TokenInfo>;
    async fn get_market_depth(&self, address: &str) -> Result<MarketDepth>;
    async fn get_price_history(&self, address: &str, interval: &str) -> Result<Vec<PricePoint>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: i64,
    pub price: f64,
    pub volume: f64,
}

impl BirdeyeProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Arc::new(Client::new()),
            api_key: api_key.to_string(),
        }
    }

    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", BIRDEYE_API_URL, endpoint);
        
        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .query(params)
            .send()
            .await?
            .error_for_status()?;

        let data = response.json::<T>().await?;
        Ok(data)
    }
}

#[async_trait]
impl MarketDataProvider for BirdeyeProvider {
    async fn get_token_info(&self, address: &str) -> Result<TokenInfo> {
        self.make_request(
            "/public/token",
            &[("address", address)],
        ).await
    }

    async fn get_market_depth(&self, address: &str) -> Result<MarketDepth> {
        self.make_request(
            "/public/orderbook",
            &[("address", address)],
        ).await
    }

    async fn get_price_history(&self, address: &str, interval: &str) -> Result<Vec<PricePoint>> {
        self.make_request(
            "/public/price_history",
            &[
                ("address", address),
                ("interval", interval),
            ],
        ).await
    }
}

// Additional helper functions for market analysis
impl BirdeyeProvider {
    pub async fn analyze_liquidity(&self, address: &str) -> Result<LiquidityAnalysis> {
        let depth = self.get_market_depth(address).await?;
        
        let total_bid_liquidity: f64 = depth.bids
            .iter()
            .map(|entry| entry.price * entry.size)
            .sum();

        let total_ask_liquidity: f64 = depth.asks
            .iter()
            .map(|entry| entry.price * entry.size)
            .sum();

        Ok(LiquidityAnalysis {
            total_bid_liquidity,
            total_ask_liquidity,
            bid_ask_ratio: total_bid_liquidity / total_ask_liquidity,
            depth_quality: calculate_depth_quality(&depth),
        })
    }

    pub async fn get_market_impact(&self, address: &str, size_usd: f64) -> Result<MarketImpact> {
        let depth = self.get_market_depth(address).await?;
        let token_info = self.get_token_info(address).await?;

        let size_tokens = size_usd / token_info.price_usd;
        let (price_impact, executed_price) = calculate_price_impact(&depth, size_tokens, token_info.price_usd);

        Ok(MarketImpact {
            price_impact,
            executed_price,
            size_usd,
            size_tokens,
        })
    }
}

#[derive(Debug)]
pub struct LiquidityAnalysis {
    pub total_bid_liquidity: f64,
    pub total_ask_liquidity: f64,
    pub bid_ask_ratio: f64,
    pub depth_quality: f64,
}

#[derive(Debug)]
pub struct MarketImpact {
    pub price_impact: f64,
    pub executed_price: f64,
    pub size_usd: f64,
    pub size_tokens: f64,
}

fn calculate_depth_quality(depth: &MarketDepth) -> f64 {
    // Implement depth quality calculation
    // This could consider factors like:
    // - Spread
    // - Depth distribution
    // - Number of price levels
    0.0 // Placeholder
}

fn calculate_price_impact(
    depth: &MarketDepth,
    size_tokens: f64,
    current_price: f64,
) -> (f64, f64) {
    // Implement price impact calculation
    // This should walk the order book to determine:
    // - Average execution price
    // - Price impact percentage
    (0.0, current_price) // Placeholder
} 
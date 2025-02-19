use super::BIRDEYE_API_BASE;
use crate::models::token_info::TokenExtensions;
use crate::models::trending_token::{TrendingToken, TrendingTokenData};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenPrice {
    pub value: f64,
    pub decimals: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenData {
    pub address: String,
    pub symbol: String,
    pub name: String,
    #[serde(rename = "logoURI")]
    pub image: Option<String>,
    pub decimals: u8,
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
    pub fdv: Option<f64>,
    pub liquidity: Option<f64>,
    pub price: f64,
    #[serde(rename = "priceChange24hPercent")]
    pub price_change_24h: Option<f64>,
    #[serde(rename = "v24h")]
    pub volume_24h: Option<f64>,
    #[serde(rename = "v24hChangePercent")]
    pub volume_change_24h: Option<f64>,
    #[serde(rename = "trade24h")]
    pub trade_24h: Option<i64>,
    pub holder: Option<i64>,
    pub extensions: Option<TokenExtensions>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiTokenData {
    #[serde(flatten)]
    pub tokens: HashMap<String, TokenData>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenMarketResponse {
    pub address: String,
    pub decimals: i32,
    pub symbol: String,
    pub name: String,
    #[serde(rename = "marketCap")]
    pub market_cap: f64,
    pub fdv: f64,
    pub extensions: TokenExtensions,
    #[serde(rename = "logoURI")]
    pub logo_uri: String,
    pub liquidity: f64,
    #[serde(rename = "lastTradeUnixTime")]
    pub last_trade_unix_time: i64,
    #[serde(rename = "lastTradeHumanTime")]
    pub last_trade_human_time: String,
    pub price: f64,
    #[serde(rename = "history30mPrice")]
    pub history30m_price: f64,
    #[serde(rename = "priceChange30mPercent")]
    pub price_change_30m_percent: f64,
    #[serde(rename = "history1hPrice")]
    pub history1h_price: f64,
    #[serde(rename = "priceChange1hPercent")]
    pub price_change_1h_percent: f64,
    #[serde(rename = "history2hPrice")]
    pub history2h_price: f64,
    #[serde(rename = "priceChange2hPercent")]
    pub price_change_2h_percent: f64,
    #[serde(rename = "history4hPrice")]
    pub history4h_price: f64,
    #[serde(rename = "priceChange4hPercent")]
    pub price_change_4h_percent: f64,
    #[serde(rename = "history6hPrice")]
    pub history6h_price: f64,
    #[serde(rename = "priceChange6hPercent")]
    pub price_change_6h_percent: f64,
    #[serde(rename = "history8hPrice")]
    pub history8h_price: f64,
    #[serde(rename = "priceChange8hPercent")]
    pub price_change_8h_percent: f64,
    #[serde(rename = "history12hPrice")]
    pub history12h_price: f64,
    #[serde(rename = "priceChange12hPercent")]
    pub price_change_12h_percent: f64,
    #[serde(rename = "history24hPrice")]
    pub history24h_price: f64,
    #[serde(rename = "priceChange24hPercent")]
    pub price_change_24h_percent: f64,
    #[serde(rename = "uniqueWallet30m")]
    pub unique_wallet30m: i64,
    #[serde(rename = "uniqueWalletHistory30m")]
    pub unique_wallet_history30m: i64,
    #[serde(rename = "uniqueWallet30mChangePercent")]
    pub unique_wallet30m_change_percent: f64,
    #[serde(rename = "uniqueWallet1h")]
    pub unique_wallet1h: i64,
    #[serde(rename = "uniqueWalletHistory1h")]
    pub unique_wallet_history1h: i64,
    #[serde(rename = "uniqueWallet1hChangePercent")]
    pub unique_wallet1h_change_percent: f64,
    #[serde(rename = "uniqueWallet2h")]
    pub unique_wallet2h: i64,
    #[serde(rename = "uniqueWalletHistory2h")]
    pub unique_wallet_history2h: i64,
    #[serde(rename = "uniqueWallet2hChangePercent")]
    pub unique_wallet2h_change_percent: f64,
    #[serde(rename = "uniqueWallet4h")]
    pub unique_wallet4h: i64,
    #[serde(rename = "uniqueWalletHistory4h")]
    pub unique_wallet_history4h: i64,
    #[serde(rename = "uniqueWallet4hChangePercent")]
    pub unique_wallet4h_change_percent: f64,
    #[serde(rename = "uniqueWallet8h")]
    pub unique_wallet8h: i64,
    #[serde(rename = "uniqueWalletHistory8h")]
    pub unique_wallet_history8h: i64,
    #[serde(rename = "uniqueWallet8hChangePercent")]
    pub unique_wallet8h_change_percent: f64,
    #[serde(rename = "uniqueWallet24h")]
    pub unique_wallet24h: i64,
    #[serde(rename = "uniqueWalletHistory24h")]
    pub unique_wallet_history24h: i64,
    #[serde(rename = "uniqueWallet24hChangePercent")]
    pub unique_wallet24h_change_percent: f64,
    pub supply: f64,
    #[serde(rename = "totalSupply")]
    pub total_supply: f64,
    pub mc: f64,
    #[serde(rename = "circulatingSupply")]
    pub circulating_supply: f64,
    #[serde(rename = "realMc")]
    pub real_mc: f64,
    pub holder: i64,
    pub trade30m: i64,
    #[serde(rename = "tradeHistory30m")]
    pub trade_history30m: i64,
    #[serde(rename = "trade30mChangePercent")]
    pub trade30m_change_percent: f64,
    pub sell30m: i64,
    #[serde(rename = "sellHistory30m")]
    pub sell_history30m: i64,
    #[serde(rename = "sell30mChangePercent")]
    pub sell30m_change_percent: f64,
    pub buy30m: i64,
    #[serde(rename = "buyHistory30m")]
    pub buy_history30m: i64,
    #[serde(rename = "buy30mChangePercent")]
    pub buy30m_change_percent: f64,
    pub v30m: f64,
    #[serde(rename = "v30mUSD")]
    pub v30m_usd: f64,
    #[serde(rename = "vHistory30m")]
    pub v_history30m: f64,
    #[serde(rename = "vHistory30mUSD")]
    pub v_history30m_usd: f64,
    #[serde(rename = "v30mChangePercent")]
    pub v30m_change_percent: f64,
    #[serde(rename = "vBuy30m")]
    pub v_buy30m: f64,
    #[serde(rename = "vBuy30mUSD")]
    pub v_buy30m_usd: f64,
    #[serde(rename = "vBuyHistory30m")]
    pub v_buy_history30m: f64,
    #[serde(rename = "vBuyHistory30mUSD")]
    pub v_buy_history30m_usd: f64,
    #[serde(rename = "vBuy30mChangePercent")]
    pub v_buy30m_change_percent: f64,
    #[serde(rename = "vSell30m")]
    pub v_sell30m: f64,
    #[serde(rename = "vSell30mUSD")]
    pub v_sell30m_usd: f64,
    #[serde(rename = "vSellHistory30m")]
    pub v_sell_history30m: f64,
    #[serde(rename = "vSellHistory30mUSD")]
    pub v_sell_history30m_usd: f64,
    #[serde(rename = "vSell30mChangePercent")]
    pub v_sell30m_change_percent: f64,
    pub trade24h: i64,
    #[serde(rename = "tradeHistory24h")]
    pub trade_history24h: i64,
    #[serde(rename = "trade24hChangePercent")]
    pub trade24h_change_percent: f64,
    pub sell24h: i64,
    #[serde(rename = "sellHistory24h")]
    pub sell_history24h: i64,
    #[serde(rename = "sell24hChangePercent")]
    pub sell24h_change_percent: f64,
    pub buy24h: i64,
    #[serde(rename = "buyHistory24h")]
    pub buy_history24h: i64,
    #[serde(rename = "buy24hChangePercent")]
    pub buy24h_change_percent: f64,
    pub v24h: f64,
    #[serde(rename = "v24hUSD")]
    pub v24h_usd: f64,
    #[serde(rename = "vHistory24h")]
    pub v_history24h: f64,
    #[serde(rename = "vHistory24hUSD")]
    pub v_history24h_usd: f64,
    #[serde(rename = "v24hChangePercent")]
    pub v24h_change_percent: f64,
    #[serde(rename = "vBuy24h")]
    pub v_buy24h: f64,
    #[serde(rename = "vBuy24hUSD")]
    pub v_buy24h_usd: f64,
    #[serde(rename = "vBuyHistory24h")]
    pub v_buy_history24h: f64,
    #[serde(rename = "vBuyHistory24hUSD")]
    pub v_buy_history24h_usd: f64,
    #[serde(rename = "vBuy24hChangePercent")]
    pub v_buy24h_change_percent: f64,
    #[serde(rename = "vSell24h")]
    pub v_sell24h: f64,
    #[serde(rename = "vSell24hUSD")]
    pub v_sell24h_usd: f64,
    #[serde(rename = "vSellHistory24h")]
    pub v_sell_history24h: f64,
    #[serde(rename = "vSellHistory24hUSD")]
    pub v_sell_history24h_usd: f64,
    #[serde(rename = "vSell24hChangePercent")]
    pub v_sell24h_change_percent: f64,
    #[serde(rename = "numberMarkets")]
    pub number_markets: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnchainMetrics {
    pub unique_holders: u32,
    pub active_wallets_24h: u32,
    pub transactions_24h: u32,
    pub average_transaction_size: f64,
    pub whale_transactions_24h: u32,
}

#[async_trait]
pub trait BirdeyeApi: Send + Sync {
    /// Get detailed market data for a token by address
    async fn get_market_data(&self, address: &str) -> Result<TokenMarketResponse>;
    
    /// Get on-chain metrics for a token
    async fn get_onchain_metrics(&self, address: &str) -> Result<OnchainMetrics>;
    
    /// Get trending tokens data
    async fn get_trending_tokens(&self) -> Result<Vec<TrendingToken>>;
}

pub struct BirdeyeClient {
    client: Client,
    api_key: String,
}

impl BirdeyeClient {
    pub fn new(api_key: String) -> Self {
        BirdeyeClient {
            client: Client::new(),
            api_key,
        }
    }

    async fn get(&self, endpoint: &str) -> Result<reqwest::Response> {
        let url = format!("{}{}", BIRDEYE_API_BASE, endpoint);
        let response = self
            .client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(anyhow!(
                "Birdeye API request failed with status {}: {}",
                status,
                text
            ))
        }
    }
}

#[async_trait]
impl BirdeyeApi for BirdeyeClient {
    async fn get_market_data(&self, address: &str) -> Result<TokenMarketResponse> {
        let endpoint = format!("/defi/v3/token/market-data?address={}", address);
        let response: ApiResponse<TokenMarketResponse> = self.get(&endpoint).await?.json().await?;

        if response.success {
            Ok(response.data)
        } else {
            Err(anyhow!(
                "Failed to get market data: {}",
                response.message.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    async fn get_onchain_metrics(&self, address: &str) -> Result<OnchainMetrics> {
        let endpoint = format!("/defi/v3/token/onchain-metrics?address={}", address);
        let response: ApiResponse<OnchainMetrics> = self.get(&endpoint).await?.json().await?;

        if response.success {
            Ok(response.data)
        } else {
            Err(anyhow!(
                "Failed to get onchain metrics: {}",
                response
                    .message
                    .unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    async fn get_trending_tokens(&self) -> Result<Vec<TrendingToken>> {
        let endpoint = "/defi/token_trending?sort_by=rank&sort_type=asc&limit=20";
        let response: ApiResponse<TrendingTokenData> = self.get(endpoint).await?.json().await?;

        if response.success {
            Ok(response.data.tokens)
        } else {
            Err(anyhow!(
                "Failed to get trending tokens: {}",
                response.message.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
}

// Mock BirdeyeApi for testing
#[cfg(test)]
pub struct MockBirdeyeApi {
    pub market_data: Option<TokenMarketResponse>,
    pub onchain_metrics: Option<OnchainMetrics>,
    pub trending_tokens: Option<Vec<TrendingToken>>,
}

#[cfg(test)]
impl MockBirdeyeApi {
    pub fn new() -> Self {
        MockBirdeyeApi {
            market_data: None,
            onchain_metrics: None,
            trending_tokens: None,
        }
    }
}

#[cfg(test)]
#[async_trait]
impl BirdeyeApi for MockBirdeyeApi {
    async fn get_market_data(&self, _address: &str) -> Result<TokenMarketResponse> {
        self.market_data.clone().ok_or(anyhow!("Mock not set"))
    }

    async fn get_onchain_metrics(&self, _address: &str) -> Result<OnchainMetrics> {
        self.onchain_metrics.clone().ok_or(anyhow!("Mock not set"))
    }

    async fn get_trending_tokens(&self) -> Result<Vec<TrendingToken>> {
        self.trending_tokens.clone().ok_or(anyhow!("Mock not set"))
    }
}

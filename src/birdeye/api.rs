use super::BIRDEYE_API_URL;
use crate::models::token_info::TokenExtensions;
use crate::models::token_trending::{TrendingToken, TrendingTokenData};
use crate::models::market_data::{ApiResponse, TokenMarketResponse, TokenMarketData};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OnchainMetrics {
    pub unique_holders: u32,
    pub active_wallets_24h: u32,
    pub transactions_24h: u32,
    pub average_transaction_size: f64,
    pub whale_transactions_24h: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenOverviewResponse {
    pub address: String,
    pub decimals: u8,
    pub symbol: String,
    pub name: String,
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
    pub fdv: Option<f64>,
    pub extensions: Option<TokenExtensions>,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub liquidity: Option<f64>,
    #[serde(rename = "lastTradeUnixTime")]
    pub last_trade_unix_time: Option<i64>,
    #[serde(rename = "lastTradeHumanTime")]
    pub last_trade_human_time: Option<String>,
    pub price: f64,
    #[serde(rename = "priceChange24hPercent")]
    pub price_change_24h: Option<f64>,
    #[serde(rename = "volume24h")]
    pub volume_24h: Option<f64>,
    pub holder: Option<i32>,
    #[serde(rename = "numberMarkets")]
    pub number_markets: Option<i64>,
    pub supply: Option<f64>,
    #[serde(rename = "totalSupply")]
    pub total_supply: Option<f64>,
    #[serde(rename = "circulatingSupply")]
    pub circulating_supply: Option<f64>,
}

#[async_trait]
pub trait BirdeyeApi: Send + Sync {
    /// Get detailed market data for a token by address
    async fn get_market_data(&self, address: &str) -> Result<TokenMarketResponse>;
    
    /// Get basic token overview information
    async fn get_token_overview(&self, address: &str) -> Result<TokenOverviewResponse>;
    
    /// Get trending tokens data
    async fn get_token_trending(&self) -> Result<Vec<TrendingToken>>;
}

pub struct BirdeyeClient {
    client: Client,
    api_key: String,
}

impl BirdeyeClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to create reqwest client");
        BirdeyeClient { client, api_key }
    }

    async fn get(&self, endpoint: &str) -> Result<reqwest::Response> {
        let url = format!("{}{}", BIRDEYE_API_URL, endpoint);
        debug!("Making GET request to: {}", url);
        
        let response = self
            .client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await
            .context(format!("Failed to send GET request to {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "No error text".to_string());
            error!("HTTP error {}: {}", status, error_text);
            return Err(anyhow!("HTTP {} error: {}", status, error_text));
        }

        debug!("Received successful response from {}", url);
        Ok(response)
    }
}

#[async_trait]
impl BirdeyeApi for BirdeyeClient {
    async fn get_market_data(&self, address: &str) -> Result<TokenMarketResponse> {
        debug!("Fetching market data for address: {}", address);
        let endpoint = format!("/defi/v3/token/market-data?address={}", address);
        let response: ApiResponse<TokenMarketResponse> = self
            .get(&endpoint)
            .await?
            .json()
            .await
            .context("Failed to deserialize market data response")?;

        if response.success {
            debug!("Successfully retrieved market data for {}", address);
            Ok(response.data)
        } else {
            let error_msg = response.message.unwrap_or_else(|| "Unknown error".to_string());
            error!("Failed to get market data: {}", error_msg);
            Err(anyhow!("Failed to get market data: {}", error_msg))
        }
    }

    async fn get_token_overview(&self, address: &str) -> Result<TokenOverviewResponse> {
        debug!("Fetching token overview for address: {}", address);
        let endpoint = format!("/defi/token_overview?address={}", address);
        let response: ApiResponse<TokenOverviewResponse> = self
            .get(&endpoint)
            .await?
            .json()
            .await
            .context("Failed to deserialize token overview response")?;

        if response.success {
            debug!("Successfully retrieved token overview for {}", address);
            Ok(response.data)
        } else {
            let error_msg = response.message.unwrap_or_else(|| "Unknown error".to_string());
            error!("Failed to get token overview: {}", error_msg);
            Err(anyhow!("Failed to get token overview: {}", error_msg))
        }
    }

    async fn get_token_trending(&self) -> Result<Vec<TrendingToken>> {
        debug!("Fetching trending tokens");
        let endpoint = "/defi/token_trending?sort_by=rank&sort_type=asc&limit=20";
        let response: ApiResponse<TrendingTokenData> = self
            .get(&endpoint)
            .await?
            .json()
            .await
            .context("Failed to deserialize trending tokens response")?;

        if response.success {
            debug!("Successfully retrieved {} trending tokens", response.data.tokens.len());
            Ok(response.data.tokens)
        } else {
            let error_msg = response.message.unwrap_or_else(|| "Unknown error".to_string());
            error!("Failed to get trending tokens: {}", error_msg);
            Err(anyhow!("Failed to get trending tokens: {}", error_msg))
        }
    }
}

// Mock BirdeyeApi for testing
#[cfg(test)]
pub struct MockBirdeyeApi {
    pub market_data: Option<TokenMarketResponse>,
    pub token_overview: Option<TokenOverviewResponse>,
    pub token_trending: Option<Vec<TrendingToken>>,
}

#[cfg(test)]
impl MockBirdeyeApi {
    pub fn new() -> Self {
        MockBirdeyeApi {
            market_data: None,
            token_overview: None,
            token_trending: None,
        }
    }
}

#[cfg(test)]
#[async_trait]
impl BirdeyeApi for MockBirdeyeApi {
    async fn get_market_data(&self, _address: &str) -> Result<TokenMarketResponse> {
        self.market_data.clone().ok_or(anyhow!("Mock not set"))
    }

    async fn get_token_overview(&self, _address: &str) -> Result<TokenOverviewResponse> {
        self.token_overview.clone().ok_or(anyhow!("Mock not set"))
    }

    async fn get_token_trending(&self) -> Result<Vec<TrendingToken>> {
        self.token_trending.clone().ok_or(anyhow!("Mock not set"))
    }
}

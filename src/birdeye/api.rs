use super::{BIRDEYE_API_BASE};
use crate::models::token_info::{TokenInfo, TokenExtensions};
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
    pub fdv: f64,
    #[serde(rename = "marketCap")]
    pub market_cap: f64,
    pub supply: f64,
    #[serde(rename = "totalSupply")]
    pub total_supply: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TrendingToken {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnchainMetrics {
    pub unique_holders: u64,
    pub active_wallets_24h: u64,
    pub whale_transactions_24h: u64,
}

#[async_trait]
pub trait BirdeyeApi: Send + Sync {
    async fn get_token_info(&self, symbol: &str) -> Result<TokenInfo>;
    async fn get_token_info_by_address(&self, address: &str) -> Result<TokenInfo>;
    async fn get_market_data(&self, address: &str) -> Result<TokenMarketResponse>;
    async fn get_trending_tokens(&self, limit: usize) -> Result<Vec<TrendingToken>>;
    async fn get_onchain_metrics(&self, address: &str) -> Result<OnchainMetrics>;
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
    async fn get_token_info(&self, symbol: &str) -> Result<TokenInfo> {
        let endpoint = format!("/defi/v3/token/meta-data/multiple?list_address={}", symbol);
        let response: ApiResponse<TokenData> = self.get(&endpoint).await?.json().await?;

        if response.success {
            let token_data = response.data;
            Ok(TokenInfo {
                address: token_data.address,
                symbol: token_data.symbol,
                name: token_data.name,
                decimals: token_data.decimals,
                price: token_data.price,
                volume_24h: token_data.volume_24h.unwrap_or(0.0),
                market_cap: token_data.market_cap,
                price_change_24h: Some(token_data.price_change_24h.unwrap_or(0.0)),
                volume_change_24h: Some(token_data.volume_change_24h.unwrap_or(0.0)),
                liquidity: token_data.liquidity.unwrap_or(0.0),
                trade_24h: Some(token_data.trade_24h.unwrap_or(0)),
                logo_uri: token_data.image,
                extensions: None,
                timestamp: bson::DateTime::now(),
            })
        } else {
            Err(anyhow!(
                "Failed to get token info: {}",
                response.message.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    async fn get_token_info_by_address(&self, address: &str) -> Result<TokenInfo> {
        let endpoint = format!("/defi/token_overview?address={}", address);
        let response: ApiResponse<TokenData> = self.get(&endpoint).await?.json().await?;

        if response.success {
            let token_data = response.data;
            Ok(TokenInfo {
                address: token_data.address,
                symbol: token_data.symbol,
                name: token_data.name,
                decimals: token_data.decimals,
                price: token_data.price,
                volume_24h: token_data.volume_24h.unwrap_or(0.0),
                market_cap: token_data.market_cap,
                price_change_24h: Some(token_data.price_change_24h.unwrap_or(0.0)),
                volume_change_24h: Some(token_data.volume_change_24h.unwrap_or(0.0)),
                liquidity: token_data.liquidity.unwrap_or(0.0),
                trade_24h: Some(token_data.trade_24h.unwrap_or(0)),
                logo_uri: token_data.image,
                extensions: token_data.extensions,
                timestamp: bson::DateTime::now(),
            })
        } else {
            Err(anyhow!(
                "Failed to get token info by address: {}",
                response.message.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    async fn get_market_data(&self, address: &str) -> Result<TokenMarketResponse> {
        let endpoint = format!("/defi/token_overview?address={}", address);
        let response: ApiResponse<TokenData> = self.get(&endpoint).await?.json().await?;

        if response.success {
            let data = response.data;
            Ok(TokenMarketResponse {
                fdv: data.fdv.unwrap_or(0.0),
                market_cap: data.market_cap.unwrap_or(0.0),
                supply: data.fdv.unwrap_or(0.0) / data.price,
                total_supply: data.fdv.unwrap_or(0.0) / data.price,
            })
        } else {
            Err(anyhow!(
                "Failed to get market data: {}",
                response.message.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    async fn get_trending_tokens(&self, limit: usize) -> Result<Vec<TrendingToken>> {
        let endpoint = format!("/defi/v3/search?sort_by=volume_24h_usd&sort_type=desc&limit={}&verify_token=true", limit);
        let response: ApiResponse<Vec<TrendingToken>> = self.get(&endpoint).await?.json().await?;

        if response.success {
            Ok(response.data)
        } else {
            Err(anyhow!(
                "Failed to get trending tokens: {}",
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
                response.message.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
}

// Mock BirdeyeApi for testing
#[cfg(test)]
pub struct MockBirdeyeApi {
    pub token_info: Option<TokenInfo>,
    pub token_info_by_address: Option<TokenInfo>,
    pub market_data: Option<TokenMarketResponse>,
    pub trending_tokens: Option<Vec<TrendingToken>>,
}

#[cfg(test)]
impl MockBirdeyeApi {
    pub fn new() -> Self {
        MockBirdeyeApi {
            token_info: None,
            token_info_by_address: None,
            market_data: None,
            trending_tokens: None,
        }
    }

    pub fn expect_get_token_info_by_address(&mut self) -> &mut Self {
        self.token_info_by_address = Some(TokenInfo {
            address: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            decimals: 9,
            price: 100.0,
            volume_24h: 1000000.0,
            market_cap: Some(1000000000.0),
            price_change_24h: Some(5.0),
            volume_change_24h: Some(10.0),
            liquidity: 500000.0,
            trade_24h: Some(1000),
            logo_uri: Some("https://example.com/sol.png".to_string()),
            extensions: None,
            timestamp: bson::DateTime::now(),
        });
        self
    }
}

#[cfg(test)]
#[async_trait]
impl BirdeyeApi for MockBirdeyeApi {
    async fn get_token_info(&self, _symbol: &str) -> Result<TokenInfo> {
        self.token_info.clone().ok_or(anyhow!("Mock not set"))
    }

    async fn get_token_info_by_address(&self, _address: &str) -> Result<TokenInfo> {
        self.token_info_by_address.clone().ok_or(anyhow!("Mock not set"))
    }

    async fn get_market_data(&self, _address: &str) -> Result<TokenMarketResponse> {
        self.market_data.clone().ok_or(anyhow!("Mock not set"))
    }

    async fn get_trending_tokens(&self, _limit: usize) -> Result<Vec<TrendingToken>> {
        self.trending_tokens.clone().ok_or(anyhow!("Mock not set"))
    }

    async fn get_onchain_metrics(&self, _address: &str) -> Result<OnchainMetrics> {
        Ok(OnchainMetrics {
            unique_holders: 0,
            active_wallets_24h: 0,
            whale_transactions_24h: 0,
        })
    }
}
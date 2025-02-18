use super::{BIRDEYE_API_BASE};
use crate::models::token_info::TokenInfo;
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
    pub image: Option<String>,
    pub price: f64,
    pub decimals: u8,
    #[serde(rename = "price24h")]
    pub price_24h: Option<f64>,
    #[serde(rename = "priceChange24h")]
    pub price_change_24h: Option<f64>,
    #[serde(rename = "volume24h")]
    pub volume_24h: Option<f64>,
    #[serde(rename = "volumeChange24h")]
    pub volume_change_24h: Option<f64>,
    pub liquidity: Option<f64>,
    #[serde(rename = "txns24h")]
    pub trade_24h: Option<i64>,
    pub holders: Option<i64>,
    #[serde(rename = "fdv")]
    pub fully_diluted_value: Option<f64>,
    #[serde(rename = "mcap")]
    pub market_cap: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiTokenData {
    #[serde(flatten)]
    pub tokens: HashMap<String, TokenData>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenMarketResponse {
    pub value: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TrendingToken {
    pub name: String,
    pub value: f64,
}

#[async_trait]
pub trait BirdeyeApi: Send + Sync {
    async fn get_token_info(&self, symbol: &str) -> Result<TokenInfo>;
    async fn get_token_info_by_address(&self, address: &str) -> Result<TokenInfo>;
    async fn get_market_data(&self, address: &str) -> Result<TokenMarketResponse>;
    async fn get_trending_tokens(&self, limit: usize) -> Result<Vec<TrendingToken>>;
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
        let endpoint = format!("/public/token_info/{}", symbol);
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
        let endpoint = format!("/public/token_info/{}", address);
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
                "Failed to get token info by address: {}",
                response.message.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    async fn get_market_data(&self, address: &str) -> Result<TokenMarketResponse> {
        let endpoint = format!("/public/token_price/{}", address);
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

    async fn get_trending_tokens(&self, limit: usize) -> Result<Vec<TrendingToken>> {
        let endpoint = format!("/public/trending_tokens?limit={}", limit);
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
}
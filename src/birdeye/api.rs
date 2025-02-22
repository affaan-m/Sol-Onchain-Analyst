use super::BIRDEYE_API_URL;
use crate::models::market_data::{ApiResponse, TokenMarketResponse};
use crate::models::token_info::TokenExtensions;
use crate::models::token_trending::{TrendingToken, TrendingTokenData};
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
    pub market_cap: f64,
    pub fdv: f64,
    pub extensions: Option<TokenExtensions>,
    #[serde(rename = "logoURI")]
    pub logo_uri: String,
    pub liquidity: f64,
    #[serde(rename = "lastTradeUnixTime")]
    pub last_trade_unix_time: i64,
    #[serde(rename = "lastTradeHumanTime")]
    pub last_trade_human_time: String,
    pub price: f64,
    #[serde(rename = "history30mPrice")]
    pub history_30m_price: f64,
    #[serde(rename = "priceChange30mPercent")]
    pub price_change_30m_percent: f64,
    #[serde(rename = "history1hPrice")]
    pub history_1h_price: f64,
    #[serde(rename = "priceChange1hPercent")]
    pub price_change_1h_percent: f64,
    #[serde(rename = "history2hPrice")]
    pub history_2h_price: f64,
    #[serde(rename = "priceChange2hPercent")]
    pub price_change_2h_percent: f64,
    #[serde(rename = "history4hPrice")]
    pub history_4h_price: f64,
    #[serde(rename = "priceChange4hPercent")]
    pub price_change_4h_percent: f64,
    #[serde(rename = "history6hPrice")]
    pub history_6h_price: f64,
    #[serde(rename = "priceChange6hPercent")]
    pub price_change_6h_percent: f64,
    #[serde(rename = "history8hPrice")]
    pub history_8h_price: f64,
    #[serde(rename = "priceChange8hPercent")]
    pub price_change_8h_percent: f64,
    #[serde(rename = "history12hPrice")]
    pub history_12h_price: f64,
    #[serde(rename = "priceChange12hPercent")]
    pub price_change_12h_percent: f64,
    #[serde(rename = "history24hPrice")]
    pub history_24h_price: f64,
    #[serde(rename = "priceChange24hPercent")]
    pub price_change_24h_percent: f64,
    #[serde(rename = "uniqueWallet30m")]
    pub unique_wallet_30m: i64,
    #[serde(rename = "uniqueWalletHistory30m")]
    pub unique_wallet_history_30m: i64,
    #[serde(rename = "uniqueWallet30mChangePercent")]
    pub unique_wallet_30m_change_percent: f64,
    #[serde(rename = "uniqueWallet1h")]
    pub unique_wallet_1h: i64,
    #[serde(rename = "uniqueWalletHistory1h")]
    pub unique_wallet_history_1h: i64,
    #[serde(rename = "uniqueWallet1hChangePercent")]
    pub unique_wallet_1h_change_percent: f64,
    #[serde(rename = "uniqueWallet2h")]
    pub unique_wallet_2h: i64,
    #[serde(rename = "uniqueWalletHistory2h")]
    pub unique_wallet_history_2h: i64,
    #[serde(rename = "uniqueWallet2hChangePercent")]
    pub unique_wallet_2h_change_percent: f64,
    #[serde(rename = "uniqueWallet4h")]
    pub unique_wallet_4h: i64,
    #[serde(rename = "uniqueWalletHistory4h")]
    pub unique_wallet_history_4h: i64,
    #[serde(rename = "uniqueWallet4hChangePercent")]
    pub unique_wallet_4h_change_percent: f64,
    #[serde(rename = "uniqueWallet8h")]
    pub unique_wallet_8h: i64,
    #[serde(rename = "uniqueWalletHistory8h")]
    pub unique_wallet_history_8h: i64,
    #[serde(rename = "uniqueWallet8hChangePercent")]
    pub unique_wallet_8h_change_percent: f64,
    #[serde(rename = "uniqueWallet24h")]
    pub unique_wallet_24h: i64,
    #[serde(rename = "uniqueWalletHistory24h")]
    pub unique_wallet_history_24h: i64,
    #[serde(rename = "uniqueWallet24hChangePercent")]
    pub unique_wallet_24h_change_percent: f64,
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
    pub trade_history_30m: i64,
    #[serde(rename = "trade30mChangePercent")]
    pub trade_30m_change_percent: f64,
    pub sell30m: i64,
    #[serde(rename = "sellHistory30m")]
    pub sell_history_30m: i64,
    #[serde(rename = "sell30mChangePercent")]
    pub sell_30m_change_percent: f64,
    pub buy30m: i64,
    #[serde(rename = "buyHistory30m")]
    pub buy_history_30m: i64,
    #[serde(rename = "buy30mChangePercent")]
    pub buy_30m_change_percent: f64,
    pub v30m: f64,
    #[serde(rename = "v30mUSD")]
    pub v30m_usd: f64,
    #[serde(rename = "vHistory30m")]
    pub v_history_30m: f64,
    #[serde(rename = "vHistory30mUSD")]
    pub v_history_30m_usd: f64,
    #[serde(rename = "v30mChangePercent")]
    pub v30m_change_percent: f64,
    #[serde(rename = "vBuy30m")]
    pub v_buy_30m: f64,
    #[serde(rename = "vBuy30mUSD")]
    pub v_buy_30m_usd: f64,
    #[serde(rename = "vBuyHistory30m")]
    pub v_buy_history_30m: f64,
    #[serde(rename = "vBuyHistory30mUSD")]
    pub v_buy_history_30m_usd: f64,
    #[serde(rename = "vBuy30mChangePercent")]
    pub v_buy_30m_change_percent: f64,
    #[serde(rename = "vSell30m")]
    pub v_sell_30m: f64,
    #[serde(rename = "vSell30mUSD")]
    pub v_sell_30m_usd: f64,
    #[serde(rename = "vSellHistory30m")]
    pub v_sell_history_30m: f64,
    #[serde(rename = "vSellHistory30mUSD")]
    pub v_sell_history_30m_usd: f64,
    #[serde(rename = "vSell30mChangePercent")]
    pub v_sell_30m_change_percent: f64,
    pub trade24h: i64,
    #[serde(rename = "tradeHistory24h")]
    pub trade_history_24h: i64,
    #[serde(rename = "trade24hChangePercent")]
    pub trade_24h_change_percent: f64,
    pub sell24h: i64,
    #[serde(rename = "sellHistory24h")]
    pub sell_history_24h: i64,
    #[serde(rename = "sell24hChangePercent")]
    pub sell_24h_change_percent: f64,
    pub buy24h: i64,
    #[serde(rename = "buyHistory24h")]
    pub buy_history_24h: i64,
    #[serde(rename = "buy24hChangePercent")]
    pub buy_24h_change_percent: f64,
    pub v24h: f64,
    #[serde(rename = "v24hUSD")]
    pub v24h_usd: f64,
    #[serde(rename = "vHistory24h")]
    pub v_history_24h: f64,
    #[serde(rename = "vHistory24hUSD")]
    pub v_history_24h_usd: f64,
    #[serde(rename = "v24hChangePercent")]
    pub v24h_change_percent: f64,
    #[serde(rename = "vBuy24h")]
    pub v_buy_24h: f64,
    #[serde(rename = "vBuy24hUSD")]
    pub v_buy_24h_usd: f64,
    #[serde(rename = "vBuyHistory24h")]
    pub v_buy_history_24h: f64,
    #[serde(rename = "vBuyHistory24hUSD")]
    pub v_buy_history_24h_usd: f64,
    #[serde(rename = "vBuy24hChangePercent")]
    pub v_buy_24h_change_percent: f64,
    #[serde(rename = "vSell24h")]
    pub v_sell_24h: f64,
    #[serde(rename = "vSell24hUSD")]
    pub v_sell_24h_usd: f64,
    #[serde(rename = "vSellHistory24h")]
    pub v_sell_history_24h: f64,
    #[serde(rename = "vSellHistory24hUSD")]
    pub v_sell_history_24h_usd: f64,
    #[serde(rename = "vSell24hChangePercent")]
    pub v_sell_24h_change_percent: f64,
    #[serde(rename = "numberMarkets")]
    pub number_markets: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenV3Response {
    pub address: String,
    pub symbol: String,
    pub name: String,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
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
    pub social_metrics: Option<SocialMetrics>,
    pub dev_metrics: Option<DevMetrics>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SocialMetrics {
    pub twitter_followers: Option<i64>,
    pub twitter_handle: Option<String>,
    pub discord_members: Option<i64>,
    pub telegram_members: Option<i64>,
    pub comments_disabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DevMetrics {
    pub github_stars: Option<i64>,
    pub github_forks: Option<i64>,
    pub github_contributors: Option<i64>,
    pub last_commit_date: Option<String>,
    pub dev_wallet_count: Option<i64>,
    pub dev_activity_30d: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenV3ListResponse {
    pub data: Vec<TokenV3Response>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

#[async_trait]
pub trait BirdeyeApi: Send + Sync {
    /// Get detailed market data for a token by address
    async fn get_market_data(&self, address: &str) -> Result<TokenMarketResponse>;

    /// Get basic token overview information
    async fn get_token_overview(&self, address: &str) -> Result<TokenOverviewResponse>;

    /// Get trending tokens data
    async fn get_token_trending(&self) -> Result<Vec<TrendingToken>>;

    /// Get token list with v3 endpoint
    async fn get_token_list_v3(&self, page: i64, limit: i64) -> Result<TokenV3ListResponse>;

    /// Get token metadata with v3 endpoint
    async fn get_token_metadata_v3(&self, address: &str) -> Result<TokenV3Response>;
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
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "No error text".to_string());
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
            let error_msg = response
                .message
                .unwrap_or_else(|| "Unknown error".to_string());
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
            if response.data.address != address {
                error!(
                    "Token address mismatch: requested {}, but got {}",
                    address, response.data.address
                );
                return Err(anyhow!("Token not found: {}", address));
            }
            debug!("Successfully retrieved token overview for {}", address);
            Ok(response.data)
        } else {
            let error_msg = response
                .message
                .unwrap_or_else(|| "Unknown error".to_string());
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
            debug!(
                "Successfully retrieved {} trending tokens",
                response.data.tokens.len()
            );
            Ok(response.data.tokens)
        } else {
            let error_msg = response
                .message
                .unwrap_or_else(|| "Unknown error".to_string());
            error!("Failed to get trending tokens: {}", error_msg);
            Err(anyhow!("Failed to get trending tokens: {}", error_msg))
        }
    }

    async fn get_token_list_v3(&self, page: i64, limit: i64) -> Result<TokenV3ListResponse> {
        let endpoint = format!("/v3/tokens?page={}&limit={}", page, limit);
        let response = self.get(&endpoint).await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get token list: {}", error_text));
        }

        let token_list = response.json::<TokenV3ListResponse>().await
            .context("Failed to parse token list response")?;

        Ok(token_list)
    }

    async fn get_token_metadata_v3(&self, address: &str) -> Result<TokenV3Response> {
        let endpoint = format!("/v3/token/{}/metadata", address);
        let response = self.get(&endpoint).await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get token metadata: {}", error_text));
        }

        let token_metadata = response.json::<TokenV3Response>().await
            .context("Failed to parse token metadata response")?;

        Ok(token_metadata)
    }
}

// Mock BirdeyeApi for testing
#[cfg(test)]
pub struct MockBirdeyeApi {
    pub market_data: Option<TokenMarketResponse>,
    pub token_overview: Option<TokenOverviewResponse>,
    pub token_trending: Option<Vec<TrendingToken>>,
    pub token_list_v3: Option<TokenV3ListResponse>,
    pub token_metadata_v3: Option<TokenV3Response>,
}

#[cfg(test)]
impl MockBirdeyeApi {
    pub fn new() -> Self {
        MockBirdeyeApi {
            market_data: None,
            token_overview: None,
            token_trending: None,
            token_list_v3: None,
            token_metadata_v3: None,
        }
    }

    pub fn with_token_list_v3(mut self, token_list: TokenV3ListResponse) -> Self {
        self.token_list_v3 = Some(token_list);
        self
    }

    pub fn with_token_metadata_v3(mut self, token_metadata: TokenV3Response) -> Self {
        self.token_metadata_v3 = Some(token_metadata);
        self
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

    async fn get_token_list_v3(&self, _page: i64, _limit: i64) -> Result<TokenV3ListResponse> {
        self.token_list_v3.clone().ok_or_else(|| anyhow!("No mock token list data"))
    }

    async fn get_token_metadata_v3(&self, _address: &str) -> Result<TokenV3Response> {
        self.token_metadata_v3.clone().ok_or_else(|| anyhow!("No mock token metadata"))
    }
}

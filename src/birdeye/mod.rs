use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use async_trait::async_trait;
use tokio::time::{sleep, Duration};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

const BIRDEYE_API_BASE: &str = "https://public-api.birdeye.so";
const RATE_LIMIT_DELAY: u64 = 500; // 500ms between requests

// Common token addresses
const TOKEN_ADDRESSES: &[(&str, &str)] = &[
    ("SOL", "So11111111111111111111111111111111111111112"),
    ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    ("BONK", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub price: f64,
    pub volume24h: f64,
    pub price_change_24h: f64,
    pub liquidity: f64,
    pub trade24h: i64,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BirdeyeApi: Send + Sync {
    async fn get_token_info(&self, symbol: &str) -> Result<TokenInfo>;
    async fn get_token_info_by_address(&self, address: &str) -> Result<TokenInfo>;
}

pub struct BirdeyeClient {
    client: Client,
    api_key: String,
    last_request: Arc<AtomicU64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenMarketResponse {
    success: bool,
    data: TokenMarketData,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenMarketData {
    address: String,
    price: f64,
    volume_24h: f64,
    decimals: u8,
    price_sol: f64,
    market_cap: f64,
    fully_diluted_market_cap: Option<f64>,
    circulating_supply: Option<f64>,
    total_supply: Option<f64>,
    price_change_24h: Option<f64>,
    volume_change_24h: Option<f64>,
}

impl Default for TokenMarketData {
    fn default() -> Self {
        Self {
            address: String::new(),
            price: 0.0,
            volume_24h: 0.0,
            decimals: 0,
            price_sol: 0.0,
            market_cap: 0.0,
            fully_diluted_market_cap: None,
            circulating_supply: None,
            total_supply: None,
            price_change_24h: None,
            volume_change_24h: None,
        }
    }
}

impl BirdeyeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_key,
            last_request: Arc::new(AtomicU64::new(0)),
        }
    }

    fn get_token_address(symbol: &str) -> Result<&'static str> {
        let symbol = symbol.trim_start_matches('$').to_uppercase();
        TOKEN_ADDRESSES
            .iter()
            .find(|(s, _)| *s == symbol)
            .map(|(_, addr)| *addr)
            .ok_or_else(|| anyhow!("Unknown token symbol: {}", symbol))
    }

    async fn rate_limit(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let last = self.last_request.load(Ordering::SeqCst);
        let elapsed = now.saturating_sub(last);
        
        if elapsed < RATE_LIMIT_DELAY {
            sleep(Duration::from_millis(RATE_LIMIT_DELAY - elapsed)).await;
        }
        
        self.last_request.store(now, Ordering::SeqCst);
    }

    async fn get_market_data(&self, address: &str) -> Result<TokenMarketResponse> {
        self.rate_limit().await;
        
        let url = format!("{}/v2/tokens/token_data?address={}", BIRDEYE_API_BASE, address);
        tracing::info!("Requesting Birdeye data: {}", url);
        
        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?;
            
        let status = response.status();
        let text = response.text().await?;
        
        tracing::info!("Birdeye response status: {}", status);
        tracing::info!("Birdeye response body: {}", text);
        
        if !status.is_success() {
            if let Ok(error_response) = serde_json::from_str::<TokenMarketResponse>(&text) {
                return Err(anyhow!("Token not found: {}", error_response.data.address));
            }
            return Err(anyhow!("HTTP error {}: {}", status, text));
        }
            
        serde_json::from_str(&text)
            .map_err(|e| anyhow!("Failed to parse market data: {}\nResponse: {}", e, text))
    }
}

#[async_trait]
impl BirdeyeApi for BirdeyeClient {
    async fn get_token_info(&self, symbol: &str) -> Result<TokenInfo> {
        let address = Self::get_token_address(symbol)?;
        self.get_token_info_by_address(address).await
    }

    async fn get_token_info_by_address(&self, address: &str) -> Result<TokenInfo> {
        let response = self.get_market_data(address).await?;
        
        if !response.success {
            return Err(anyhow!("Birdeye API error for token {}", address));
        }

        let data = response.data;
        Ok(TokenInfo {
            price: data.price,
            volume24h: data.volume_24h,
            price_change_24h: data.price_change_24h.unwrap_or_default(),
            liquidity: 0.0, // Not available in this endpoint
            trade24h: 0, // Not available in this endpoint
        })
    }
} 
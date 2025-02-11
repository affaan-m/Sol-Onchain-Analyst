use crate::types::{
    api::{
        LiquidityAnalysis, MarketImpact, PricePoint, TokenInfo, TokenOverview, TokenSearchParams,
        WalletPortfolio,
    },
    error::BirdeyeError,
    TimeInterval,
};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const API_BASE_URL: &str = "https://public-api.birdeye.so";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Deserialize)]
struct TokenSearchResponse {
    data: Vec<TokenInfo>,
}

#[derive(Debug, Clone)]
pub struct BirdeyeProvider {
    client: Client,
    api_key: String,
}

impl BirdeyeProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(DEFAULT_TIMEOUT)
                .build()
                .unwrap_or_default(),
            api_key,
        }
    }

    pub async fn search_tokens(
        &self,
        params: TokenSearchParams,
    ) -> Result<Vec<TokenInfo>, BirdeyeError> {
        let url = format!("{}/search/tokens", API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .query(&[("query", &params.query)])
            .send()
            .await
            .map_err(|e| BirdeyeError::RequestError(e.to_string()))?;

        let data = response
            .json::<TokenSearchResponse>()
            .await
            .map_err(|e| BirdeyeError::SerializationError(e.to_string()))?;

        Ok(data.data)
    }

    pub async fn get_token_overview(&self, address: String) -> Result<TokenOverview, BirdeyeError> {
        let url = format!("{}/token/{}/overview", API_BASE_URL, address);
        let response = self
            .client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await
            .map_err(|e| BirdeyeError::RequestError(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| BirdeyeError::SerializationError(e.to_string()))
    }

    pub async fn analyze_liquidity(
        &self,
        address: &str,
    ) -> Result<LiquidityAnalysis, BirdeyeError> {
        let url = format!("{}/token/{}/liquidity", API_BASE_URL, address);
        let response = self
            .client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await
            .map_err(|e| BirdeyeError::RequestError(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| BirdeyeError::SerializationError(e.to_string()))
    }

    pub async fn get_market_impact(
        &self,
        address: &str,
        size_usd: f64,
    ) -> Result<MarketImpact, BirdeyeError> {
        let url = format!("{}/token/{}/impact", API_BASE_URL, address);
        let response = self
            .client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .query(&[("size_usd", size_usd.to_string())])
            .send()
            .await
            .map_err(|e| BirdeyeError::RequestError(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| BirdeyeError::SerializationError(e.to_string()))
    }

    pub async fn get_price_history(
        &self,
        address: &str,
        interval: TimeInterval,
    ) -> Result<Vec<PricePoint>, BirdeyeError> {
        let url = format!("{}/token/{}/price", API_BASE_URL, address);
        let response = self
            .client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .query(&[("interval", interval.to_string())])
            .send()
            .await
            .map_err(|e| BirdeyeError::RequestError(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| BirdeyeError::SerializationError(e.to_string()))
    }

    pub async fn get_wallet_portfolio(
        &self,
        wallet_address: &str,
    ) -> Result<WalletPortfolio, BirdeyeError> {
        let url = format!("{}/wallet/{}/portfolio", API_BASE_URL, wallet_address);
        let response = self
            .client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await
            .map_err(|e| BirdeyeError::RequestError(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| BirdeyeError::SerializationError(e.to_string()))
    }
}

// Implement Send and Sync for BirdeyeProvider since reqwest::Client is already Send + Sync
unsafe impl Send for BirdeyeProvider {}
unsafe impl Sync for BirdeyeProvider {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_search_tokens() -> Result<(), BirdeyeError> {
        let api_key = env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY must be set");
        let provider = BirdeyeProvider::new(api_key);
        let params = TokenSearchParams {
            query: "SOL".to_string(),
            offset: None,
            limit: Some(10),
        };

        let response = provider.search_tokens(params).await?;
        assert!(!response.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_token_overview() -> Result<(), BirdeyeError> {
        let api_key = env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY must be set");
        let provider = BirdeyeProvider::new(api_key);
        let overview = provider
            .get_token_overview("So11111111111111111111111111111111111111112".to_string())
            .await?;
        assert!(overview.price > 0.0);
        Ok(())
    }
}

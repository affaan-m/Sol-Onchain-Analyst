use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use serde_json::json;
use crate::{
    types::api::TokenInfo,
    TokenOverview, WalletPortfolio, LiquidityAnalysis, MarketImpact, PricePoint,
};

/// Mock response data for testing
#[derive(Clone)]
pub struct MockResponse {
    pub status: u16,
    pub body: serde_json::Value,
    pub delay: Duration,
}

/// Mock HTTP client for testing
pub struct MockHttpClient {
    responses: Arc<RwLock<HashMap<String, MockResponse>>>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        let mut responses = HashMap::new();
        
        // Add default mock responses
        responses.insert(
            "/token/search".to_string(),
            MockResponse {
                status: 200,
                body: json!({
                    "data": [
                        {
                            "address": "So11111111111111111111111111111111111111112",
                            "symbol": "SOL",
                            "name": "Wrapped SOL",
                            "decimals": 9,
                            "price_usd": 100.0,
                            "volume_24h": 1000000.0,
                            "market_cap": 10000000000.0
                        }
                    ],
                    "pagination": {
                        "offset": 0,
                        "limit": 10,
                        "total": 1
                    }
                }),
                delay: Duration::from_millis(100),
            },
        );

        responses.insert(
            "/token/overview".to_string(),
            MockResponse {
                status: 200,
                body: json!({
                    "address": "So11111111111111111111111111111111111111112",
                    "decimals": 9,
                    "symbol": "SOL",
                    "name": "Wrapped SOL",
                    "price": 100.0,
                    "volume_24h": 1000000.0,
                    "price_change_24h": 5.0,
                    "liquidity": 500000.0,
                    "holders": 1000000
                }),
                delay: Duration::from_millis(50),
            },
        );

        Self {
            responses: Arc::new(RwLock::new(responses)),
        }
    }

    /// Add or update a mock response
    pub async fn set_response(&self, endpoint: &str, response: MockResponse) {
        let mut responses = self.responses.write().await;
        responses.insert(endpoint.to_string(), response);
    }

    /// Simulate a rate limit error
    pub async fn simulate_rate_limit(&self, endpoint: &str) {
        self.set_response(
            endpoint,
            MockResponse {
                status: 429,
                body: json!({"error": "Rate limit exceeded"}),
                delay: Duration::from_millis(50),
            },
        ).await;
    }

    /// Simulate a network error
    pub async fn simulate_network_error(&self, endpoint: &str) {
        self.set_response(
            endpoint,
            MockResponse {
                status: 500,
                body: json!({"error": "Internal server error"}),
                delay: Duration::from_millis(50),
            },
        ).await;
    }

    /// Get mock response for an endpoint
    pub async fn get(&self, endpoint: &str) -> Option<MockResponse> {
        let responses = self.responses.read().await;
        responses.get(endpoint).cloned()
    }
}

/// Create test data for various response types
pub fn create_test_token_info() -> TokenInfo {
    TokenInfo {
        address: "So11111111111111111111111111111111111111112".to_string(),
        symbol: "SOL".to_string(),
        name: "Wrapped SOL".to_string(),
        decimals: 9,
        price: Some(100.0),
        volume_24h: Some(1000000.0),
        market_cap: Some(10000000000.0),
    }
}

pub fn create_test_token_overview() -> TokenOverview {
    TokenOverview {
        address: "So11111111111111111111111111111111111111112".to_string(),
        symbol: "SOL".to_string(),
        name: "Wrapped SOL".to_string(),
        decimals: 9,
        price: 100.0,
        volume_24h: 1000000.0,
        market_cap: 10000000000.0,
        fully_diluted_market_cap: Some(12000000000.0),
        total_supply: 100000000.0,
        circulating_supply: Some(80000000.0),
    }
}

pub fn create_test_wallet_portfolio() -> WalletPortfolio {
    WalletPortfolio {
        wallet_address: "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK".to_string(),
        total_value_usd: 1000000.0,
        tokens: vec![],
    }
}

pub fn create_test_liquidity_analysis() -> LiquidityAnalysis {
    LiquidityAnalysis {
        total_bid_liquidity: 250000.0,
        total_ask_liquidity: 250000.0,
        bid_ask_ratio: 1.0,
        depth_quality: 0.8,
    }
}

pub fn create_test_market_impact() -> MarketImpact {
    MarketImpact {
        price_impact: 0.01,
        executed_price: 101.0,
        size_usd: 10000.0,
        size_tokens: 100.0,
    }
}

pub fn create_test_price_history() -> Vec<PricePoint> {
    vec![
        PricePoint {
            timestamp: chrono::Utc::now().timestamp(),
            price: 100.0,
            volume: 100000.0,
        },
        PricePoint {
            timestamp: chrono::Utc::now().timestamp() - 3600,
            price: 99.0,
            volume: 95000.0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_mock_client_basic() {
        let client = MockHttpClient::new();
        let response = client.get("/token/search").await.unwrap();
        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn test_mock_client_rate_limit() {
        let client = MockHttpClient::new();
        client.simulate_rate_limit("/token/search").await;
        let response = client.get("/token/search").await.unwrap();
        assert_eq!(response.status, 429);
    }

    #[tokio::test]
    async fn test_mock_client_delay() {
        let client = MockHttpClient::new();
        let start = Instant::now();
        let response = client.get("/token/search").await.unwrap();
        assert!(start.elapsed() >= response.delay);
    }

    #[tokio::test]
    async fn test_mock_client_custom_response() {
        let client = MockHttpClient::new();
        client.set_response(
            "/custom",
            MockResponse {
                status: 200,
                body: json!({"test": true}),
                delay: Duration::from_millis(0),
            },
        ).await;
        
        let response = client.get("/custom").await.unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, json!({"test": true}));
    }
}

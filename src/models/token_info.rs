use bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price: f64,
    pub volume_24h: f64,
    pub market_cap: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub volume_change_24h: Option<f64>,
    pub liquidity: f64,
    pub trade_24h: Option<i64>,
    pub logo_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<TokenExtensions>,
    pub timestamp: DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExtensions {
    #[serde(rename = "coingecko_id")]
    pub coingecko_id: Option<String>,
    #[serde(rename = "serum_v3_usdc")]
    pub serum_v3_usdc: Option<String>,
    #[serde(rename = "serum_v3_usdt")]
    pub serum_v3_usdt: Option<String>,
    pub website: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub description: Option<String>,
    pub discord: Option<String>,
    pub medium: Option<String>,
}

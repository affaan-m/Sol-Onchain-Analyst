use serde::{Deserialize, Serialize};
use bson::{DateTime, oid::ObjectId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingTokenResponse {
    pub data: TrendingTokenData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingTokenData {
    #[serde(rename = "updateUnixTime")]
    pub update_unix_time: i64,
    #[serde(rename = "updateTime")]
    pub update_time: String,
    pub tokens: Vec<TrendingToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingToken {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub address: String,
    pub decimals: i32,
    pub liquidity: Option<f64>,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub name: String,
    pub symbol: String,
    #[serde(rename = "volume24hUSD")]
    pub volume_24h_usd: Option<f64>,
    #[serde(rename = "volume24hChangePercent")]
    pub volume_24h_change_percent: Option<f64>,
    pub fdv: Option<f64>,
    pub marketcap: Option<f64>,
    pub rank: Option<i32>,
    pub price: Option<f64>,
    #[serde(rename = "price24hChangePercent")]
    pub price_24h_change_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime>,
}

impl TrendingToken {
    pub fn collection_name() -> &'static str {
        "trending_tokens"
    }
} 
use bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingTokenData {
    pub tokens: Vec<TrendingToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingToken {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub address: String,
    pub decimals: i32,
    pub liquidity: f64,
    pub logo_uri: String,
    pub name: String,
    pub symbol: String,
    pub volume_24h_usd: f64,
    pub rank: i32,
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_24h_change_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fdv: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketcap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_24h_change_percent: Option<f64>,
}


use bson::{self, oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

pub mod token_info;
pub mod token_trending;
pub mod token_analytics;
pub mod market_signal;
pub mod market_data;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetrics {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub token_address: String,
    pub metrics: bson::Document,
    pub timestamp: DateTime,
}

// Add typed collection helpers
impl TokenMetrics {
    pub fn collection_name() -> &'static str {
        "token_metrics"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub vector: Vec<f32>,
    pub metadata: bson::Document,
    pub timestamp: DateTime,
}

impl VectorDocument {
    pub fn collection_name() -> &'static str {
        "vectors"
    }
}

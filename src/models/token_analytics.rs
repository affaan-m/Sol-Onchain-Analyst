use bigdecimal::BigDecimal;
// use crate::MongoDbPool;
use bson::{DateTime, Document, oid::ObjectId};
use serde::{Deserialize, Serialize};
// use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalytics {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub token_address: String,
    pub token_name: String,
    pub token_symbol: String,
    pub price: BigDecimal,
    pub volume_24h: Option<BigDecimal>,
    pub market_cap: Option<BigDecimal>,
    pub total_supply: Option<BigDecimal>,
    pub holder_count: Option<i32>,
    pub timestamp: DateTime,
    pub created_at: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Document>,
}
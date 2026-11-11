use bigdecimal::BigDecimal;
// use crate::MongoDbPool;
use bson::{oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};
// use time::OffsetDateTime;

/// TokenAnalytics represents token market data with MongoDB Atlas Search vector index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalytics {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    // Base token data
    pub token_address: String,    // type: "string"
    pub token_name: String,       // type: "string"
    pub token_symbol: String,     // type: "string"
    pub decimals: u8,             // type: "number"
    pub logo_uri: Option<String>, // type: "string"

    // Price metrics
    pub price: BigDecimal,                    // type: "number"
    pub price_change_24h: Option<BigDecimal>, // type: "number"
    pub price_change_7d: Option<BigDecimal>,  // type: "number"

    // Volume metrics
    pub volume_24h: Option<BigDecimal>,          // type: "number"
    pub volume_change_24h: Option<BigDecimal>,   // type: "number"
    pub volume_by_price_24h: Option<BigDecimal>, // type: "number"

    // Market metrics
    pub market_cap: Option<BigDecimal>, // type: "number"
    pub fully_diluted_market_cap: Option<BigDecimal>, // type: "number"
    pub circulating_supply: Option<BigDecimal>, // type: "number"
    pub total_supply: Option<BigDecimal>, // type: "number"

    // Liquidity metrics
    pub liquidity: Option<BigDecimal>,            // type: "number"
    pub liquidity_change_24h: Option<BigDecimal>, // type: "number"

    // Trading metrics
    pub trades_24h: Option<i64>,                // type: "number"
    pub average_trade_size: Option<BigDecimal>, // type: "number"

    // Holder metrics
    pub holder_count: Option<i32>,           // type: "number"
    pub active_wallets_24h: Option<i32>,     // type: "number"
    pub whale_transactions_24h: Option<i32>, // type: "number"

    // Technical indicators
    pub rsi_14: Option<BigDecimal>,          // type: "number"
    pub macd: Option<BigDecimal>,            // type: "number"
    pub macd_signal: Option<BigDecimal>,     // type: "number"
    pub bollinger_upper: Option<BigDecimal>, // type: "number"
    pub bollinger_lower: Option<BigDecimal>, // type: "number"

    // Timestamps and metadata
    pub timestamp: DateTime,               // type: "date"
    pub created_at: Option<DateTime>,      // type: "date"
    pub last_trade_time: Option<DateTime>, // type: "date"

    // Extensions and metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Document>, // type: "document"

    // Vector embedding for similarity search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>, // type: "knnVector", dimensions: 1536
}

// MongoDB Atlas Search Vector Index Definition (for reference):
// {
//   "mappings": {
//     "dynamic": false,
//     "fields": {
//       "token_address": { "type": "string" },
//       "token_name": { "type": "string" },
//       "token_symbol": { "type": "string" },
//       "decimals": { "type": "number" },
//       "logo_uri": { "type": "string" },
//       "price": { "type": "number" },
//       "price_change_24h": { "type": "number" },
//       "price_change_7d": { "type": "number" },
//       "volume_24h": { "type": "number" },
//       "volume_change_24h": { "type": "number" },
//       "volume_by_price_24h": { "type": "number" },
//       "market_cap": { "type": "number" },
//       "fully_diluted_market_cap": { "type": "number" },
//       "circulating_supply": { "type": "number" },
//       "total_supply": { "type": "number" },
//       "liquidity": { "type": "number" },
//       "liquidity_change_24h": { "type": "number" },
//       "trades_24h": { "type": "number" },
//       "average_trade_size": { "type": "number" },
//       "holder_count": { "type": "number" },
//       "active_wallets_24h": { "type": "number" },
//       "whale_transactions_24h": { "type": "number" },
//       "rsi_14": { "type": "number" },
//       "macd": { "type": "number" },
//       "macd_signal": { "type": "number" },
//       "bollinger_upper": { "type": "number" },
//       "bollinger_lower": { "type": "number" },
//       "timestamp": { "type": "date" },
//       "created_at": { "type": "date" },
//       "last_trade_time": { "type": "date" },
//       "metadata": { "type": "document" },
//       "embedding": {
//         "type": "knnVector",
//         "dimensions": 1536
//       }
//     }
//   }
// }

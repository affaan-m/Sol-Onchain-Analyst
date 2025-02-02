use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;
use serde_json::Value;

#[derive(Debug, Clone, FromRow)]
pub struct TokenAnalytics {
    pub id: Option<Uuid>,
    pub token_address: String,
    pub token_name: String,
    pub token_symbol: String,
    pub price: BigDecimal,
    pub volume_24h: Option<BigDecimal>,
    pub market_cap: Option<BigDecimal>,
    pub total_supply: Option<BigDecimal>,
    pub holder_count: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
} 
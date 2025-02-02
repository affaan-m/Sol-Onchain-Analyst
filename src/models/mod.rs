use sqlx::Type;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};

pub mod market_signal;
pub mod token_analytics;


#[derive(Debug, Clone, PartialEq, Type)]
#[sqlx(type_name = "trade_status", rename_all = "lowercase")]
pub enum TradeStatus {
    Open,
    Closed,
    Pending,
    Executed,
    Cancelled,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TokenAnalytics {
    pub id: Option<i32>,
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
}
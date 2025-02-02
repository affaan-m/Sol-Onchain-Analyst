use anyhow::Result;
use bigdecimal::{BigDecimal, ToPrimitive};
use crate::models::market_signal::MarketSignal;
use crate::models::token_analytics::TokenAnalytics;
use crate::utils::f64_to_decimal;
use std::sync::Arc;
use sqlx::PgPool;

pub struct PortfolioOptimizer {
    db: Arc<PgPool>,
}

impl PortfolioOptimizer {
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }

    pub async fn get_allocation(&self, _token: &TokenAnalytics, _signal: &MarketSignal) -> Result<BigDecimal> {
        // For now, return a default allocation
        Ok(f64_to_decimal(0.1)) // 10% allocation
    }

    pub async fn get_position_allocation(&self, address: &str) -> Result<BigDecimal> {
        let allocation = sqlx::query!(
            r#"
            SELECT allocation
            FROM position_allocations
            WHERE token_address = $1
            "#,
            address
        )
        .fetch_optional(&*self.db)
        .await?;

        Ok(allocation
            .map(|a| f64_to_decimal(a.allocation.to_f64().unwrap_or(0.0)))
            .unwrap_or_else(|| f64_to_decimal(0.0)))
    }
}

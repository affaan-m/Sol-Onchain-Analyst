use anyhow::Result;
use bigdecimal::{BigDecimal, ToPrimitive};
use crate::models::market_signal::MarketSignal;
use crate::models::token_analytics::TokenAnalytics;
use crate::utils::f64_to_decimal;
use std::sync::Arc;
use rig_mongodb::{MongoDbPool, bson::doc};
use crate::error::Error;
use crate::models::allocation::Allocation;

pub struct PortfolioOptimizer {
    db: Arc<MongoDbPool>,
}

impl PortfolioOptimizer {
    pub fn new(db: Arc<MongoDbPool>) -> Self {
        Self { db }
    }

    pub async fn get_allocation(&self, _token: &TokenAnalytics, _signal: &MarketSignal) -> Result<BigDecimal> {
        // For now, return a default allocation
        Ok(f64_to_decimal(0.1)) // 10% allocation
    }

    pub async fn get_position_allocation(&self, address: &str) -> Result<BigDecimal> {
        let collection = self.db.collection("allocations");
        
        let filter = doc! {
            "token_address": address,
        };
        
        let doc = collection.find_one(filter, None)
            .await?;
            
        let allocation = doc
            .and_then(|d| d.get_f64("allocation"))
            .unwrap_or(0.0);

        Ok(f64_to_decimal(allocation))
    }

    async fn get_allocation(&self, token_address: &str) -> Result<Option<Allocation>, Error> {
        let collection = self.db.database("cainam").collection("allocations");
        
        let filter = doc! {
            "token_address": token_address,
        };
        
        collection.find_one(filter, None)
            .await
            .map_err(|e| Error::Database(e.to_string()))
    }
}

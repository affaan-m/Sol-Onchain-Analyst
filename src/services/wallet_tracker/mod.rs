use anyhow::{Context, Result};
use mongodb::{bson::{doc, Document}, Collection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::config::mongodb::MongoDbPool;

pub mod models;
pub use models::*;

/// Service for tracking KOL (Key Opinion Leader) wallets and their token holdings
pub struct WalletTrackerService {
    db_pool: Arc<MongoDbPool>,
}

impl WalletTrackerService {
    pub fn new(db_pool: Arc<MongoDbPool>) -> Self {
        Self { db_pool }
    }

    /// Get collection of KOL wallets
    fn get_kol_wallets_collection(&self) -> Result<Collection<KolWallet>> {
        let db = self.db_pool.get_database()?;
        Ok(db.collection::<KolWallet>("kol_wallets"))
    }
    
    /// Get collection of token recommendations
    fn get_token_recommendations_collection(&self) -> Result<Collection<TokenRecommendation>> {
        let db = self.db_pool.get_database()?;
        Ok(db.collection::<TokenRecommendation>("token_recommendations"))
    }
    
    /// Add a new KOL wallet to the database
    pub async fn add_kol_wallet(&self, kol_wallet: KolWallet) -> Result<()> {
        let collection = self.get_kol_wallets_collection()?;
        collection.insert_one(kol_wallet, None)
            .await
            .context("Failed to insert KOL wallet")?;
        Ok(())
    }
    
    /// Get all active KOL wallets
    pub async fn get_active_kol_wallets(&self) -> Result<Vec<KolWallet>> {
        let collection = self.get_kol_wallets_collection()?;
        let filter = doc! { "active": true };
        let wallets = collection.find(filter, None)
            .await
            .context("Failed to find active KOL wallets")?
            .try_collect()
            .await
            .context("Failed to collect KOL wallets")?;
        Ok(wallets)
    }
    
    /// Check if any KOLs have purchased a specific token
    pub async fn check_kol_ownership(&self, token_address: &str) -> Result<Vec<KolOwnership>> {
        // This would typically involve calling a blockchain API to check wallet holdings
        // For now, we'll return a placeholder implementation
        info!("Checking KOL ownership for token: {}", token_address);
        
        // Get active KOL wallets
        let wallets = self.get_active_kol_wallets().await?;
        
        // Placeholder: In a real implementation, we would:
        // 1. Call Solana blockchain API to check token holdings for each wallet
        // 2. Return actual ownership data
        
        // For now, return empty vector
        Ok(Vec::new())
    }
    
    /// Update token recommendation with KOL ownership data
    pub async fn update_token_recommendation_with_kol_data(
        &self, 
        token_address: &str
    ) -> Result<()> {
        // Get KOL ownership data
        let kol_ownership = self.check_kol_ownership(token_address).await?;
        
        // Update token recommendation if KOL ownership exists
        if !kol_ownership.is_empty() {
            let collection = self.get_token_recommendations_collection()?;
            let filter = doc! { "token_address": token_address };
            let update = doc! { 
                "$set": { "kol_ownership": bson::to_bson(&kol_ownership)? } 
            };
            
            collection.update_one(filter, update, None)
                .await
                .context("Failed to update token recommendation with KOL data")?;
                
            info!("Updated token recommendation with KOL ownership data: {} KOLs", kol_ownership.len());
        }
        
        Ok(())
    }
} 
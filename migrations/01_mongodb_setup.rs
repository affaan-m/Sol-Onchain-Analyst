use rig_mongodb::{MongoDbPool, bson::doc};
use anyhow::Result;
use tracing::info;
use crate::config::{mongodb::MongoConfig, pool::MongoPoolConfig};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    info!("Starting MongoDB migrations...");
    
    // Use migration-specific configuration
    let config = MongoConfig {
        pool: MongoPoolConfig {
            min_pool_size: 1,
            max_pool_size: 2,
            connect_timeout: std::time::Duration::from_secs(30),
        },
        ..MongoConfig::from_env()
    };
    
    let pool = config.create_pool().await?;
    let db = pool.database(&config.database);
    
    info!("Creating collections and indexes...");

    // Token analytics collection
    db.create_collection("token_analytics", None).await?;
    db.collection("token_analytics").create_index(
        doc! {
            "token_address": 1,
            "timestamp": -1
        },
        None,
    ).await?;

    // Market signals collection
    db.create_collection("market_signals", None).await?;
    db.collection("market_signals").create_index(
        doc! {
            "asset_address": 1,
            "timestamp": -1
        },
        None,
    ).await?;

    // Vector store collection with improved search configuration
    db.create_collection("vectors", None).await?;
    db.collection("vectors").create_index(
        doc! {
            "vector": "2dsphere",
            "metadata.timestamp": -1,
            "weights": {
                "vector": 1,
                "metadata.timestamp": 1
            },
            "name": "vector_search_idx",
            "background": true
        },
        None,
    ).await?;

    // Trade history collection
    db.create_collection("trade_history", None).await?;
    db.collection("trade_history").create_index(
        doc! {
            "trader_address": 1,
            "timestamp": -1,
            "status": 1
        },
        None,
    ).await?;

    // Risk models collection
    db.create_collection("risk_models", None).await?;
    db.collection("risk_models").create_index(
        doc! {
            "model_type": 1,
            "asset_address": 1,
            "timestamp": -1
        },
        None,
    ).await?;

    // Portfolio allocations collection
    db.create_collection("portfolio_allocations", None).await?;
    db.collection("portfolio_allocations").create_index(
        doc! {
            "wallet_address": 1,
            "token_address": 1,
            "timestamp": -1
        },
        None,
    ).await?;

    info!("MongoDB migrations completed successfully!");
    Ok(())
}
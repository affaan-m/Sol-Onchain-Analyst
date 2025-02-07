use rig_mongodb::{MongoDbPool, bson::doc};
use anyhow::Result;
use tracing::info;
use crate::config::mongodb::MongoConfig;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    info!("Running position allocations migration...");
    
    let config = MongoConfig::from_env();
    let pool = config.create_pool().await?;
    let db = pool.database(&config.database);

    // Create portfolio_allocations collection with validation
    db.create_collection("portfolio_allocations", Some(doc! {
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["wallet_address", "token_address", "allocation_weight", "timestamp"],
                "properties": {
                    "wallet_address": { "bsonType": "string" },
                    "token_address": { "bsonType": "string" },
                    "allocation_weight": { "bsonType": "decimal128" },
                    "target_weight": { "bsonType": "decimal128" },
                    "min_weight": { "bsonType": "decimal128" },
                    "max_weight": { "bsonType": "decimal128" },
                    "last_rebalance": { "bsonType": "date" },
                    "timestamp": { "bsonType": "date" }
                }
            }
        }
    })).await?;

    // Create indexes for efficient allocation lookups
    db.collection("portfolio_allocations").create_index(
        doc! {
            "wallet_address": 1,
            "token_address": 1,
            "timestamp": -1
        },
        None,
    ).await?;

    // Create rebalance_history collection
    db.create_collection("rebalance_history", Some(doc! {
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["wallet_address", "token_address", "old_weight", "new_weight", "timestamp"],
                "properties": {
                    "wallet_address": { "bsonType": "string" },
                    "token_address": { "bsonType": "string" },
                    "old_weight": { "bsonType": "decimal128" },
                    "new_weight": { "bsonType": "decimal128" },
                    "reason": { "bsonType": "string" },
                    "timestamp": { "bsonType": "date" }
                }
            }
        }
    })).await?;

    // Create indexes for rebalance history
    db.collection("rebalance_history").create_index(
        doc! {
            "wallet_address": 1,
            "timestamp": -1
        },
        None,
    ).await?;

    info!("Position allocations migration completed successfully!");
    Ok(())
}
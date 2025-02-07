use rig_mongodb::{MongoDbPool, bson::doc};
use anyhow::Result;
use tracing::info;
use crate::config::mongodb::MongoConfig;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    info!("Running trade status migration...");
    
    let config = MongoConfig::from_env();
    let pool = config.create_pool().await?;
    let db = pool.database(&config.database);

    // Create trade_history collection with status validation
    db.create_collection("trade_history", Some(doc! {
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["trade_id", "status", "updated_at"],
                "properties": {
                    "trade_id": { "bsonType": "string" },
                    "status": { 
                        "enum": [
                            "initiated",
                            "pending",
                            "completed",
                            "failed",
                            "cancelled",
                            "timeout"
                        ] 
                    },
                    "updated_at": { "bsonType": "date" },
                    "error": { "bsonType": "string" }
                }
            }
        }
    })).await?;

    // Create indexes for efficient status tracking
    db.collection("trade_history").create_index(
        doc! {
            "trade_id": 1,
            "updated_at": -1
        },
        None,
    ).await?;

    db.collection("trade_history").create_index(
        doc! {
            "status": 1,
            "updated_at": -1
        },
        None,
    ).await?;

    info!("Trade status migration completed successfully!");
    Ok(())
}
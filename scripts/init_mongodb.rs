use rig_mongodb::{MongoDbPool, bson::doc, options::ClientOptions};
use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let mongodb_uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    info!("Connecting to MongoDB...");
    
    let client_options = ClientOptions::parse(&mongodb_uri).await?;
    let pool = MongoDbPool::new_with_options(&mongodb_uri, client_options).await?;
    let db = pool.database("cainam");
    
    info!("Creating collections and indexes...");
    
    // Create token_analytics collection with indexes
    db.create_collection("token_analytics", None).await?;
    db.collection("token_analytics").create_index(
        doc! {
            "token_address": 1,
            "timestamp": -1
        },
        None,
    ).await?;

    // Create market_signals collection with indexes
    db.create_collection("market_signals", None).await?;
    db.collection("market_signals").create_index(
        doc! {
            "asset_address": 1,
            "timestamp": -1
        },
        None,
    ).await?;

    // Create vector_store collection with indexes
    db.create_collection("vectors", None).await?;
    db.collection("vectors").create_index(
        doc! {
            "vector": "2dsphere",
            "metadata.timestamp": -1
        },
        None,
    ).await?;

    info!("MongoDB setup completed successfully!");
    Ok(())
}
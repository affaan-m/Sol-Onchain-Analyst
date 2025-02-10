use anyhow::Result;
use mongodb::{
    bson::{doc, Document},
    Client, IndexModel,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    info!("Initializing vector store collections...");

    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let client = Client::with_uri_str(&uri).await?;
    let db = client.database("analytics");

    // Create token_analytics collection
    info!("Creating token_analytics collection...");
    match db.create_collection("token_analytics").await {
        Ok(_) => info!("Created token_analytics collection"),
        Err(e) if e.to_string().contains("already exists") => {
            info!("Collection token_analytics already exists")
        }
        Err(e) => return Err(e.into()),
    }

    // Create market_signals collection
    info!("Creating market_signals collection...");
    match db.create_collection("market_signals").await {
        Ok(_) => info!("Created market_signals collection"),
        Err(e) if e.to_string().contains("already exists") => {
            info!("Collection market_signals already exists")
        }
        Err(e) => return Err(e.into()),
    }

    // Create vector search index for token_analytics
    info!("Creating vector search index for token_analytics...");
    let command = doc! {
        "createSearchIndexes": "token_analytics",
        "indexes": [{
            "name": "vector_index",
            "definition": {
                "mappings": {
                    "dynamic": true,
                    "fields": {
                        "embedding": {
                            "type": "knnVector",
                            "dimensions": 1536,
                            "similarity": "cosine"
                        }
                    }
                }
            }
        }]
    };
    
    match db.run_command(command).await {
        Ok(_) => info!("Created vector search index for token_analytics"),
        Err(e) if e.to_string().contains("already exists") => {
            info!("Vector search index for token_analytics already exists")
        }
        Err(e) => return Err(e.into()),
    }

    // Create metadata index for token_analytics
    info!("Creating metadata index for token_analytics...");
    let index_model = IndexModel::builder()
        .keys(doc! {
            "token_address": 1,
            "timestamp": -1
        })
        .build();
    
    let collection = db.collection::<Document>("token_analytics");
    match collection.create_index(index_model).await {
        Ok(_) => info!("Created metadata index for token_analytics"),
        Err(e) if e.to_string().contains("already exists") => {
            info!("Metadata index for token_analytics already exists")
        }
        Err(e) => return Err(e.into()),
    }

    // Create metadata index for market_signals
    info!("Creating metadata index for market_signals...");
    let index_model = IndexModel::builder()
        .keys(doc! {
            "asset_address": 1,
            "timestamp": -1
        })
        .build();
    
    let collection = db.collection::<Document>("market_signals");
    match collection.create_index(index_model).await {
        Ok(_) => info!("Created metadata index for market_signals"),
        Err(e) if e.to_string().contains("already exists") => {
            info!("Metadata index for market_signals already exists")
        }
        Err(e) => return Err(e.into()),
    }

    info!("Vector store initialization completed successfully!");
    Ok(())
} 
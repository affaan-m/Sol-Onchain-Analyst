use mongodb::{
    bson::Document,
    options::{IndexOptions, ClientOptions, CreateCollectionOptions},
    Client, IndexModel,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize MongoDB client
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let db = client.database("cainam");

    println!("Connected to MongoDB successfully");

    // Create collections if they don't exist
    println!("Creating collections...");
    let collections = db.list_collection_names().await?;
    
    if !collections.contains(&"token_analytics".to_string()) {
        db.create_collection("token_analytics").await?;
        println!("Created token_analytics collection");
    }
    
    if !collections.contains(&"market_signals".to_string()) {
        db.create_collection("market_signals").await?;
        println!("Created market_signals collection");
    }

    // Get collections
    let token_analytics = db.collection::<Document>("token_analytics");
    let market_signals = db.collection::<Document>("market_signals");

    // Create indexes for token_analytics collection
    println!("Creating indexes for token_analytics collection...");
    
    // Compound index on token_address and timestamp
    let compound_index_options = IndexOptions::builder().build();
    let compound_index = IndexModel::builder()
        .keys(mongodb::bson::doc! {
            "token_address": 1,
            "timestamp": 1
        })
        .options(compound_index_options)
        .build();
    token_analytics.create_index(compound_index).await?;

    // Standard index for embeddings
    let embedding_index_options = IndexOptions::builder()
        .name(Some("embedding_index".to_string()))
        .build();
    let embedding_index = IndexModel::builder()
        .keys(mongodb::bson::doc! {
            "embedding": 1
        })
        .options(embedding_index_options)
        .build();
    token_analytics.create_index(embedding_index).await?;

    println!("Created indexes for token_analytics collection");

    // Create indexes for market_signals collection
    println!("Creating indexes for market_signals collection...");
    
    let market_index_options = IndexOptions::builder().build();
    let market_index = IndexModel::builder()
        .keys(mongodb::bson::doc! {
            "asset_address": 1,
            "timestamp": 1
        })
        .options(market_index_options)
        .build();
    market_signals.create_index(market_index).await?;

    println!("Created indexes for market_signals collection");
    println!("Successfully created all MongoDB indexes");

    Ok(())
} 
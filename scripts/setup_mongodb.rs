use cainam_core::config::mongodb::MongoConfig;
use mongodb::{
    bson::Document,
    options::{ClientOptions, CreateCollectionOptions, IndexOptions},
    Client, IndexModel,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize MongoDB client using configuration
    let config = MongoConfig::from_env();
    let mut client_options = ClientOptions::parse(&config.uri).await?;
    client_options.server_api = Some(
        mongodb::options::ServerApi::builder()
            .version(mongodb::options::ServerApiVersion::V1)
            .build(),
    );
    let client = Client::with_options(client_options)?;
    let db = client.database(&config.database);

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

    // Create vector search index for embeddings
    let vector_search_command = mongodb::bson::doc! {
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

    db.run_command(vector_search_command).await?;
    println!("Created vector search index for token_analytics collection");

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

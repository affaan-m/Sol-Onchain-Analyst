use cainam_core::config::mongodb::MongoConfig;
use mongodb::{
    bson::Document,
    options::{ClientOptions, IndexOptions},
    Client, IndexModel,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables first
    dotenvy::dotenv().ok();
    
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
    } else {
        println!("Collection token_analytics already exists");
    }

    if !collections.contains(&"market_signals".to_string()) {
        db.create_collection("market_signals").await?;
        println!("Created market_signals collection");
    } else {
        println!("Collection market_signals already exists");
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
    
    match token_analytics.create_index(compound_index).await {
        Ok(_) => println!("Created compound index for token_analytics"),
        Err(e) if e.to_string().contains("already exists") => {
            println!("Compound index already exists for token_analytics");
        }
        Err(e) => return Err(e.into()),
    }

    // Create vector search index for embeddings
    let vector_search_command = mongodb::bson::doc! {
        "createSearchIndexes": "token_analytics",
        "indexes": [{
            "name": "vector_index",
            "definition": {
                "mappings": {
                    "dynamic": true,
                    "fields": {
                        "id": {
                            "type": "string"
                        },
                        "token_address": {
                            "type": "string"
                        },
                        "token_name": {
                            "type": "string"
                        },
                        "token_symbol": {
                            "type": "string"
                        },
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

    match db.run_command(vector_search_command).await {
        Ok(_) => println!("Created vector search index for token_analytics collection"),
        Err(e) if e.to_string().contains("already defined") => {
            println!("Vector search index already exists for token_analytics collection");
        }
        Err(e) => return Err(e.into()),
    }

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
    
    match market_signals.create_index(market_index).await {
        Ok(_) => println!("Created index for market_signals"),
        Err(e) if e.to_string().contains("already exists") => {
            println!("Index already exists for market_signals");
        }
        Err(e) => return Err(e.into()),
    }

    println!("MongoDB setup completed successfully!");

    Ok(())
}

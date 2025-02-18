use cainam_core::{
    config::mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig},
    birdeye::api::BirdeyeClient,
    models::trending_token::TrendingToken,
    services::token_analytics::TokenAnalyticsService,
};
use mongodb::bson::doc;
use std::sync::Arc;
use std::env;
use tokio;
use tracing::{info, error, Level};
use dotenvy::dotenv;
use anyhow::{Result, Context};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    info!("Starting token analytics capture...");

    // Load environment variables
    dotenv().ok();
    
    // Get MongoDB connection details
    let mongodb_uri = env::var("MONGODB_URI")
        .context("MONGODB_URI must be set")?;
    let mongodb_database = env::var("MONGODB_DATABASE")
        .context("MONGODB_DATABASE must be set")?;
    
    info!("Connecting to MongoDB at: {}", mongodb_uri);

    // Initialize MongoDB connection
    let config = MongoConfig {
        uri: mongodb_uri,
        database: mongodb_database.clone(),
        app_name: Some("token-analytics-capture".to_string()),
        pool_config: MongoPoolConfig::default(),
    };

    let db_pool = MongoDbPool::create_pool(config).await?;
    info!("Successfully connected to MongoDB");

    // Initialize Birdeye client
    let birdeye_api_key = env::var("BIRDEYE_API_KEY")
        .context("BIRDEYE_API_KEY must be set")?;
    
    let birdeye_client = Arc::new(BirdeyeClient::new(birdeye_api_key.clone()));
    info!("Initialized Birdeye client");

    // Initialize TokenAnalyticsService
    let analytics_service = TokenAnalyticsService::new(
        db_pool.clone(),
        birdeye_client.clone(),
        None
    ).await?;
    info!("Initialized TokenAnalyticsService");

    // Get database and collections
    let db = db_pool.database(&mongodb_database);
    let trending_collection = db.collection::<TrendingToken>("trending_tokens");

    // Get the most recent trending tokens with sorting by timestamp
    let query = doc! {
        "$query": {},
        "$orderby": { "timestamp": -1 }
    };

    let mut cursor = trending_collection
        .find(query)
        .await?;
    let mut processed = 0;
    let mut errors = 0;

    // Process each trending token
    while let Some(token_result) = cursor.next().await {
        match token_result {
            Ok(token) => {
                info!("Processing analytics for token: {}", token.symbol);
                
                match analytics_service.fetch_and_store_token_info(&token.symbol, &token.address).await {
                    Ok(_) => {
                        processed += 1;
                        info!("Successfully stored analytics for {}", token.symbol);
                    },
                    Err(e) => {
                        errors += 1;
                        error!("Failed to process token {}: {}", token.symbol, e);
                    }
                }

                // Add a small delay to respect rate limits
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            },
            Err(e) => {
                error!("Error fetching trending token: {}", e);
                errors += 1;
            }
        }
    }

    info!("Token analytics capture completed:");
    info!("Successfully processed: {}", processed);
    info!("Errors: {}", errors);
    
    Ok(())
} 
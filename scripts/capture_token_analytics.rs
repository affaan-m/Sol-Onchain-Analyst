use anyhow::{Context, Result};
use cainam_core::{
    birdeye::api::{BirdeyeApi, BirdeyeClient},
    config::mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig},
    models::token_trending::TrendingToken,
    services::token_analytics::TokenAnalyticsService,
};
use dotenvy::dotenv;
use futures::TryStreamExt;
use mongodb::bson::doc;
use std::sync::Arc;
use tokio;
use tracing::{error, info, Level};

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
    let mongodb_uri = dotenvy::var("MONGODB_URI").context("MONGODB_URI must be set")?;
    let mongodb_database = dotenvy::var("MONGODB_DATABASE").context("MONGODB_DATABASE must be set")?;

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
    let birdeye_api_key = dotenvy::var("BIRDEYE_API_KEY").context("BIRDEYE_API_KEY must be set")?;
    let birdeye_client = Arc::new(BirdeyeClient::new(birdeye_api_key.clone()));
    info!("Initialized Birdeye client");

    // Initialize TokenAnalyticsService
    let analytics_service =
        TokenAnalyticsService::new(db_pool.clone(), birdeye_client.clone(), None).await?;
    info!("Initialized TokenAnalyticsService");

    // Get database and collections
    let db = db_pool.database(&mongodb_database);
    let trending_collection = db.collection::<TrendingToken>("token_trending");

    // Get tokens from the token_trending collection
    info!("Fetching tokens from token_trending collection...");
    let filter = doc! {};
    let mut cursor = trending_collection.find(filter).await?;
    let mut processed = 0;
    let mut errors = 0;

    // Process each token
    while let Some(token) = cursor.try_next().await? {
        info!("Processing analytics for token: {} ({})", token.symbol, token.address);

        // First get basic token overview
        match birdeye_client.get_token_overview(&token.address).await {
            Ok(overview) => {
                info!(
                    "Got token overview for {}: price=${:.4}, mcap=${:.2}",
                    token.symbol,
                    overview.price,
                    overview.market_cap.unwrap_or_default()
                );

                // If overview looks good, fetch and store detailed analytics
                match analytics_service
                    .fetch_and_store_token_info(&token.symbol, &token.address)
                    .await
                {
                    Ok(_) => {
                        processed += 1;
                        info!("Successfully stored analytics for {}", token.symbol);
                    }
                    Err(e) => {
                        errors += 1;
                        error!("Failed to store analytics for {}: {}", token.symbol, e);
                    }
                }
            }
            Err(e) => {
                errors += 1;
                error!("Failed to get overview for {}: {}", token.symbol, e);
            }
        }

        // Add a small delay to respect rate limits
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    info!("Token analytics capture completed:");
    info!("Successfully processed: {}", processed);
    info!("Errors: {}", errors);

    Ok(())
}

use anyhow::Result;
use cainam_core::{
    birdeye::BirdeyeClient,
    config::mongodb::MongoDbPool,
    config::{mongodb::MongoConfig, MarketConfig},
    services::token_analytics::TokenAnalyticsService,
};
use dotenvy::dotenv;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Market Data Capture...");

    // Load configurations
    let market_config = MarketConfig::new_from_env()?;
    let mongo_config = MongoConfig::from_env();

    // Initialize MongoDB connection
    let db = MongoDbPool::create_pool(mongo_config).await?;

    // Initialize Birdeye API client
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY must be set");
    let birdeye_api = Arc::new(BirdeyeClient::new(birdeye_api_key));

    // Create token analytics service
    let token_analytics =
        TokenAnalyticsService::new(db.clone(), birdeye_api, Some(market_config)).await?;

    info!("Services initialized. Beginning data capture...");

    // Run continuous market data capture
    loop {
        match token_analytics.update_market_data().await {
            Ok(_) => info!("Successfully updated market data"),
            Err(e) => info!("Error updating market data: {}", e),
        }

        // Wait for 5 minutes before next update
        time::sleep(Duration::from_secs(300)).await;
    }

    Ok(())
}

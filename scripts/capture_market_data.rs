use anyhow::Result;
use cainam_core::{
    birdeye::BirdeyeClient,
    services::TokenDataService,
};
use dotenvy::dotenv;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Load environment variables
    dotenv()?;

    let mongo_uri = std::env::var("MONGODB_URI")?;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY")?;
    
    // Initialize token data service
    let token_data_service = TokenDataService::new(mongo_uri, birdeye_api_key).await?;

    // Get market tokens from environment
    let market_tokens = std::env::var("MARKET_TOKENS")?;
    let token_pairs: Vec<(String, String)> = market_tokens
        .split(',')
        .filter_map(|pair| {
            let parts: Vec<&str> = pair.split(':').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect();

    info!("Starting market data capture for {} tokens", token_pairs.len());

    loop {
        for (symbol, address) in &token_pairs {
            match token_data_service.update_token_data(address, symbol).await {
                Ok(_) => info!("Successfully updated market data for {}", symbol),
                Err(e) => info!("Error updating market data for {}: {}", symbol, e),
            }
            // Small delay between tokens to respect rate limits
            time::sleep(Duration::from_millis(500)).await;
        }

        // Wait 5 minutes before next update
        info!("Waiting 5 minutes before next update");
        time::sleep(Duration::from_secs(300)).await;
    }
}

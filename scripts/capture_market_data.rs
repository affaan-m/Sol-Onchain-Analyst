use cainam_core::{
    config::mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig},
    services::TokenAnalyticsService,
    birdeye::api::BirdeyeClient,
    error::{AgentError, AgentResult},
};
use std::sync::Arc;
use std::env;
use tokio;
use tracing::{info, error, Level};
use dotenvy::dotenv;
use anyhow::Result;

const MARKET_TOKENS_ENV: &str = "MARKET_TOKENS";

#[tokio::main]
async fn main() -> AgentResult<()> {
    // Initialize tracing with a more visible format
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    println!("Starting market data capture...");

    // Load environment variables
    dotenv().ok();

    // Get MongoDB connection details
    let mongodb_uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let mongodb_database = env::var("MONGODB_DATABASE").expect("MONGODB_DATABASE must be set");

    // Initialize MongoDB connection
    let config = MongoConfig {
        uri: mongodb_uri.clone(),
        database: mongodb_database.clone(),
        app_name: Some("market-data-capture".to_string()),
        pool_config: MongoPoolConfig::default(),
    };

    let db_pool = MongoDbPool::create_pool(config).await.map_err(|e| AgentError::Other(e.into()))?;
    info!("Connected to MongoDB at {}", mongodb_uri);

    // Initialize Birdeye client
    let birdeye_api_key = env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY must be set");
    let birdeye_client = Arc::new(BirdeyeClient::new(birdeye_api_key));
    info!("Initialized Birdeye client");

    // Initialize analytics service
    let analytics_service = TokenAnalyticsService::new(db_pool, birdeye_client.clone(), None).await?;
    info!("Initialized analytics service");

    // Get market tokens from environment
    let market_tokens = env::var(MARKET_TOKENS_ENV)
        .expect("MARKET_TOKENS must be set")
        .split(',')
        .map(|pair| {
            let parts: Vec<&str> = pair.split(':').collect();
            if parts.len() != 2 {
                panic!("Invalid market token format. Expected FORMAT: SYMBOL:ADDRESS");
            }
            (parts[0].to_string(), parts[1].to_string())
        })
        .collect::<Vec<_>>();

    info!("Processing {} market tokens", market_tokens.len());

    // Process each token
    for (symbol, address) in market_tokens {
        info!("Processing token: {} ({})", symbol, address);
        
        match analytics_service.fetch_and_store_token_info(&symbol, &address).await {
            Ok(analytics) => {
                info!(
                    "Successfully captured data for {}: price=${:.4}, volume=${:.2}",
                    symbol,
                    analytics.price,
                    analytics.volume_24h.unwrap_or_default()
                );
            }
            Err(e) => {
                error!("Failed to capture data for {}: {}", symbol, e);
                // Continue with next token instead of stopping
                continue;
            }
        }
    }

    info!("Market data capture completed");
    Ok(())
}

use cainam_core::{
    config::mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig, TokenAnalyticsData},
    services::token_data_service::TokenDataService,
    birdeye::api::{BirdeyeApi, BirdeyeClient},
    error::{AgentError, AgentResult},
    models::trending_token::TrendingToken,
};
use mongodb::Collection;
use std::sync::Arc;
use std::env;
use tokio;
use tracing::{info, error, Level};
use dotenvy::dotenv;
use bson::Uuid;
use anyhow::Result;

const MARKET_TOKENS_ENV: &str = "MARKET_TOKENS";

#[tokio::main]
async fn main() -> Result<()> {
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

    let db_pool = MongoDbPool::create_pool(config).await?;
    info!("Connected to MongoDB at {}", mongodb_uri);

    // Initialize Birdeye client
    let birdeye_api_key = env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY must be set");
    let birdeye_client: Arc<dyn BirdeyeApi> = Arc::new(BirdeyeClient::new(birdeye_api_key.clone()));
    info!("Initialized Birdeye client");

    // Initialize token data service
    let token_service = TokenDataService::new_with_pool(db_pool.clone(), birdeye_api_key).await?;
    info!("Initialized token data service");

    // Get trending tokens
    info!("Fetching trending tokens...");
    let trending_response = birdeye_client.get_trending_tokens_full().await?;

    // Get MongoDB collection for trending tokens
    let db = db_pool.database("");
    let trending_collection: Collection<TrendingToken> = db.collection(TrendingToken::collection_name());

    // Store trending tokens with timestamp
    let timestamp = bson::DateTime::now();
    let tokens = trending_response.data.tokens.clone();
    for mut token in tokens {
        token.timestamp = Some(timestamp);
        if let Err(e) = trending_collection.insert_one(token).await {
            error!("Failed to store trending token: {}", e);
            continue;
        }
    }

    info!("Stored {} trending tokens", trending_response.data.tokens.len());

    // Process each trending token
    for token in trending_response.data.tokens {
        info!("Processing token: {} ({})", token.symbol, token.address);
        
        // Convert token data to TokenAnalyticsData
        let token_data = TokenAnalyticsData {
            id: Uuid::new().to_string(),
            token_address: token.address.clone(),
            token_name: token.name.clone(),
            token_symbol: token.symbol.clone(),
            price: token.price,
            volume_24h: Some(token.volume_24h_usd),
            market_cap: Some(token.marketcap),
            total_supply: None, // Not available in trending token data
            timestamp: token.timestamp.unwrap_or_else(bson::DateTime::now),
            created_at: Some(bson::DateTime::now()),
        };
        
        match token_service.store_token_data(token_data).await {
            Ok(_) => {
                info!(
                    "Successfully captured data for {}: price=${:.4}, volume=${:.2}",
                    token.symbol,
                    token.price,
                    token.volume_24h_usd
                );
            }
            Err(e) => {
                error!("Failed to capture data for {}: {}", token.symbol, e);
                continue;
            }
        }
    }

    info!("Market data capture completed");
    Ok(())
}

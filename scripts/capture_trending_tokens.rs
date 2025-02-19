use anyhow::{Context, Result};
use cainam_core::{
    birdeye::api::{BirdeyeApi, BirdeyeClient},
    config::mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig},
    models::trending_token::TrendingToken,
};
use dotenvy::dotenv;
use mongodb::bson::{doc, oid::ObjectId, DateTime};
use mongodb::IndexModel;
use std::sync::Arc;
use tokio;
use tracing::{info, Level};

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

    info!("Starting trending tokens capture...");

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
        app_name: Some("trending-tokens-capture".to_string()),
        pool_config: MongoPoolConfig::default(),
    };

    let db_pool = MongoDbPool::create_pool(config).await?;
        info!("Successfully connected to MongoDB");
    let db = db_pool.database(&mongodb_database);
        info!("Database: {}", db.name());

    // Initialize Birdeye client
    let birdeye_api_key = dotenvy::var("BIRDEYE_API_KEY").context("BIRDEYE_API_KEY must be set")?;

    let birdeye_client: Arc<dyn BirdeyeApi> = Arc::new(BirdeyeClient::new(birdeye_api_key));
    info!("Initialized Birdeye client");
    
    // Get trending tokens collection
    let trending_collection = db.collection::<TrendingToken>("trending_tokens");

    // Create compound index on address and timestamp
    let index = IndexModel::builder()
        .keys(doc! {
            "address": 1,
            "timestamp": -1
        })
        .build();

    trending_collection.create_index(index).await?;

    info!("Fetching trending tokens from Birdeye...");
    let trending_tokens = birdeye_client.get_trending_tokens().await?;
    let current_timestamp = DateTime::now();

    let mut tokens_stored = 0;
    for token in trending_tokens {
        // Add timestamp and id to token before storing
        let token_with_meta = TrendingToken {
            id: Some(ObjectId::new()),
            timestamp: Some(current_timestamp),
            address: token.address,
            decimals: token.decimals,
            liquidity: token.liquidity,
            logo_uri: token.logo_uri,
            name: token.name,
            symbol: token.symbol,
            volume_24h_usd: token.volume_24h_usd,
            rank: token.rank,
            price: token.price,
            volume_24h_change_percent: None,
            fdv: None,
            marketcap: None,
            price_24h_change_percent: None,
        };

        match trending_collection.insert_one(token_with_meta).await {
            Ok(_) => tokens_stored += 1,
            Err(e) => info!("Error inserting token: {}", e),
        }
    }

    info!("Successfully captured {} trending tokens", tokens_stored);
    Ok(())
}

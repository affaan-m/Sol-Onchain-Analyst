use cainam_core::{
    config::mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig, TokenAnalyticsData},
    services::token_data_service::TokenDataService,
    birdeye::api::{BirdeyeApi, BirdeyeClient},
    error::{AgentError, AgentResult},
    models::trending_token::TrendingToken,
};
use mongodb::{Collection, IndexModel, options::IndexOptions};
use mongodb::bson::{doc, Document};
use std::sync::Arc;
use std::env;
use tokio;
use tracing::{info, error, Level};
use dotenvy::dotenv;
use bson::Uuid;
use anyhow::{Result, Context};
use bson::oid::ObjectId;

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
    println!("Loaded environment variables");

    // Get MongoDB connection details
    let mongodb_uri = env::var("MONGODB_URI")
        .context("MONGODB_URI must be set")?;
    let mongodb_database = env::var("MONGODB_DATABASE")
        .context("MONGODB_DATABASE must be set")?;
    println!("Got MongoDB connection details: {}", mongodb_uri);

    // Initialize MongoDB connection
    let config = MongoConfig {
        uri: mongodb_uri.clone(),
        database: mongodb_database.clone(),
        app_name: Some("market-data-capture".to_string()),
        pool_config: MongoPoolConfig::default(),
    };

    println!("Connecting to MongoDB...");
    let db_pool = MongoDbPool::create_pool(config).await?;
    println!("Successfully connected to MongoDB");

    // Initialize Birdeye client
    let birdeye_api_key = env::var("BIRDEYE_API_KEY")
        .context("BIRDEYE_API_KEY must be set")?;
    println!("Got Birdeye API key");
    
    let birdeye_client: Arc<dyn BirdeyeApi> = Arc::new(BirdeyeClient::new(birdeye_api_key.clone()));
    info!("Initialized Birdeye client");

    // Initialize token data service
    println!("Initializing token data service...");
    let token_service = TokenDataService::new_with_pool(db_pool.clone(), birdeye_api_key).await
        .context("Failed to create token data service")?;
    info!("Initialized token data service");

    // Get database from pool (database name is already configured)
    let db = db_pool.database("");
    
    // Create trending tokens collection
    let trending_collection = db.collection::<TrendingToken>("trending_tokens");
    
    // Create index for trending tokens collection
    trending_collection.create_index(
        mongodb::IndexModel::builder()
            .keys(doc! { 
                "address": 1,
                "timestamp": -1 
            })
            .build()
    ).await?;

    println!("Fetching trending tokens from Birdeye...");
    let trending_response = birdeye_client.get_trending_tokens_full().await?;
    
    for token in trending_response.data.tokens {
        // Add timestamp and id to token before storing
        let token_with_meta = TrendingToken {
            id: Some(ObjectId::new()),
            timestamp: Some(bson::DateTime::now()),
            ..token
        };
        
        if let Err(e) = trending_collection.insert_one(token_with_meta).await {
            println!("Error inserting token: {}", e);
        }
    }

    println!("Successfully captured market data");
    Ok(())
}

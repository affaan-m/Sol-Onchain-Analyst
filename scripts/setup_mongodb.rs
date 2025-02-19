#![recursion_limit = "256"]

use anyhow::{Context, Result};
use cainam_core::config::mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig};
use dotenvy::dotenv;
use mongodb::bson::doc;
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

    info!("Starting MongoDB setup...");

    // Load environment variables
    dotenv().ok();

    // Get MongoDB connection details
    let mongodb_uri = dotenvy::var("MONGODB_URI")
        .context("MONGODB_URI must be set")?;
    let mongodb_database = dotenvy::var("MONGODB_DATABASE")
        .context("MONGODB_DATABASE must be set")?;

    info!("Connecting to MongoDB at: {}", mongodb_uri);

    // Initialize MongoDB connection
    let config = MongoConfig {
        uri: mongodb_uri,
        database: mongodb_database.clone(),
        app_name: Some("mongodb-setup".to_string()),
        pool_config: MongoPoolConfig::default(),
    };

    let db_pool = MongoDbPool::create_pool(config).await?;
    let db = db_pool.database(&mongodb_database);
    info!("Successfully connected to MongoDB");

    // Drop existing time series collections
    info!("Dropping existing time series collections...");
    match db.run_command(doc! { "drop": "trending_tokens" }).await {
        Ok(_) => info!("Dropped trending_tokens collection"),
        Err(e) => info!("Error dropping trending_tokens: {}", e),
    }
    match db.run_command(doc! { "drop": "token_analytics" }).await {
        Ok(_) => info!("Dropped token_analytics collection"),
        Err(e) => info!("Error dropping token_analytics: {}", e),
    }

    // Setup trending_tokens collection
    info!("Setting up trending_tokens collection...");
    match db
        .run_command(doc! {
            "create": "trending_tokens"
        })
        .await
    {
        Ok(_) => info!("Created trending_tokens collection"),
        Err(e) => info!("trending_tokens collection may already exist: {}", e),
    }

    // Create compound index for time-based queries
    match db
        .run_command(doc! {
            "createIndexes": "trending_tokens",
            "indexes": [{
                "key": { "timestamp": -1 },
                "name": "timestamp_desc"
            }]
        })
        .await
    {
        Ok(_) => info!("Created timestamp index for trending_tokens"),
        Err(e) => info!("Index may already exist: {}", e),
    }

    // Create search index for trending_tokens
    info!("Setting up search index for trending_tokens...");
    let trending_search_index = doc! {
        "mappings": {
            "dynamic": true,
            "fields": {
                "address": { "type": "string", "searchable": true },
                "decimals": { "type": "number" },
                "liquidity": { "type": "sortableNumberBetaV1" },
                "logo_uri": { "type": "string" },
                "name": { "type": "string", "searchable": true },
                "symbol": { "type": "token" },
                "volume_24h_usd": { "type": "sortableNumberBetaV1" },
                "volume_24h_change_percent": { "type": "number" },
                "fdv": { "type": "sortableNumberBetaV1" },
                "marketcap": { "type": "sortableNumberBetaV1" },
                "rank": { "type": "numberFacet" },
                "price": { "type": "sortableNumberBetaV1" },
                "price_24h_change_percent": { "type": "number" },
                "timestamp": { "type": "sortableDateBetaV1" }
            }
        }
    };

    match db
        .run_command(doc! {
            "createIndexes": "trending_tokens",
            "indexes": [{
                "key": { "address": "text", "name": "text", "symbol": "text" },
                "name": "default_search",
                "weights": {
                    "address": 10,
                    "name": 10,
                    "symbol": 5
                }
            }]
        })
        .await
    {
        Ok(_) => info!("Created search index for trending_tokens"),
        Err(e) => info!("Search index may already exist: {}", e),
    }

    // Setup token_analytics collection
    info!("Setting up token_analytics collection...");
    match db
        .run_command(doc! {
            "create": "token_analytics"
        })
        .await
    {
        Ok(_) => info!("Created token_analytics collection"),
        Err(e) => info!("token_analytics collection may already exist: {}", e),
    }

    // Create compound index for time-based queries
    match db
        .run_command(doc! {
            "createIndexes": "token_analytics",
            "indexes": [{
                "key": { "token_address": 1, "timestamp": -1 },
                "name": "token_time_desc"
            }]
        })
        .await
    {
        Ok(_) => info!("Created token_time index for token_analytics"),
        Err(e) => info!("Index may already exist: {}", e),
    }

    // Create search index for token_analytics
    info!("Setting up search index for token_analytics...");
    let analytics_search_index = doc! {
        "mappings": {
            "dynamic": true,
            "fields": {
                "token_address": { "type": "string", "searchable": true },
                "token_name": { "type": "string", "searchable": true },
                "token_symbol": { "type": "token" },
                "token_time": { "type": "sortableDateBetaV1" },
                "token_liquidity": { "type": "sortableNumberBetaV1" },
                "token_volume_24h_usd": { "type": "sortableNumberBetaV1" },
                "token_volume_24h_change_percent": { "type": "number" },
                "token_fdv": { "type": "sortableNumberBetaV1" },
                "token_marketcap": { "type": "sortableNumberBetaV1" },
                "token_rank": { "type": "numberFacet" },
                "token_price": { "type": "sortableNumberBetaV1" },
                "token_price_24h_change_percent": { "type": "number" }
            }
        }
    };

    match db
        .run_command(doc! {
            "createIndexes": "token_analytics",
            "indexes": [{
                "key": { "token_address": "text", "token_name": "text", "token_symbol": "text" },
                "name": "default_search",
                "weights": {
                    "token_address": 10,
                    "token_name": 10,
                    "token_symbol": 5
                }
            }]
        })
        .await
    {
        Ok(_) => info!("Created search index for token_analytics"),
        Err(e) => info!("Search index may already exist: {}", e),
    }

    // Setup market_signals collection
    info!("Setting up market_signals collection...");
    match db
        .run_command(doc! {
            "create": "market_signals"
        })
        .await
    {
        Ok(_) => info!("Created market_signals collection"),
        Err(e) => info!("market_signals collection may already exist: {}", e),
    }

    // Create search index for market_signals
    info!("Setting up search index for market_signals...");
    match db
        .run_command(doc! {
            "createIndexes": "market_signals",
            "indexes": [{
                "key": { "token_address": "text", "signal_type": "text" },
                "name": "default_search",
                "weights": {
                    "token_address": 10,
                    "signal_type": 5
                }
            }]
        })
        .await
    {
        Ok(_) => info!("Created search index for market_signals"),
        Err(e) => info!("Search index may already exist: {}", e),
    }

    // Setup trading_positions collection
    info!("Setting up trading_positions collection...");
    match db
        .run_command(doc! {
            "create": "trading_positions"
        })
        .await
    {
        Ok(_) => info!("Created trading_positions collection"),
        Err(e) => info!("trading_positions collection may already exist: {}", e),
    }

    // Create search index for trading_positions
    info!("Setting up search index for trading_positions...");
    match db
        .run_command(doc! {
            "createIndexes": "trading_positions",
            "indexes": [{
                "key": { "token_address": "text", "position_type": "text", "status": "text" },
                "name": "default_search",
                "weights": {
                    "token_address": 10,
                    "position_type": 5,
                    "status": 5
                }
            }]
        })
        .await
    {
        Ok(_) => info!("Created search index for trading_positions"),
        Err(e) => info!("Search index may already exist: {}", e),
    }

    info!("MongoDB setup completed successfully!");
    Ok(())
}

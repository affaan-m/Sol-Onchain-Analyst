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
    let mongodb_uri = dotenvy::var("MONGODB_URI").context("MONGODB_URI must be set")?;
    let mongodb_database =
        dotenvy::var("MONGODB_DATABASE").context("MONGODB_DATABASE must be set")?;

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
    match db.run_command(doc! { "drop": "token_trending" }).await {
        Ok(_) => info!("Dropped token_trending collection"),
        Err(e) => info!("Error dropping token_trending: {}", e),
    }
    match db.run_command(doc! { "drop": "token_analytics" }).await {
        Ok(_) => info!("Dropped token_analytics collection"),
        Err(e) => info!("Error dropping token_analytics: {}", e),
    }

    // Setup token_trending collection
    info!("Setting up token_trending collection...");
    match db
        .run_command(doc! {
            "create": "token_trending"
        })
        .await
    {
        Ok(_) => info!("Created token_trending collection"),
        Err(e) => info!("token_trending collection may already exist: {}", e),
    }

    // Create compound index for time-based queries
    match db
        .run_command(doc! {
            "createIndexes": "token_trending",
            "indexes": [{
                "key": { "timestamp": -1 },
                "name": "timestamp_desc"
            }]
        })
        .await
    {
        Ok(_) => info!("Created timestamp index for token_trending"),
        Err(e) => info!("Index may already exist: {}", e),
    }

    // Create search index for token_trending
    info!("Setting up search index for token_trending...");
    match db
        .run_command(doc! {
            "createSearchIndexes": "token_trending",
            "indexes": [{
                "name": "token_trending_index",
                "definition": {
                    "mappings": {
                        "dynamic": true,
                        "fields": {
                            "address": { "type": "string" },
                            "decimals": { "type": "number" },
                            "liquidity": { "type": "number" },
                            "logo_uri": { "type": "string" },
                            "name": { "type": "string" },
                            "symbol": { "type": "string" },
                            "volume_24h_usd": { "type": "number" },
                            "volume_24h_change_percent": { "type": "number" },
                            "fdv": { "type": "number" },
                            "marketcap": { "type": "number" },
                            "rank": { "type": "number" },
                            "price": { "type": "number" },
                            "price_24h_change_percent": { "type": "number" },
                            "timestamp": { "type": "date" }
                        }
                    }
                }
            }]
        })
        .await
    {
        Ok(_) => info!("Created search index for token_trending"),
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
    match db
        .run_command(doc! {
            "createSearchIndexes": "token_analytics",
            "indexes": [{
                "name": "token_analytics_index",
                "definition": {
                    "mappings": {
                        "dynamic": true,
                        "fields": {
                            "bollinger_lower": { "type": "number" },
                            "bollinger_upper": { "type": "number" },
                            "fully_diluted_market_cap": { "type": "number" },
                            "liquidity": { "type": "number" },
                            "macd": { "type": "number" },
                            "macd_signal": { "type": "number" },
                            "market_cap": { "type": "number" },
                            "price": { "type": "number" },
                            "price_change_24h": { "type": "number" },
                            "rsi_14": { "type": "number" },
                            "timestamp": { "type": "date" },
                            "token_address": { "type": "string"},
                            "token_name": { "type": "string"},
                            "token_symbol": { "type": "string"},
                            "volume_24h": { "type": "number" },
                            "volume_change_24h": { "type": "number" }
                        }
                    }
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
            "createSearchIndexes": "market_signals",
            "indexes": [{
                "name": "market_signals_index",
                "definition": {
                    "mappings": {
                        "dynamic": true,
                        "fields": {
                            "token_address": { "type": "string"},
                            "signal_type": { "type": "string"},
                            "timestamp": { "type": "date" },
                            "price": { "type": "number" },
                            "price_change_24h": { "type": "number" },
                            "volume_change_24h": { "type": "number" },
                            "confidence": { "type": "number" },
                            "risk_score": { "type": "number" }
                        }
                    }
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
            "createSearchIndexes": "trading_positions",
            "indexes": [{
                "name": "trading_positions_index",
                "definition": {
                    "mappings": {
                        "dynamic": true,
                        "fields": {
                            "token_address": { "type": "string"},
                            "position_type": { "type": "string"},
                            "status": { "type": "string"},
                            "timestamp": { "type": "date" },
                            "entry_price": { "type": "number" },
                            "current_price": { "type": "number" },
                            "size": { "type": "number" },
                            "pnl": { "type": "number" }
                        }
                    }
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

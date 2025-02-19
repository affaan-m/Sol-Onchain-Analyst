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

    // Setup trending_tokens collection
    info!("Setting up trending_tokens collection...");
    let trending_options = doc! {
            "timeField": "timestamp",
            "granularity": "minutes"
    };

    match db
        .run_command(doc! {
            "create": "trending_tokens",
            "timeseries": trending_options
        })
        .await
    {
        Ok(_) => info!("Created trending_tokens collection"),
        Err(e) => info!("trending_tokens collection may already exist: {}", e),
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
            "createSearchIndex": "trending_tokens",
            "definition": trending_search_index
        })
        .await
    {
        Ok(_) => info!("Created search index for trending_tokens"),
        Err(e) => info!("Search index may already exist: {}", e),
    }

    // Setup token_analytics collection
    info!("Setting up token_analytics collection...");
    let analytics_options = doc! {
            "timeField": "timestamp",
            "granularity": "minutes"
    };

    match db
        .run_command(doc! {
            "create": "token_analytics",
            "timeseries": analytics_options
        })
        .await
    {
        Ok(_) => info!("Created token_analytics collection"),
        Err(e) => info!("token_analytics collection may already exist: {}", e),
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
                "decimals": { "type": "number" },
                "logo_uri": { "type": "string" },

                // Price metrics (stored as Decimal128)
                "price": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "price", "type": "decimal128" }
                },
                "price_change_24h": { "type": "sortableNumberBetaV1" },
                "price_change_7d": { "type": "sortableNumberBetaV1" },

                // Volume metrics (stored as Decimal128)
                "volume_24h": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "volume_24h", "type": "decimal128" }
                },
                "volume_change_24h": { "type": "sortableNumberBetaV1" },
                "volume_by_price_24h": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "volume_by_price_24h", "type": "decimal128" }
                },

                // Market metrics (stored as Decimal128)
                "market_cap": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "market_cap", "type": "decimal128" }
                },
                "fully_diluted_market_cap": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "fully_diluted_market_cap", "type": "decimal128" }
                },
                "circulating_supply": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "circulating_supply", "type": "decimal128" }
                },
                "total_supply": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "total_supply", "type": "decimal128" }
                },

                // Trading metrics
                "trades_24h": { "type": "numberFacet" },
                "average_trade_size": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "average_trade_size", "type": "decimal128" }
                },

                // Holder metrics
                "holder_count": { "type": "numberFacet" },
                "active_wallets_24h": { "type": "numberFacet" },
                "whale_transactions_24h": { "type": "numberFacet" },

                // Technical indicators (stored as Decimal128)
                "rsi_14": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "rsi_14", "type": "decimal128" }
                },
                "macd": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "macd", "type": "decimal128" }
                },
                "macd_signal": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "macd_signal", "type": "decimal128" }
                },
                "bollinger_upper": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "bollinger_upper", "type": "decimal128" }
                },
                "bollinger_lower": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "bollinger_lower", "type": "decimal128" }
                },

                // Social metrics
                "social_score": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "social_score", "type": "decimal128" }
                },
                "social_volume": { "type": "numberFacet" },
                "social_sentiment": {
                    "type": "sortableNumberBetaV1",
                    "path": { "value": "social_sentiment", "type": "decimal128" }
                },
                "dev_activity": { "type": "numberFacet" },

                // Timestamps
                "timestamp": { "type": "sortableDateBetaV1" },
                "created_at": { "type": "date" },
                "last_trade_time": { "type": "date" },

                // Vector search
                "embedding": {
                    "type": "knnVector",
                    "dimensions": 1536,
                    "similarity": "cosine"
                }
            }
        }
    };

    match db
        .run_command(doc! {
            "createSearchIndex": "token_analytics",
            "definition": analytics_search_index
        })
        .await
    {
        Ok(_) => info!("Created search index for token_analytics"),
        Err(e) => info!("Search index may already exist: {}", e),
    }

    // Setup market_signals collection
    info!("Setting up market_signals collection...");
    let signals_options = doc! {
        "timeseries": {
            "timeField": "timestamp",
            "granularity": "minutes",
            "metaField": "metadata"
        }
    };

    match db
        .run_command(doc! {
            "create": "market_signals",
            "timeseries": signals_options
        })
        .await
    {
        Ok(_) => info!("Created market_signals collection"),
        Err(e) => info!("market_signals collection may already exist: {}", e),
    }

    // Create search index for market_signals
    info!("Setting up search index for market_signals...");
    let signals_search_index = doc! {
        "mappings": {
            "dynamic": true,
            "fields": {
                "token_address": { "type": "string", "searchable": true },
                "signal_type": { "type": "stringFacet" },
                "confidence": { "type": "sortableNumberBetaV1" },
                "risk_score": { "type": "sortableNumberBetaV1" },
                "price": { "type": "sortableNumberBetaV1" },
                "volume_change": { "type": "sortableNumberBetaV1" },
                "timestamp": { "type": "sortableDateBetaV1" },
                "metadata": { "type": "document" }
            }
        }
    };

    match db
        .run_command(doc! {
            "createSearchIndex": "market_signals",
            "definition": signals_search_index
        })
        .await
    {
        Ok(_) => info!("Created search index for market_signals"),
        Err(e) => info!("Search index may already exist: {}", e),
    }

    // Setup trading_positions collection
    info!("Setting up trading_positions collection...");
    let positions_options = doc! {
        "timeseries": {
            "timeField": "entry_time",
            "granularity": "minutes",
            "metaField": "metadata"
        }
    };

    match db
        .run_command(doc! {
            "create": "trading_positions",
            "timeseries": positions_options
        })
        .await
    {
        Ok(_) => info!("Created trading_positions collection"),
        Err(e) => info!("trading_positions collection may already exist: {}", e),
    }

    // Create search index for trading_positions
    info!("Setting up search index for trading_positions...");
    let positions_search_index = doc! {
        "mappings": {
            "dynamic": true,
            "fields": {
                "token_address": { "type": "string", "searchable": true },
                "entry_price": { "type": "sortableNumberBetaV1" },
                "current_price": { "type": "sortableNumberBetaV1" },
                "position_size": { "type": "sortableNumberBetaV1" },
                "position_type": { "type": "stringFacet" },
                "entry_time": { "type": "sortableDateBetaV1" },
                "last_update": { "type": "date" },
                "pnl": { "type": "sortableNumberBetaV1" },
                "status": { "type": "stringFacet" }
            }
        }
    };

    match db
        .run_command(doc! {
            "createSearchIndex": "trading_positions",
            "definition": positions_search_index
        })
        .await
    {
        Ok(_) => info!("Created search index for trading_positions"),
        Err(e) => info!("Search index may already exist: {}", e),
    }

    info!("MongoDB setup completed successfully!");
    Ok(())
}

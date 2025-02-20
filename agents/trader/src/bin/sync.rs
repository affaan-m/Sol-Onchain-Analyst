//! Market Data Synchronization Service
//!
//! This binary runs a service that continuously synchronizes market data from various sources
//! (primarily BirdEye) into MongoDB for analysis and trading decisions. It handles:
//!
//! - Fetching trending tokens at configurable intervals
//! - Storing token states with price, volume, and market data
//! - Detailed logging of all operations for monitoring
//! - Graceful shutdown on Ctrl+C
//!
//! # Configuration
//! The service is configured through environment variables:
//! - `MONGODB_URI`: MongoDB connection string (default: mongodb://localhost:32770)
//! - `BIRDEYE_API_KEY`: API key for BirdEye data
//! - `DATA_SYNC_INTERVAL_SECONDS`: Interval between syncs (default: 60)
//! - `RUST_LOG`: Logging level configuration
//!
//! # Usage
//! ```bash
//! cargo run --bin sync
//! ```

use crate::config::mongodb::MongoConfig;
use crate::config::pool::MongoPoolConfig;
use anyhow::Result;
use chrono::Utc;
use dotenvy::dotenv;
use mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig};
use rig_solana_trader::{
    database::DatabaseClient,
    market_data::{
        birdeye::BirdEyeProvider, AggregatedDataProvider, DataProvider, MarketTrend, TokenMetadata,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenState {
    address: String,
    symbol: String,
    name: String,
    price_usd: f64,
    price_sol: f64,
    volume_24h: f64,
    market_cap: f64,
    price_change_24h: f64,
    volume_change_24h: f64,
    timestamp: chrono::DateTime<Utc>,
}

struct DataSyncService {
    data_provider: Arc<AggregatedDataProvider>,
    db: Arc<DatabaseClient>,
}

impl DataSyncService {
    #[instrument]
    fn new(data_provider: Arc<AggregatedDataProvider>, db: Arc<DatabaseClient>) -> Self {
        info!("Creating new DataSyncService instance");
        let service = Self { data_provider, db };

        service.start_sync_tasks();
        info!("DataSyncService initialized successfully");
        service
    }

    #[instrument(skip(self))]
    fn start_sync_tasks(&self) {
        let data_provider = Arc::clone(&self.data_provider);
        let db = Arc::clone(&self.db);

        info!("Starting market data sync task");
        tokio::spawn(async move {
            loop {
                info!("Beginning new market data sync cycle");
                debug!("Fetching trending tokens from data provider");

                match data_provider.as_ref().get_token_trending(100).await {
                    Ok(trends) => {
                        info!(
                            token_count = trends.len(),
                            "Successfully fetched trending tokens"
                        );

                        for trend in trends {
                            debug!(
                                token.address = %trend.token_address,
                                token.symbol = %trend.metadata.symbol,
                                token.name = %trend.metadata.name,
                                token.price_usd = trend.metadata.price_usd,
                                token.volume_24h = trend.metadata.volume_24h,
                                token.price_change_24h = trend.price_change_24h,
                                "Processing token data"
                            );

                            let token_state = TokenState {
                                address: trend.token_address.clone(),
                                symbol: trend.metadata.symbol.clone(),
                                name: trend.metadata.name.clone(),
                                price_usd: trend.metadata.price_usd,
                                price_sol: trend.metadata.price_sol,
                                volume_24h: trend.metadata.volume_24h,
                                market_cap: trend.metadata.market_cap,
                                price_change_24h: trend.price_change_24h,
                                volume_change_24h: trend.volume_change_24h,
                                timestamp: Utc::now(),
                            };

                            debug!(
                                token.symbol = %token_state.symbol,
                                token.price_usd = token_state.price_usd,
                                token.volume_24h = token_state.volume_24h,
                                "Inserting token state into MongoDB"
                            );

                            match db.insert_one("token_states", &token_state).await {
                                Ok(_) => info!(
                                    token.symbol = %token_state.symbol,
                                    token.price_usd = token_state.price_usd,
                                    token.volume_24h = token_state.volume_24h,
                                    token.price_change_24h = token_state.price_change_24h,
                                    "Successfully stored token state"
                                ),
                                Err(e) => error!(
                                    token.symbol = %token_state.symbol,
                                    error = %e,
                                    "Failed to insert token state"
                                ),
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            error = %e,
                            "Failed to fetch trending tokens"
                        );
                    }
                }

                info!("Market data sync cycle complete");
                debug!("Sleeping for 60 seconds before next sync cycle");
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // Initialize MongoDB with custom sync configuration
    let config = MongoConfig {
        database: "solana_trades".to_string(),
        pool: MongoPoolConfig {
            min_pool_size: 2,
            max_pool_size: 5,
            connect_timeout: std::time::Duration::from_secs(30),
        },
        ..Default::default()
    };

    info!("Connecting to MongoDB at {}", config.uri);
    let pool = config.create_pool().await?;
    info!("Successfully connected to MongoDB");

    // Initialize collections with proper schemas
    let db = pool.database(&config.database);

    // Setup collections for trade sync
    db.create_collection(
        "token_states",
        Some(doc! {
            "timeseries": {
                "timeField": "timestamp",
                "metaField": "token_address",
                "granularity": "minutes"
            }
        }),
    )
    .await?;

    db.collection("token_states")
        .create_index(
            doc! {
                "token_address": 1,
                "timestamp": -1
            },
            None,
        )
        .await?;

    // Start sync process
    sync_tokens(pool).await?;

    Ok(())
}

async fn sync_tokens(pool: Arc<MongoDbPool>) -> Result<()> {
    // ...existing sync code...
}

//! Database Module
//!
//! This module handles all MongoDB interactions for the trading bot. It manages:
//! - Market data storage and retrieval
//! - Trade history
//! - Position tracking
//! - Risk model persistence
//! - Sentiment analysis data
//!
//! # Environment Variables
//! Required environment variables:
//! - `DATABASE_URL`: MongoDB connection string
//!
//! # Example
//! ```no_run
//! use rig_solana_trader::database::DatabaseClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = DatabaseClient::new("mongodb://user:pass@localhost/db", "trading_db").await?;
//!     Ok(())
//! }
//! ```

use mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig};
use std::sync::Arc;
use anyhow::Result;
use tracing::{debug, info};

pub mod positions;
pub mod sync;

/// Database client for interacting with MongoDB
pub struct DatabaseClient {
    pool: Arc<MongoDbPool>,
    database: String,
}

impl DatabaseClient {
    /// Create a new database client
    pub async fn new(uri: &str, database: &str) -> Result<Arc<Self>> {
        debug!("Initializing MongoDB client");

        let config = MongoConfig {
            uri: uri.to_string(),
            database: database.to_string(),
            ..Default::default()
        };

        let pool = config.create_pool().await?;
        
        // Initialize collections and indexes
        info!("Initializing MongoDB collections and indexes...");
        Self::init_collections(&pool, database).await?;

        info!("MongoDB client initialized successfully");
        Ok(Arc::new(Self {
            pool,
            database: database.to_string(),
        }))
    }
    
    async fn init_collections(pool: &MongoDbPool, database: &str) -> Result<()> {
        let db = pool.database(database);
        
        // Create token states collection with timeseries
        db.create_collection("token_states", Some(doc! {
            "timeseries": {
                "timeField": "timestamp",
                "metaField": "address",
                "granularity": "minutes"
            }
        })).await?;
        
        // Create index for efficient queries
        db.collection("token_states").create_index(
            doc! {
                "address": 1,
                "timestamp": -1
            },
            None,
        ).await?;
        
        Ok(())
    }
    
    /// Get the database pool
    pub fn pool(&self) -> Arc<MongoDbPool> {
        self.pool.clone()
    }
    
    /// Get the database name
    pub fn database(&self) -> &str {
        &self.database
    }
}
//! Database Module
//!
//! This module handles all PostgreSQL interactions for the trading bot. It manages:
//! - Market data storage and retrieval
//! - Trade history
//! - Position tracking
//! - Risk model persistence
//! - Sentiment analysis data
//!
//! # Environment Variables
//! Required environment variables:
//! - `DATABASE_URL`: PostgreSQL connection string
//!
//! # Example
//! ```no_run
//! use rig_solana_trader::database::DatabaseClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = DatabaseClient::new("postgresql://user:pass@localhost/db").await?;
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use serde::{Serialize, Deserialize};
use tracing::{debug, info};
use uuid::Uuid;
use anyhow::Result;

pub mod positions;
pub mod sync;

/// Database client for interacting with PostgreSQL
pub struct DatabaseClient {
    pool: Pool<Postgres>,
}

impl DatabaseClient {
    /// Create a new database client
    pub async fn new(database_url: &str) -> Result<Arc<Self>> {
        debug!("Initializing PostgreSQL client");

        let pool = PgPoolOptions::new()
            .max_connections(50)
            .idle_timeout(std::time::Duration::from_secs(5))
            .connect(database_url)
            .await?;

        info!("PostgreSQL client initialized successfully");

        // Initialize collections and indexes
        info!("Initializing PostgreSQL tables and indexes...");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        Ok(Arc::new(Self { pool }))
    }

    /// Insert a document into a collection
    pub async fn insert_document<T: Serialize>(&self, table: &str, document: &T) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let json = serde_json::to_value(document)?;

        sqlx::query!(
            "INSERT INTO $1 (id, document) VALUES ($2, $3)",
            table,
            id,
            json
        )
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    /// Find documents in a collection
    pub async fn find_documents<T: for<'de> Deserialize<'de>>(
        &self,
        table: &str,
        query: &str,
    ) -> Result<Vec<T>> {
        let rows = sqlx::query!(
            "SELECT document FROM $1 WHERE document @> $2::jsonb",
            table,
            query
        )
        .fetch_all(&self.pool)
        .await?;

        let documents = rows
            .into_iter()
            .map(|row| serde_json::from_value(row.document))
            .collect::<Result<Vec<T>, _>>()?;

        Ok(documents)
    }

    /// Update a document in a collection
    pub async fn update_document<T: Serialize>(
        &self,
        table: &str,
        id: Uuid,
        document: &T,
    ) -> Result<bool> {
        let json = serde_json::to_value(document)?;

        let result = sqlx::query!(
            "UPDATE $1 SET document = $2 WHERE id = $3",
            table,
            json,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete a document from a collection
    pub async fn delete_document(&self, table: &str, id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM $1 WHERE id = $2",
            table,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get the database pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
} 
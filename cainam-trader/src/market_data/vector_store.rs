use rig_core::{
    embeddings::EmbeddingsBuilder,
    providers::openai::{Client, TEXT_EMBEDDING_ADA_002},
    vector_store::{in_memory_store::InMemoryVectorStore, VectorStoreIndex},
    Embed,
};
use rig_postgres::PostgresVectorStore;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, debug};
use crate::database::DatabaseClient;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalysis {
    pub id: Uuid,
    pub token_address: String,
    pub sentiment_score: f64,
    pub technical_score: f64,
    pub risk_score: f64,
    pub symbol: String,
    pub description: String,
    pub recent_events: Vec<String>,
    pub market_sentiment: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct TokenVectorStore {
    store: PostgresVectorStore,
}

impl TokenVectorStore {
    pub fn new(pool: Pool<Postgres>) -> Self {
        // Initialize OpenAI client for embeddings
        let openai_client = rig_core::providers::openai::Client::from_env();
        let model = openai_client.embedding_model(rig_core::providers::openai::TEXT_EMBEDDING_3_SMALL);

        // Initialize PostgreSQL vector store
        let store = PostgresVectorStore::with_defaults(model, pool);

        Self { store }
    }

    pub async fn add_analysis(&self, analysis: TokenAnalysis, embeddings: Embeddings) -> Result<()> {
        info!("Saving token analysis to vector store");
        self.store.insert_document(&analysis, embeddings.embeddings[0].clone()).await?;
        Ok(())
    }

    pub async fn search_similar(&self, query: &str, limit: usize) -> Result<Vec<TokenAnalysis>> {
        info!("Searching for similar tokens");
        let results = self.store.top_n::<TokenAnalysis>(query, limit).await?;
        info!("Found {} similar tokens", results.len());
        Ok(results.into_iter().map(|(_, _, doc)| doc).collect())
    }

    pub async fn get_analysis(&self, token_address: &str) -> Result<Option<TokenAnalysis>> {
        let query = format!("token_address = '{}'", token_address);
        let results = self.store.find_documents::<TokenAnalysis>(&query).await?;
        Ok(results.into_iter().next())
    }
} 
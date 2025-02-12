use anyhow::Result;
use cainam_core::config::mongodb::MongoConfig;
use cainam_core::config::mongodb::{MongoDbPool, TokenAnalyticsData, TokenAnalyticsDataExt};
use mongodb::bson::doc;
use rig::providers::openai::{Client as OpenAiClient, EmbeddingModel, TEXT_EMBEDDING_3_SMALL};
use std::env;
use tracing::info;
use tracing_subscriber::fmt;
use rig::embeddings::embed::{Embed, TextEmbedder, EmbedError};

// Add a local wrapper for TokenAnalyticsData to bypass the orphan rule.
#[derive(serde::Serialize)]
struct WrappedTokenAnalyticsData(TokenAnalyticsData);

impl From<TokenAnalyticsData> for WrappedTokenAnalyticsData {
    fn from(data: TokenAnalyticsData) -> Self {
        WrappedTokenAnalyticsData(data)
    }
}

impl Embed for WrappedTokenAnalyticsData {
    fn embed(&self, embedder: &mut TextEmbedder) -> Result<(), EmbedError> {
        let text = format!(
            "Name: {}, Symbol: {}, Address: {}",
            self.0.token_name, self.0.token_symbol, self.0.token_address
        );
        embedder.embed(text);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (with file and line numbers for easier debugging)
    fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    info!("Starting vector search test...");

    // Initialize MongoDB connection using the configuration from the environment.
    let config = MongoConfig::from_env();
    let pool = MongoDbPool::create_pool(config).await?;

    // Clear the collection before inserting test data
    pool.client()
        .database("cainam")
        .collection::<TokenAnalyticsData>("token_analytics")
        .delete_many(doc! {})
        .await?;

    // Initialize the OpenAI client and create an embedding model using TEXT_EMBEDDING_3_SMALL.
    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let openai_client = OpenAiClient::new(&openai_api_key);
    let embedding_model = openai_client.embedding_model(TEXT_EMBEDDING_3_SMALL);

    // Define sample test token data.
    // Here we leave the embedding vector empty so that insert_token_analytics_documents can generate it.
    let test_tokens = vec![
        WrappedTokenAnalyticsData(TokenAnalyticsData {
            id: "1".to_string(),
            token_address: "So11111111111111111111111111111111111111112".to_string(),
            token_name: "Wrapped SOL".to_string(),
            token_symbol: "SOL".to_string(),
            embedding: vec![],
        }),
        WrappedTokenAnalyticsData(TokenAnalyticsData {
            id: "2".to_string(),
            token_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            token_name: "USD Coin".to_string(),
            token_symbol: "USDC".to_string(),
            embedding: vec![],
        }),
        WrappedTokenAnalyticsData(TokenAnalyticsData {
            id: "3".to_string(),
            token_address: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
            token_name: "Bonk".to_string(),
            token_symbol: "BONK".to_string(),
            embedding: vec![],
        }),
    ];

    // Insert test token documents.
    // Because the trait is implemented for MongoDbPool, and pool is an Arc<MongoDbPool>,
    // we call the methods on &*pool (which dereferences the Arc).
    info!("Inserting test token data...");
    (&*pool)
        .insert_token_analytics_documents("token_analytics", embedding_model.clone(), test_tokens)
        .await?;
    info!("Test data inserted successfully");

    // Define the vector search queries.
    let test_queries = vec![
        "Find me a stablecoin",
        "What's the native token of Solana",
        "Show me a meme token",
    ];

    // Execute each query and print search results.
    for query in test_queries {
        info!("Testing search with query: {}", query);
        let results = (&*pool)
            .top_n("token_analytics", embedding_model.clone(), query, 2)
            .await?;
        info!("Search results for '{}': {:#?}", query, results);
    }

    info!("Vector search test completed successfully");
    Ok(())
} 
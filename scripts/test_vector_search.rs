use anyhow::Result;
use cainam_core::config::mongodb::MongoConfig;
use cainam_core::config::mongodb::{MongoDbPool, TokenAnalyticsData};
use rig::providers::openai::{Client as OpenAiClient, EmbeddingModel, TEXT_EMBEDDING_3_SMALL};
use std::env;
use tracing::info;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();
    info!("Starting vector search test...");

    // Initialize MongoDB connection
    let config = MongoConfig::from_env();
    let pool = MongoDbPool::create_pool(config).await?;

    // Initialize OpenAI client
    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let openai_client = OpenAiClient::new(&openai_api_key);
    let embedding_model = EmbeddingModel::new(openai_client, TEXT_EMBEDDING_3_SMALL, 1536);

    // Sample test data
    let test_tokens = vec![
        TokenAnalyticsData {
            id: "1".to_string(),
            token_address: "So11111111111111111111111111111111111111112".to_string(),
            token_name: "Wrapped SOL".to_string(),
            token_symbol: "SOL".to_string(),
        },
        TokenAnalyticsData {
            id: "2".to_string(),
            token_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            token_name: "USD Coin".to_string(),
            token_symbol: "USDC".to_string(),
        },
        TokenAnalyticsData {
            id: "3".to_string(),
            token_address: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
            token_name: "Bonk".to_string(),
            token_symbol: "BONK".to_string(),
        },
    ];

    // Insert test data
    info!("Inserting test token data...");
    pool.insert_token_analytics_documents::<TokenAnalyticsData>(
        "token_analytics",
        embedding_model.clone(),
        test_tokens,
    )
    .await?;
    info!("Test data inserted successfully");

    // Test vector search with different queries
    let test_queries = vec![
        "Find me a stablecoin",
        "What's the native token of Solana",
        "Show me a meme token",
    ];

    for query in test_queries {
        info!("Testing search with query: {}", query);
        let results = pool
            .top_n(
                "token_analytics",
                embedding_model.clone(),
                query,
                2,
            )
            .await?;

        info!("Search results for '{}': {:#?}", query, results);
    }

    info!("Vector search test completed successfully");
    Ok(())
} 
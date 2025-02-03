use anyhow::Result;
use chrono::Utc;
use rig_solana_trader::{
    market_data::vector_store::{TokenAnalysis, TokenVectorStore},
    database::DatabaseClient,
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize PostgreSQL connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(std::time::Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    // Initialize vector store
    let vector_store = TokenVectorStore::new(pool);

    // Create test analysis
    let analysis = TokenAnalysis {
        id: Uuid::new_v4(),
        token_address: "So11111111111111111111111111111111111111112".to_string(),
        symbol: "SOL".to_string(),
        description: "Solana's native token".to_string(),
        recent_events: vec![
            "Network upgrade successful".to_string(),
            "New DeFi protocol launched".to_string(),
        ],
        market_sentiment: "Bullish".to_string(),
        timestamp: Utc::now(),
    };

    // Generate embeddings
    let openai_client = rig_core::providers::openai::Client::from_env();
    let model = openai_client.embedding_model(rig_core::providers::openai::TEXT_EMBEDDING_3_SMALL);
    let embeddings = rig_core::embeddings::EmbeddingsBuilder::new(model)
        .documents(vec![analysis.clone()])?
        .build()
        .await?;

    // Add analysis to vector store
    vector_store.add_analysis(analysis, embeddings).await?;

    // Search for similar tokens
    let similar_tokens = vector_store
        .search_similar("high performance blockchain token", 5)
        .await?;

    println!("Found {} similar tokens:", similar_tokens.len());
    for token in similar_tokens {
        println!("- {} ({})", token.symbol, token.market_sentiment);
    }

    Ok(())
} 
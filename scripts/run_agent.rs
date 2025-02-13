use anyhow::Result;
use cainam_core::{
    agent::trader::TradingAgent,
    config::mongodb::MongoDbPool,
    config::{mongodb::MongoConfig, AgentConfig},
    trading::SolanaAgentKit,
};
use dotenvy::dotenv;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Cainam Trading Agent...");

    // Load configuration
    let agent_config = AgentConfig::new_from_env()?;
    let mongo_config = MongoConfig::from_env();

    // Initialize MongoDB connection
    let db = MongoDbPool::create_pool(mongo_config).await?;

    // Initialize Solana agent kit
    let solana_agent = SolanaAgentKit::new_from_env()?;

    // Create and run the trading agent
    let agent = TradingAgent::new(agent_config, db, solana_agent).await?;

    info!("Agent initialized successfully. Starting main loop...");
    agent.run().await?;

    Ok(())
}

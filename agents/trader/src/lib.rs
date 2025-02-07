//! Solana Trading Bot
//!
//! This crate provides a framework for building automated trading bots on Solana.
//! It includes:
//!
//! - Market data collection and analysis
//! - Trading strategy implementation
//! - Risk management
//! - Trade execution via Jupiter
//! - Twitter integration for trade announcements
//! - PostgreSQL persistence for market data and positions
//!
//! # Architecture
//!
//! The bot is organized into several key modules:
//!
//! - `market_data`: Handles market data collection and analysis
//! - `strategy`: Implements trading strategies
//! - `execution`: Manages trade execution
//! - `database`: Handles PostgreSQL persistence
//! - `twitter`: Twitter API integration
//!
//! # Example Usage
//!
//! ```no_run
//! use rig_solana_trader::{TradingBot, Config};
//! use std::env;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Load configuration
//!     let config = Config::from_env()?;
//!
//!     // Create and start bot
//!     let mut bot = TradingBot::new(config).await?;
//!     bot.run().await?;
//!
//!     Ok(())
//! }
//! ```

use rig_core::{
    agent::{Agent, AgentSystem},
    message_bus::MessageBus,
};
use rig_postgres::PostgresVectorStore;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::env;
use std::time::Duration;
use tracing::{debug, info};

pub mod agents;
pub mod analysis;
pub mod database;
pub mod decision;
pub mod dex;
pub mod execution;
pub mod integrations;
pub mod market_data;
pub mod personality;
pub mod prediction;
pub mod state;
pub mod storage;
pub mod strategy;
pub mod twitter;
pub mod wallet;

/// Initialize the trading bot with the given configuration
pub async fn init_bot(
    database_url: &str,
    openai_api_key: &str,
    twitter_api_key: &str,
) -> anyhow::Result<()> {
    // Initialize PostgreSQL connection
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .idle_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .map_err(|e| {
            debug!("PostgreSQL connection error: {:?}", e);
            anyhow::anyhow!("Failed to connect to PostgreSQL")
        })?;

    info!("PostgreSQL connection established");

    // Initialize OpenAI client for embeddings
    let openai_client = rig_core::providers::openai::Client::from_env();
    let model = openai_client.embedding_model(rig_core::providers::openai::TEXT_EMBEDDING_3_SMALL);

    // Initialize vector store
    let vector_store = PostgresVectorStore::with_defaults(model, pool);

    // Initialize message bus
    let message_bus = MessageBus::new();

    // Initialize personality
    let personality = Arc::new(personality::StoicPersonality::new());

    // Create agent system
    let mut agent_system = AgentSystem::new()
        .with_retry_policy(3, Duration::from_secs(10))
        .with_health_check_interval(Duration::from_secs(30));

    // Add agents
    agent_system
        .add_agent(agents::DataIngestionAgent::new(
            message_bus.clone(),
            vector_store.clone(),
            personality.clone(),
        ))
        .add_agent(agents::PredictionAgent::new(
            message_bus.clone(),
            vector_store.clone(),
            personality.clone(),
        ))
        .add_agent(agents::DecisionAgent::new(
            message_bus.clone(),
            vector_store.clone(),
            personality.clone(),
        ))
        .add_agent(agents::ExecutionAgent::new(
            message_bus.clone(),
            vector_store.clone(),
            personality.clone(),
        ))
        .add_agent(agents::TwitterAgent::new(
            message_bus.clone(),
            personality.clone(),
        ));

    // Start all agents
    agent_system.run().await?;

    Ok(())
}

/// Example usage
pub async fn example() -> anyhow::Result<()> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let twitter_api_key = env::var("TWITTER_API_KEY").expect("TWITTER_API_KEY not set");

    init_bot(
        &database_url,
        &openai_api_key,
        &twitter_api_key,
    ).await?;

    Ok(())
} 
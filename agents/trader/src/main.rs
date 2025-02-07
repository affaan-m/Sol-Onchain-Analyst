use rig_core::{
    agent::{Agent, AgentSystem},
    message_bus::MessageBus,
};
use rig_postgres::PostgresVectorStore;
use sqlx::postgres::PgPoolOptions;
use rig_solana_trader::{
    agents::{DataIngestionAgent, DecisionAgent, ExecutionAgent, PredictionAgent, TwitterAgent},
    personality::StoicPersonality,
};
use std::sync::Arc;
use std::env;
use std::time::Duration;

mod data_ingestion;
mod prediction;
mod decision;
mod execution;
mod feedback;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize shared components
    let message_bus = MessageBus::new();
    let personality = Arc::new(StoicPersonality::new());
    
    // Configure PostgreSQL connection pool
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .idle_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await?;

    // Initialize OpenAI client for embeddings
    let openai_client = rig_core::providers::openai::Client::from_env();
    let model = openai_client.embedding_model(rig_core::providers::openai::TEXT_EMBEDDING_3_SMALL);

    // Initialize PostgreSQL vector store
    let vector_store = PostgresVectorStore::with_defaults(model, pool);

    // Create agent system
    let mut agent_system = AgentSystem::new()
        .with_retry_policy(3, Duration::from_secs(10))
        .with_health_check_interval(Duration::from_secs(30));

    // Add agents with their dependencies
    agent_system
        .add_agent(DataIngestionAgent::new(
            message_bus.clone(),
            vector_store.clone(),
            personality.clone(),
        ))
        .add_agent(PredictionAgent::new(
            message_bus.clone(),
            vector_store.clone(),
            personality.clone(),
        ))
        .add_agent(DecisionAgent::new(
            message_bus.clone(),
            vector_store.clone(),
            personality.clone(),
        ))
        .add_agent(ExecutionAgent::new(
            message_bus.clone(),
            vector_store.clone(),
            personality.clone(),
        ))
        .add_agent(TwitterAgent::new(
            message_bus.clone(),
            personality.clone(),
        ));

    // Start all agents
    agent_system.run().await?;
    
    Ok(())
}

async fn trading_loop(
    executor: Arc<SolanaExecutor>,
    risk_manager: Arc<RiskManager>,
    twitter: Arc<TwitterClient>,
) -> Result<()> {
    let market_client = MarketDataClient::new(env::var("PUMPFUN_API_KEY")?);
    
    loop {
        let token_data = market_client.get_token_data("TOKEN_MINT").await?;
        let analysis = TradeAnalysis {
            market_cap: token_data.current_market_cap,
            volume_ratio: token_data.buy_volume_4h / token_data.sell_volume_4h,
            risk_assessment: market_client.analyze_market(&token_data),
        };

        let action = TradeAction {
            action_type: TradeType::Buy,
            params: TradeParams {
                mint: "TOKEN_MINT".into(),
                amount: 0.1,
                slippage: 10,
                units: 1_000_000,
            },
            analysis: Some(analysis),
        };

        risk_manager.validate_trade(&action)?;
        
        let signature = executor.execute_trade(action.clone()).await?;
        twitter.post_trade(&action, &signature.to_string()).await?;

        tokio::time::sleep(Duration::from_secs(300)).await;
    }
} 
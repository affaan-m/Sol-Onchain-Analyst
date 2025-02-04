use anyhow::Result;
use chrono::Utc;
use rig_solana_trader::{
    database::DatabaseClient,
    market_data::{MarketDataProvider, loaders::MarketDataLoader},
    strategy::{TradingStrategy, pipeline::TradingPipeline},
    execution::ExecutionEngine,
    agents::TradingAgentSystem,
    market_data::vector_store::{TokenAnalysis, TokenVectorStore},
    strategy::{StrategyConfig, StrategyParameters, RiskLevel},
};
use rig::providers::openai::Client as OpenAIClient;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::path::PathBuf;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .pretty()
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize OpenAI client
    let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let openai_client = OpenAIClient::new(&openai_api_key);
    let model = openai_client.completion_model("gpt-4o");

    // Initialize PostgreSQL connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    // Initialize database client
    let db_client = DatabaseClient::new(&database_url).await?;

    // Initialize vector store
    let vector_store = TokenVectorStore::new(pool.clone());

    // Initialize market data components
    let market_data = MarketDataProvider::new(&openai_api_key, db_client.clone()).await?;
    let data_loader = MarketDataLoader::new();

    // Create test strategy config
    let strategy_config = StrategyConfig {
        id: Uuid::new_v4(),
        name: "Test Strategy".to_string(),
        description: "A test trading strategy".to_string(),
        risk_level: RiskLevel::Medium,
        parameters: StrategyParameters {
            min_market_cap: 1_000_000.0,
            min_volume_24h: 100_000.0,
            min_price_change: -5.0,
            max_price_change: 5.0,
            max_slippage: 1.0,
        },
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Initialize trading components
    let strategy = TradingStrategy::new(model.clone(), strategy_config.clone());
    let execution = ExecutionEngine::new(strategy_config.parameters.max_slippage);

    // Initialize trading pipeline
    let pipeline = TradingPipeline::new(market_data.clone(), strategy, execution);

    // Initialize multi-agent system
    let agents = TradingAgentSystem::new(model);

    // Test tokens
    let test_tokens = vec![
        "So11111111111111111111111111111111111111112", // Wrapped SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", // BONK
    ];

    for token in test_tokens {
        info!("Processing token {}", token);

        // 1. Load and analyze market data
        let market_report = data_loader.load_market_report("data/market_reports/latest.txt").await?;
        let whitepaper = data_loader.load_token_whitepaper("data/whitepapers/token.pdf").await?;
        
        // 2. Get multi-agent analysis
        let token_data = format!(
            "Token: {}\nMarket Report:\n{}\nWhitepaper:\n{}",
            token, market_report, whitepaper
        );
        let decision = agents.make_trading_decision(&token_data).await?;
        info!("Agent decision: {}", decision);

        // 3. Execute through pipeline
        let tx_signature = pipeline.execute_trade(token.to_string()).await?;
        info!("Transaction executed: {}", tx_signature);

        // 4. Store analysis in vector store
        let analysis = TokenAnalysis {
            id: Uuid::new_v4(),
            token_address: token.to_string(),
            sentiment_score: decision.sentiment_score,
            technical_score: decision.technical_score,
            risk_score: decision.risk_score,
            symbol: token.to_string(),
            description: format!("Analysis for {}", token),
            recent_events: vec![decision.reasoning.clone()],
            market_sentiment: decision.market_sentiment.clone(),
            timestamp: Utc::now(),
        };

        // Generate embeddings and store
        let embeddings = rig_core::embeddings::EmbeddingsBuilder::new(model.clone())
            .documents(vec![analysis.clone()])?
            .build()
            .await?;

        vector_store.add_analysis(analysis, embeddings).await?;
    }

    // Save strategy config
    let strategy_id = db_client.insert_document("strategies", &strategy_config).await?;
    info!("Created strategy with ID: {}", strategy_id);

    Ok(())
}
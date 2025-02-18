use anyhow::Result;
use dotenv::dotenv;
use rig_solana_trader::{
    market_data::MarketDataProvider,
    strategy::{
        TradingStrategy, StrategyConfig, analytics::PerformanceAnalyzer,
        pipeline::TradingPipeline, risk::RiskManager, execution::ExecutionEngine,
    },
};
use solana_sdk::signature::{Keypair, read_keypair_file};
use std::{env, sync::Arc};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .pretty()
        .init();

    // Load environment variables
    dotenv().ok();
    
    info!("Starting autonomous trading agent...");

    // Initialize OpenAI client and agent
    let openai_client = openai::Client::from_env();
    let agent = openai_client.agent("gpt-4").build();

    // Load wallet
    let wallet = if let Ok(private_key) = env::var("SOLANA_PRIVATE_KEY") {
        let bytes = bs58::decode(&private_key)
            .into_vec()
            .expect("Invalid private key");
        Keypair::from_bytes(&bytes).expect("Invalid keypair")
    } else {
        read_keypair_file(&*shellexpand::tilde("~/.config/solana/id.json"))
            .expect("Failed to read keypair file")
    };

    info!("Wallet loaded: {}", wallet.pubkey());

    // Initialize market data provider
    let market_data = Arc::new(MarketDataProvider::new(
        &env::var("BIRDEYE_API_KEY")?,
        &env::var("SOLANA_RPC_URL")?,
    ).await?);

    // Initialize strategy components
    let strategy_config = StrategyConfig {
        max_position_sol: env::var("MAX_POSITION_SIZE_SOL")?.parse()?,
        min_position_sol: env::var("MIN_POSITION_SIZE_SOL")?.parse()?,
        max_tokens: env::var("MAX_TOKENS_PER_WALLET")?.parse()?,
        min_confidence: env::var("MIN_CONFIDENCE_THRESHOLD")?.parse()?,
        min_liquidity_usd: env::var("MIN_LIQUIDITY_USD")?.parse()?,
        max_slippage: env::var("MAX_SLIPPAGE")?.parse()?,
    };

    let strategy = Arc::new(TradingStrategy::new(agent, strategy_config));
    
    // Initialize risk manager with initial portfolio value
    let risk_manager = Arc::new(RiskManager::new(
        env::var("INITIAL_PORTFOLIO_VALUE")?.parse()?,
        env::var("MAX_DRAWDOWN")?.parse()?,
    ));

    // Initialize execution engine without Jupiter API key
    let execution_engine = Arc::new(ExecutionEngine::new(
        &env::var("SOLANA_RPC_URL")?,
        strategy_config.max_slippage,
        wallet.pubkey(),
    )?);

    // Initialize trading pipeline
    let pipeline = TradingPipeline::new(
        market_data.clone(),
        strategy.clone(),
        risk_manager.clone(),
        execution_engine.clone(),
    );

    // Initialize performance analyzer
    let mut performance_analyzer = PerformanceAnalyzer::new(
        env::var("INITIAL_PORTFOLIO_VALUE")?.parse()?,
    );

    info!("All components initialized, starting trading loop...");

    // Main trading loop
    loop {
        // 1. Get trending tokens
        let trending_tokens = market_data.get_trending_tokens(10).await?;
        info!("Found {} trending tokens to analyze", trending_tokens.len());

        // 2. Process each token
        for token in trending_tokens {
            if let Some(result) = pipeline.process_token(&token.address).await? {
                // Record trade result
                performance_analyzer.record_trade(result.into());
                
                // Log performance metrics
                let metrics = performance_analyzer.get_metrics();
                info!(
                    "Trading Performance: Win Rate: {:.2}%, Total P/L: {:.2} SOL, Sharpe: {:.2}",
                    metrics.win_rate * 100.0,
                    metrics.total_profit_loss,
                    metrics.sharpe_ratio
                );

                // Analyze strategy performance
                let analysis = performance_analyzer.analyze_strategy_performance("default")?;
                if !analysis.recommended_adjustments.is_empty() {
                    info!("Strategy Recommendations:");
                    for recommendation in analysis.recommended_adjustments {
                        info!("- {}", recommendation);
                    }
                }
            }
        }

        // 3. Monitor existing positions
        pipeline.monitor_positions().await?;

        // 4. Wait before next iteration
        sleep(Duration::from_secs(60)).await;
    }
}
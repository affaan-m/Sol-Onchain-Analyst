use crate::{
    agent::trader::TradingAgent,
    config::AgentConfig,
    models::market_signal::{SignalType, MarketSignal},
    trading::SolanaAgentKit,
    utils::f64_to_decimal, 
};
use std::io::{self, Write};
use bson::DateTime;
use config::mongodb::{MongoConfig, MongoDbPool};
use solana_sdk::signature::Keypair;
use tokio;
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};
use std::sync::atomic::{AtomicBool, Ordering};

mod agent;
mod config;
mod error;
mod trading;
mod twitter;
mod birdeye;
mod models;
mod services;
mod utils;
mod logging;

async fn handle_user_input(
    trader: Arc<TradingAgent>,
    config: AgentConfig,
    running: Arc<AtomicBool>,
) {
    println!("\n=== Cainam Trading Agent ===");
    println!("The agent is running autonomously in the background.");
    println!("\nAvailable commands:");
    println!("  analyze <symbol> <address>    - Analyze market for a token");
    println!("  trade <symbol> <buy|sell> <amount>  - Execute a trade");
    println!("  status                        - Get current trading status");
    println!("  exit                          - Exit the program");
    println!("\nType a command and press Enter.\n");

    loop {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        print!("> ");
        io::stdout().flush().unwrap_or_default();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let parts: Vec<String> = input
                    .trim()
                    .split_whitespace()
                    .map(String::from)
                    .collect();
                
                if parts.is_empty() {
                    continue;
                }

                match parts[0].as_str() {
                    "analyze" => {
                        if parts.len() != 3 {
                            println!("Usage: analyze <symbol> <address>");
                            continue;
                        }
                        println!("Analyzing market for {}...", parts[1]);
                        tokio::spawn({
                            let trader = trader.clone();
                            let symbol = parts[1].clone();
                            let address = parts[2].clone();
                            async move {
                                match trader.analyze_market(&symbol, &address).await {
                                    Ok(Some(signal)) => {
                                        println!("\nMarket Analysis Result:");
                                        println!("  Signal: {:?}", signal.signal_type);
                                        println!("  Confidence: {:.2}", signal.confidence);
                                        println!("  Risk Score: {:.2}", signal.risk_score);
                                    }
                                    Ok(None) => println!("\nNo trading signals generated"),
                                    Err(e) => println!("\nAnalysis failed: {}", e),
                                }
                            }
                        });
                    }
                    "trade" => {
                        if parts.len() != 4 {
                            println!("Usage: trade <symbol> <buy|sell> <amount>");
                            continue;
                        }
                        let amount = match parts[3].parse::<f64>() {
                            Ok(val) => val,
                            Err(_) => {
                                println!("Invalid amount. Please provide a valid number.");
                                continue;
                            }
                        };

                        let signal_type = match parts[2].to_uppercase().as_str() {
                            "BUY" => SignalType::StrongBuy,
                            "SELL" => SignalType::StrongSell,
                            _ => {
                                println!("Invalid trade type. Use 'buy' or 'sell'");
                                continue;
                            }
                        };

                        println!("Executing {} trade for {}...", parts[2], parts[1]);
                        tokio::spawn({
                            let trader = trader.clone();
                            let symbol = parts[1].clone();
                            async move {
                                let signal = MarketSignal {
                                    id: None,
                                    asset_address: symbol.clone(),
                                    signal_type: signal_type.clone(),
                                    confidence: f64_to_decimal(0.8),
                                    risk_score: f64_to_decimal(0.2),
                                    sentiment_score: Some(f64_to_decimal(0.6)),
                                    volume_change_24h: Some(f64_to_decimal(0.15)),
                                    price_change_24h: Some(f64_to_decimal(if signal_type == SignalType::StrongBuy { 0.05 } else { -0.05 })),
                                    price: f64_to_decimal(10.0),
                                    volume_change: f64_to_decimal(0.2),
                                    timestamp: DateTime::now(),
                                    metadata: None,
                                    created_at: None,
                                };

                                let min_confidence = f64_to_decimal(config.trade_min_confidence);
                                if signal.confidence >= min_confidence {
                                    match trader.execute_trade(&symbol, &signal).await {
                                        Ok(signature) => {
                                            println!("\nTrade executed successfully!");
                                            println!("Transaction: {}", signature);
                                            if let Err(e) = trader.post_trade_update(&symbol, &parts[2], amount, &signal_type).await {
                                                println!("Failed to post trade update: {}", e);
                                            }
                                        }
                                        Err(e) => println!("\nTrade execution failed: {}", e),
                                    }
                                }
                            }
                        });
                    }
                    "status" => {
                        println!("\nTrading Agent Status:");
                        println!("  State: Active");
                        println!("  Analysis Interval: {:?}", config.analysis_interval);
                        println!("  Min Confidence: {:.2}", config.trade_min_confidence);
                        println!("  Max Trade Amount: {:.2}", config.trade_max_amount);
                    }
                    "exit" => {
                        println!("\nShutting down trading agent...");
                        running.store(false, Ordering::SeqCst);
                        break;
                    }
                    _ => println!("Unknown command. Type 'help' for available commands."),
                }
            }
            Err(e) => {
                error!("Error reading input: {}", e);
                break;
            }
        }
    }
}

async fn init_mongodb() -> Result<Arc<MongoDbPool>> {
    let config = MongoConfig::default();
    let pool = MongoDbPool::create_pool(config).await?;
    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    // logging::init_logging()?;
    
    println!("Starting Cainam Core...");

    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    println!("loadi env file...");

    // Initialize MongoDB connection pool using rig-mongodb
    let db_pool = init_mongodb().await?;
    println!("init pool...");
    
    // Initialize Solana agent
    let rpc_url = "https://api.devnet.solana.com";
    let keypair = Keypair::new();
    let solana_agent = SolanaAgentKit::new(rpc_url, keypair);

    // Load configuration from environment
    let config = AgentConfig::new_from_env()?;

    // Initialize trading agent
    let trader = Arc::new(TradingAgent::new(config.clone(), db_pool, solana_agent).await?);
    let running = Arc::new(AtomicBool::new(true));

    // Initialize services with MongoDB pool
    // let token_analytics_service = TokenAnalyticsService::new(
    //     db_pool.clone(),
    //     birdeye.clone(),
    //     birdeye_extended.clone(),
    //     Some(market_config.clone()),
    // ).await?;
    
    // let portfolio_optimizer = PortfolioOptimizer::new(db_pool.clone());
    
    // // Initialize vector store
    // let vector_store = VectorStore::new().await?;

    // // Spawn the autonomous trading agent
    // let trader_clone = trader.clone();
    // let running_clone = running.clone();
    // let trading_handle = tokio::spawn(async move {
    //     info!("Starting autonomous trading...");
    //     if let Err(e) = trader_clone.run().await {
    //         error!("Trading agent error: {}", e);
    //         running_clone.store(false, Ordering::SeqCst);
    //     }
    // });

    // Handle user input in a separate task
    let input_handle = tokio::spawn(handle_user_input(trader.clone(), config, running.clone()));

    // Wait for either task to complete
    tokio::select! {
        // _ = trading_handle => {
        //     info!("Trading task completed");
        // }
        _ = input_handle => {
            info!("User input task completed");
        }
    }

    // Wait for clean shutdown
    info!("Shutting down trading agent...");
    running.store(false, Ordering::SeqCst);
    trader.stop();

    Ok(())
}
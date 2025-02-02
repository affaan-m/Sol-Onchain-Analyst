use crate::{
    agent::trader::TradingAgent,
    config::AgentConfig,
    models::market_signal::{SignalType, MarketSignal},
    trading::SolanaAgentKit,
    utils::f64_to_decimal,
};
use std::io::{self, Write};
use tokio;
use sqlx::PgPool;
use std::sync::Arc;
use chrono::Utc;
use anyhow::Result;
use tracing::info;

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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    logging::init_logging()?;
    
    info!("Starting Cainam Core...");

    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| error::AgentError::MissingEnvVar("DATABASE_URL".to_string()))?;
    
    let db = Arc::new(
        PgPool::connect(&database_url)
            .await
            .map_err(|e| error::AgentError::Database(e))?
    );

    // Initialize Solana agent
    let solana_agent = SolanaAgentKit::new_from_env()?;

    // Load configuration from environment
    let config = AgentConfig::new_from_env()?;

    // Initialize trading agent
    let trader = TradingAgent::new(config.clone(), db.clone(), solana_agent).await?;

    println!("Trading Agent initialized! Available commands:");
    println!("  analyze <symbol> <address>    - Analyze market for a token");
    println!("  trade <symbol> <buy|sell> <amount>  - Execute a trade");
    println!("  exit                          - Exit the program");

    loop {
        print!("Enter command (analyze/trade/exit): ");
        io::stdout().flush().map_err(|e| anyhow::anyhow!(e))?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| anyhow::anyhow!(e))?;

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "analyze" => {
                if parts.len() != 3 {
                    println!("Usage: analyze <symbol> <address>");
                    continue;
                }
                match trader.analyze_market(parts[1], parts[2]).await {
                    Ok(Some(signal)) => {
                        println!("Signal generated: {:?} (confidence: {:.2})", signal.signal_type, signal.confidence);
                        let min_confidence = f64_to_decimal(config.trade_min_confidence);
                        if signal.confidence >= min_confidence {
                            match trader.execute_trade(parts[1], &signal).await {
                                Ok(signature) => println!("Trade executed: {}", signature),
                                Err(e) => println!("Trade execution failed: {}", e),
                            }
                        }
                    }
                    Ok(None) => println!("No trading signals generated"),
                    Err(e) => println!("Analysis failed: {}", e),
                }
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

                let signal = MarketSignal {
                    id: None,
                    asset_address: parts[1].to_string(),
                    signal_type: signal_type.clone(),
                    confidence: f64_to_decimal(0.8),
                    risk_score: f64_to_decimal(0.2),
                    sentiment_score: Some(f64_to_decimal(0.6)),
                    volume_change_24h: Some(f64_to_decimal(0.15)),
                    price_change_24h: Some(f64_to_decimal(if signal_type == SignalType::StrongBuy { 0.05 } else { -0.05 })),
                    price: f64_to_decimal(10.0),
                    volume_change: f64_to_decimal(0.2),
                    timestamp: Utc::now(),
                    metadata: None,
                    created_at: None,
                };

                let min_confidence = f64_to_decimal(config.trade_min_confidence);
                if signal.confidence >= min_confidence {
                    match trader.execute_trade(parts[1], &signal).await {
                        Ok(signature) => {
                            println!("Trade executed: {}", signature);
                            if let Err(e) = trader.post_trade_update(parts[1], parts[2], amount, &signal_type).await {
                                println!("Failed to post trade update: {}", e);
                            }
                        }
                        Err(e) => println!("Trade execution failed: {}", e),
                    }
                }
            }
            "exit" => break,
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
    }

    Ok(())
}
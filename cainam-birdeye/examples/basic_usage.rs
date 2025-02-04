use cainam_birdeye::{BirdeyeClient, types::api::TokenSearchParams};
use std::{env, time::Duration, io::{self, BufRead}, sync::Arc};
use anyhow::Result;
use tokio::time::sleep;
use crate::{
    agent::trader::TradingAgent,
    config::AgentConfig,
    models::market_signal::{SignalType, MarketSignal},
    trading::SolanaAgentKit,
    utils::f64_to_decimal,
};
use tokio::sync::mpsc;
use tracing::info;
use serde::{Deserialize, Serialize};

const MONITORING_INTERVAL: u64 = 60; // seconds
const VOLUME_THRESHOLD: f64 = 1000000.0; // $1M volume
const PRICE_CHANGE_THRESHOLD: f64 = 5.0; // 5%
const TRADE_AMOUNT: f64 = 0.1; // SOL

#[derive(Debug, Serialize, Deserialize)]
struct TradeAnalysis {
    confidence: f64,
    action: String,
    reasoning: String,
}

struct TradingAgent {
    client: OpenAIClient,
}

impl TradingAgent {
    async fn new(api_key: &str) -> Result<Self> {
        let client = OpenAIClient::new(api_key);
        Ok(Self { client })
    }

    async fn analyze_opportunity(&self, token_name: &str, price: f64, volume: f64, price_change: f64) -> Result<TradeAnalysis> {
        let prompt = format!(
            "Analyze this trading opportunity:\n\
            Token: {}\n\
            Price: ${:.4}\n\
            24h Volume: ${:.2}\n\
            Price Change: {:.2}%\n\n\
            Respond with a JSON object containing:\n\
            - confidence: 0-1 score of trade confidence\n\
            - action: 'BUY' or 'SELL'\n\
            - reasoning: Brief explanation",
            token_name, price, volume, price_change
        );

        let messages = vec![ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(prompt),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
        }];

        let chat_completion = ChatCompletion::builder("gpt-4o", messages)
            .create_async(&self.client)
            .await?;

        let response = chat_completion.choices[0].message.content.clone();
        let analysis: TradeAnalysis = serde_json::from_str(&response)?;
        Ok(analysis)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    // Load environment variables
    let api_key = env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY must be set");
    let openai_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let rpc_url = format!("https://api.mainnet-beta.solana.com");
    let private_key = env::var("SOLANA_PRIVATE_KEY").expect("SOLANA_PRIVATE_KEY must be set");

    // Initialize clients
    let client = BirdeyeClient::new(api_key);
    let trader = SolanaTrader::new(&private_key, &rpc_url).await?;
    let trading_agent = TradingAgent::new(&openai_key).await?;

    info!("Starting autonomous trading agent...");
    info!("Monitoring interval: {}s", MONITORING_INTERVAL);
    info!("Volume threshold: ${}", VOLUME_THRESHOLD);
    info!("Price change threshold: {}%", PRICE_CHANGE_THRESHOLD);
    info!("Trade amount: {} SOL", TRADE_AMOUNT);
    info!("Wallet address: {}", trader.wallet_address());

    // Channel for communication between user input and trading
    let (tx, mut rx) = mpsc::channel(32);
    let tx_clone = tx.clone();

    // Spawn trading loop
    let client = Arc::new(client);
    let trader = Arc::new(trader);
    let trading_agent = Arc::new(trading_agent);
    
    let trading_client = client.clone();
    let trading_trader = trader.clone();
    let trading_ai = trading_agent.clone();
    
    tokio::spawn(async move {
        loop {
            match monitor_and_trade(&trading_client, &trading_trader, &trading_ai).await {
                Ok(_) => info!("Completed monitoring cycle"),
                Err(e) => eprintln!("Error in monitoring cycle: {}", e),
            }

            // Check for user commands
            while let Ok(command) = rx.try_recv() {
                match command.as_str() {
                    "status" => {
                        info!("Agent is running");
                        info!("Last check: {:?}", chrono::Utc::now());
                    },
                    "pause" => {
                        info!("Trading paused. Type 'resume' to continue");
                        if let Ok(cmd) = rx.recv().await {
                            if cmd != "resume" {
                                info!("Unknown command: {}", cmd);
                            }
                        }
                    },
                    _ => info!("Unknown command: {}", command),
                }
            }

            sleep(Duration::from_secs(MONITORING_INTERVAL)).await;
        }
    });

    // Spawn user input handler
    tokio::spawn(async move {
        println!("Agent is running autonomously. Available commands:");
        println!("- status: Check agent status");
        println!("- pause: Pause trading");
        println!("- resume: Resume trading");
        println!("- exit: Exit the program");

        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(input) = line {
                match input.trim() {
                    "exit" => {
                        info!("Shutting down...");
                        std::process::exit(0);
                    },
                    cmd => {
                        if let Err(e) = tx_clone.send(cmd.to_string()).await {
                            eprintln!("Error sending command: {}", e);
                        }
                    }
                }
            }
        }
    });

    // Keep main task running
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}

async fn monitor_and_trade(
    client: &BirdeyeClient,
    trader: &SolanaTrader,
    trading_agent: &TradingAgent,
) -> Result<()> {
    let params = TokenSearchParams::new("".to_string())
        .with_limit(20);
    
    let tokens = client.search_tokens(params).await?;

    for token in tokens {
        if let (Some(volume), Some(price)) = (token.volume_24h, token.price) {
            if volume >= VOLUME_THRESHOLD {
                let overview = client.get_token_overview(token.address.clone()).await?;
                let price_change = ((overview.price - price) / price) * 100.0;
                
                if price_change.abs() >= PRICE_CHANGE_THRESHOLD {
                    let analytics = TokenAnalytics {
                        id: None,
                        token_address: token.address.clone(),
                        token_name: token.name.clone(),
                        // Add other fields as needed
                    };
                    
                    info!("Analyzing opportunity:");
                    info!("Token: {} ({})", analytics.token_name, token.symbol);
                    info!("Price: ${:.4}", price);
                    info!("24h Volume: ${:.2}", volume);
                    info!("Price Change: {:.2}%", price_change);
                    
                    // Get AI analysis
                    let analysis = trading_agent.analyze_opportunity(
                        &analytics,
                        price,
                        volume,
                        price_change
                    ).await?;
                    
                    info!("Analysis: {}", analysis.reasoning);
                    
                    // Execute trade if confidence is high enough
                    if analysis.confidence >= 0.8 {
                        info!("Executing {} trade with confidence {:.2}", analysis.action, analysis.confidence);
                        match trader.execute_trade(&token.address, TRADE_AMOUNT).await {
                            Ok(signature) => info!("Trade executed: {}", signature),
                            Err(e) => eprintln!("Trade failed: {}", e),
                        }
                    }
                }
            }
        }
    }

    Ok(())
} 
use cainam_core::{
    birdeye::api::{BirdeyeApi, BirdeyeClient},
    config::{AgentConfig, mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig}},
    services::{TokenAnalyticsService, TokenAnalyticsLLM},
};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::sync::Arc;
use tokio;
use tracing::{error, info, Level};
use chrono;
use anyhow::Result;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get trending tokens
    Trending,
    
    /// Get token overview and analytics
    Token {
        /// Token address
        address: String,
    },
    
    /// Get market signals for a token
    Signals {
        /// Token address
        address: String,
    },
    
    /// Start monitoring mode for specified tokens
    Monitor {
        /// Token addresses (comma-separated)
        #[arg(value_delimiter = ',')]
        addresses: Vec<String>,
        
        /// Monitoring interval in seconds
        #[arg(default_value = "300")]
        interval: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .init();

    // Load environment variables
    dotenv().ok();

    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize services
    let config = AgentConfig::new_from_env()?;
    
    // Create MongoDB configuration
    let mongodb_uri = dotenvy::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let mongodb_database = dotenvy::var("MONGODB_DATABASE").expect("MONGODB_DATABASE must be set");
    let mongo_config = MongoConfig {
        uri: mongodb_uri,
        database: mongodb_database,
        app_name: Some("cainam-cli".to_string()),
        pool_config: MongoPoolConfig::default(),
    };
    
    let db_pool = MongoDbPool::create_pool(mongo_config).await?;
    let birdeye: Arc<dyn BirdeyeApi> = Arc::new(BirdeyeClient::new(config.birdeye_api_key.clone()));
    let analytics_service = Arc::new(
        TokenAnalyticsService::new(db_pool.into(), birdeye.clone(), None).await?,
    );
    
    let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let analytics_llm = TokenAnalyticsLLM::new(
        analytics_service.clone(),
        &openai_api_key,
    );

    // Process commands
    match cli.command {
        Commands::Trending => {
            info!("Fetching trending tokens...");
            let insights = analytics_llm.get_market_insights().await?;
            println!("\n{}", insights);
        }

        Commands::Token { address } => {
            info!("Fetching token analytics for {}", address);
            let overview = birdeye.get_token_overview(&address).await?;
            let analytics = analytics_service
                .fetch_and_store_token_info(&overview.symbol, &address)
                .await?;
            
            // Get AI analysis of the token
            let analysis = analytics_llm.analyze_query(&format!(
                "Analyze the token {} ({}) with price ${:.8} and provide insights about its current market status.",
                overview.name, overview.symbol, overview.price
            )).await?;
            
            println!("\nToken Overview:");
            println!("Name: {} ({})", overview.name, overview.symbol);
            println!("Price: ${:.8}", overview.price);
            println!("Market Cap: ${:.2}M", overview.market_cap / 1_000_000.0);
            println!("24h Volume: ${:.2}", overview.v24h_usd);
            println!("24h Price Change: {:.2}%", overview.price_change_24h_percent);
            println!("Holders: {}", overview.holder);
            println!("Active Wallets (24h): {}", overview.unique_wallet_24h);
            println!("\nAI Analysis:\n{}", analysis);
        }

        Commands::Signals { address } => {
            info!("Analyzing market signals for {}", address);
            let overview = birdeye.get_token_overview(&address).await?;
            let analytics = analytics_service
                .fetch_and_store_token_info(&overview.symbol, &address)
                .await?;
            
            if let Some(signal) = analytics_service.generate_market_signals(&analytics).await? {
                println!("\nMarket Signals for {} ({}):", overview.name, overview.symbol);
                println!("Signal Type: {:?}", signal.signal_type);
                println!("Confidence: {:.2}", signal.confidence);
                println!("Risk Score: {:.2}", signal.risk_score);
                if let Some(price_change) = &signal.price_change_24h {
                    println!("Price Change (24h): {:.2}%", price_change);
                }
                if let Some(volume_change) = &signal.volume_change_24h {
                    println!("Volume Change (24h): {:.2}%", volume_change);
                }
            } else {
                println!("\nNo significant market signals detected.");
            }
        }

        Commands::Monitor { addresses, interval } => {
            info!("Starting monitoring mode...");
            println!("\nMonitoring {} tokens every {} seconds...", addresses.len(), interval);
            println!("Press Ctrl+C to stop.");

            loop {
                for address in &addresses {
                    match birdeye.get_token_overview(address).await {
                        Ok(overview) => {
                            let analytics = analytics_service
                                .fetch_and_store_token_info(&overview.symbol, address)
                                .await?;
                            
                            if let Some(signal) = analytics_service.generate_market_signals(&analytics).await? {
                                println!("\n[{}] Signal for {} ({}):", 
                                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                                    overview.name,
                                    overview.symbol
                                );
                                println!("Type: {:?} (Confidence: {:.2})", signal.signal_type, signal.confidence);
                                println!("Price: ${:.8} ({:+.2}%)", overview.price, overview.price_change_24h_percent);
                            }
                        }
                        Err(e) => {
                            error!("Failed to fetch data for {}: {}", address, e);
                        }
                    }
                }
                
                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            }
        }
    }

    Ok(())
} 
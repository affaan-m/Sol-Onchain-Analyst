use cainam_birdeye::providers::{WebSocketProvider, MarketUpdate, TradeUpdate};
use std::env;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    // Get API key from environment variable
    let api_key = env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY not found in .env file");

    // Create WebSocket provider
    let ws_provider = WebSocketProvider::new(&api_key);

    // Subscribe to market updates
    let mut market_rx = ws_provider.subscribe_market_updates();
    tokio::spawn(async move {
        while let Ok(update) = market_rx.recv().await {
            println!("Market Update:");
            println!("  Token: {}", update.address);
            println!("  Price: ${:.4}", update.price);
            println!("  24h Volume: ${:.2}", update.volume_24h);
            println!("  24h Change: {:.2}%", update.price_change_24h);
            println!();
        }
    });

    // Subscribe to trade updates
    let mut trade_rx = ws_provider.subscribe_trade_updates();
    tokio::spawn(async move {
        while let Ok(update) = trade_rx.recv().await {
            println!("Trade Update:");
            println!("  Token: {}", update.address);
            println!("  Price: ${:.4}", update.price);
            println!("  Size: ${:.2}", update.size);
            println!("  Side: {:?}", update.side);
            println!();
        }
    });

    // List of token addresses to monitor
    let tokens = vec![
        // Example token addresses (replace with actual addresses)
        "So11111111111111111111111111111111111111112".to_string(), // Wrapped SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
    ];

    // Connect and start streaming
    println!("Connecting to Birdeye WebSocket...");
    ws_provider.connect_and_stream(tokens).await?;

    // Keep the main task running
    println!("Streaming market data. Press Ctrl+C to exit.");
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

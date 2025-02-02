use cainam_birdeye::{BirdeyeClient, types::api::TokenSearchParams};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load API key from environment
    let api_key = env::var("BIRDEYE_API_KEY")
        .expect("BIRDEYE_API_KEY must be set");

    // Create Birdeye client
    let client = BirdeyeClient::new(api_key);

    // Search for top SOL tokens by volume
    let params = TokenSearchParams::new("SOL".to_string())
        .with_limit(5);

    println!("\nSearching for top SOL tokens by volume...");
    let tokens = client.search_tokens(params).await?;

    println!("\nTop 5 SOL tokens by 24h volume:");
    println!("{:<20} {:<10} {:<15} {:<15}", "Name", "Symbol", "Price ($)", "24h Volume ($)");
    println!("{}", "-".repeat(60));

    for token in tokens {
        println!(
            "{:<20} {:<10} {:<15.2} {:<15.2}",
            token.name,
            token.symbol,
            token.price.unwrap_or_default(),
            token.volume_24h.unwrap_or_default()
        );
    }

    Ok(())
} 
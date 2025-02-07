use rig_birdeye::{
    actions::{TokenSearchAction, WalletSearchAction},
    providers::birdeye::BirdeyeProvider,
    types::{
        api::{TokenSortBy, SortType, TokenSearchParams},
        error::BirdeyeError,
    },
};
use std::env;

fn setup() -> BirdeyeProvider {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create Birdeye provider
    let api_key = env::var("BIRDEYE_API_KEY")
        .expect("BIRDEYE_API_KEY must be set");
    
    BirdeyeProvider::new(&api_key)
}

#[tokio::test]
async fn test_token_search() -> Result<(), Box<dyn std::error::Error>> {
    let provider = setup();

    let params = TokenSearchParams {
        keyword: "SOL".to_string(),
        sort_by: Some(TokenSortBy::Volume),
        sort_type: Some(SortType::Descending),
        offset: None,
        limit: Some(10),
    };

    let tokens = provider.search_tokens(params).await?;
    assert!(!tokens.is_empty(), "No tokens found");
    
    // Validate first token
    let token = &tokens[0];
    assert!(!token.address.is_empty(), "Token address is empty");
    assert!(!token.symbol.is_empty(), "Token symbol is empty");
    assert!(token.price_usd > 0.0, "Token price should be positive");
    assert!(token.volume_24h > 0.0, "Token volume should be positive");

    Ok(())
}

#[tokio::test]
async fn test_wallet_search() -> Result<(), Box<dyn std::error::Error>> {
    let provider = setup();

    // Use a known Solana wallet address for testing
    let wallet_address = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK".to_string();

    let portfolio = provider.search_wallet(&wallet_address).await?;
    assert_eq!(portfolio.wallet, wallet_address, "Wallet address mismatch");
    assert!(portfolio.total_usd >= 0.0, "Total USD should be non-negative");
    assert!(!portfolio.items.is_empty(), "Portfolio should contain tokens");

    Ok(())
}

#[tokio::test]
async fn test_invalid_api_key() {
    let provider = BirdeyeProvider::new("invalid_key");

    let params = TokenSearchParams {
        keyword: "SOL".to_string(),
        sort_by: None,
        sort_type: None,
        offset: None,
        limit: None,
    };

    let result = provider.search_tokens(params).await;
    assert!(result.is_err(), "Expected error with invalid API key");
    
    match result.unwrap_err() {
        BirdeyeError::InvalidApiKey => (),
        err => panic!("Expected InvalidApiKey error, got: {:?}", err),
    }
}

#[tokio::test]
async fn test_token_overview() -> Result<(), Box<dyn std::error::Error>> {
    let provider = setup();
    
    // Use SOL token address for testing
    let token_address = "So11111111111111111111111111111111111111112";
    let overview = provider.get_token_overview(token_address).await?;
    
    assert_eq!(overview.address, token_address);
    assert_eq!(overview.symbol, "SOL");
    assert!(overview.price > 0.0);
    assert!(overview.volume_24h > 0.0);
    assert!(overview.liquidity > 0.0);

    Ok(())
}

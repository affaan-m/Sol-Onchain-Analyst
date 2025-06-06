pub mod api;

pub use crate::models::market_data::TokenMarketResponse;
use crate::models::token_info::TokenInfo;
pub use crate::models::token_trending::TrendingToken;
pub use api::BirdeyeApi;
use async_trait::async_trait;

pub const BIRDEYE_API_URL: &str = "https://public-api.birdeye.so";

pub const TOKEN_ADDRESSES: &[(&str, &str)] = &[
    ("SOL", "So11111111111111111111111111111111111111112"),
    ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    ("USDT", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
    ("PYUSD", "HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3"),
];

#[async_trait]
pub trait BirdeyeClient: Send + Sync {
    async fn get_token_info(&self, symbol: &str) -> Result<TokenInfo, anyhow::Error>;
    async fn get_token_info_by_address(&self, address: &str) -> Result<TokenInfo, anyhow::Error>;
    async fn get_market_data(&self, address: &str) -> Result<TokenMarketResponse, anyhow::Error>;
    async fn get_token_trending(&self, limit: usize) -> Result<Vec<TrendingToken>, anyhow::Error>;
}

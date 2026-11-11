pub mod agent;
pub mod birdeye;
pub mod cli;
pub mod config;
pub mod error;
pub mod logging;
pub mod models;
pub mod services;
pub mod trading;
pub mod twitter;
pub mod utils;

// Re-export commonly used types
pub use crate::config::{
    birdeye_config::BirdeyeConfig,
    get_openai_model,
    market_config::MarketConfig,
    mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig},
    AgentConfig,
};

pub mod services {
    pub mod token_filter;
    pub mod wallet_tracker;
    
    pub use token_filter::TokenFilterService;
    pub use wallet_tracker::WalletTrackerService;
}

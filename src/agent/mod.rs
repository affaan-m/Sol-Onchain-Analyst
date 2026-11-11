pub mod trader;
// pub mod risk_manager;
// pub mod portfolio_optimizer;
pub mod analyst;

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub openai_api_key: String,
    pub birdeye_api_key: String,
    pub twitter_email: String,
    pub twitter_username: String,
    pub twitter_password: String,
    pub analysis_interval: Duration,
    pub trade_min_confidence: f64,
    pub trade_max_amount: f64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            openai_api_key: String::new(),
            birdeye_api_key: String::new(),
            twitter_email: String::new(),
            twitter_username: String::new(),
            twitter_password: String::new(),
            analysis_interval: Duration::from_secs(300), // 5 minutes
            trade_min_confidence: 0.7,
            trade_max_amount: 1000.0,
        }
    }
}

// Re-export common types

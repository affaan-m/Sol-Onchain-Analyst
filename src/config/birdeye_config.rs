use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BirdeyeConfig {
    pub api_key: String,
    pub api_url: String,
}

impl BirdeyeConfig {
    pub fn new_from_env() -> Result<Self> {
        Ok(Self {
            api_key: std::env::var("BIRDEYE_API_KEY")?,
            api_url: std::env::var("BIRDEYE_API_URL")
                .unwrap_or_else(|_| "https://public-api.birdeye.so".to_string()),
        })
    }
} 
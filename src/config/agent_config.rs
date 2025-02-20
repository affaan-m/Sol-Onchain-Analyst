use super::birdeye_config::BirdeyeConfig;
use crate::error::{AgentError, AgentResult};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    pub openai_api_key: String,
    pub birdeye_api_key: String,
    pub twitter_bearer_token: String,
    pub analysis_interval: Duration,
    pub trade_min_confidence: f64,
    pub trade_max_amount: f64,
    pub trade_max_slippage: f64,
    pub birdeye: BirdeyeConfig,
}

impl AgentConfig {
    /// Creates a new AgentConfig from environment variables with validation
    pub fn new_from_env() -> AgentResult<Self> {
        // Load Birdeye config
        let birdeye = BirdeyeConfig::new_from_env()
            .map_err(|e| AgentError::Config(format!("Failed to load Birdeye config: {}", e)))?;

        let config = Self {
            openai_api_key: get_env_var("OPENAI_API_KEY")?,
            birdeye_api_key: get_env_var("BIRDEYE_API_KEY")?,
            twitter_bearer_token: env::var("TWITTER_BEARER_TOKEN")
                .unwrap_or_else(|_| "AAAA".to_string()),
            analysis_interval: parse_duration_secs("ANALYSIS_INTERVAL", 300)?,
            trade_min_confidence: parse_f64("TRADE_MIN_CONFIDENCE", 0.8)?,
            trade_max_amount: parse_f64("TRADE_MAX_AMOUNT", 100.0)?,
            trade_max_slippage: parse_f64("TRADE_MAX_SLIPPAGE", 0.05)?,
            birdeye,
        };

        config.validate()?;
        Ok(config)
    }

    /// Validates the configuration values
    fn validate(&self) -> AgentResult<()> {
        // Validate API keys are not empty
        if self.openai_api_key.is_empty() {
            return Err(AgentError::Config("OpenAI API key cannot be empty".into()));
        }
        if self.birdeye_api_key.is_empty() {
            return Err(AgentError::Config("Birdeye API key cannot be empty".into()));
        }

        // Validate trading parameters
        if !(0.0..=1.0).contains(&self.trade_min_confidence) {
            return Err(AgentError::InvalidConfig(
                "trade_min_confidence".into(),
                "must be between 0.0 and 1.0".into(),
            ));
        }
        if self.trade_max_amount <= 0.0 {
            return Err(AgentError::InvalidConfig(
                "trade_max_amount".into(),
                "must be greater than 0".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.trade_max_slippage) {
            return Err(AgentError::InvalidConfig(
                "trade_max_slippage".into(),
                "must be between 0.0 and 1.0".into(),
            ));
        }

        Ok(())
    }
}

/// Helper function to get an environment variable
fn get_env_var(key: &str) -> AgentResult<String> {
    env::var(key).map_err(|_| AgentError::MissingEnvVar(key.to_string()))
}

/// Helper function to parse a duration from seconds
fn parse_duration_secs(key: &str, default: u64) -> AgentResult<Duration> {
    let secs = env::var(key)
        .map(|v| v.parse::<u64>())
        .unwrap_or(Ok(default))
        .map_err(|_| {
            AgentError::InvalidConfig(
                key.to_string(),
                "must be a valid number of seconds".to_string(),
            )
        })?;

    Ok(Duration::from_secs(secs))
}

/// Helper function to parse an f64 value
fn parse_f64(key: &str, default: f64) -> AgentResult<f64> {
    let value = env::var(key)
        .map(|v| v.parse::<f64>())
        .unwrap_or(Ok(default))
        .map_err(|_| {
            AgentError::InvalidConfig(key.to_string(), "must be a valid number".to_string())
        })?;

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_validation() {
        // Set required environment variables
        env::set_var("OPENAI_API_KEY", "test_key");
        env::set_var("BIRDEYE_API_KEY", "test_key");
        env::set_var("TWITTER_BEARER_TOKEN", "test_token");
        env::set_var("TWITTER_EMAIL", "test@example.com");
        env::set_var("TWITTER_USERNAME", "test_user");
        env::set_var("TWITTER_PASSWORD", "test_pass");

        let config = AgentConfig::new_from_env().unwrap();
        assert_eq!(config.trade_min_confidence, 0.8); // Default value
        assert_eq!(config.trade_max_amount, 100.0); // Default value
        assert_eq!(config.trade_max_slippage, 0.05); // Default value

        // Test invalid confidence
        env::set_var("TRADE_MIN_CONFIDENCE", "2.0");
        assert!(AgentConfig::new_from_env().is_err());

        // Test invalid amount
        env::set_var("TRADE_MAX_AMOUNT", "-100");
        assert!(AgentConfig::new_from_env().is_err());

        // Test invalid slippage
        env::set_var("TRADE_MAX_SLIPPAGE", "2.0");
        assert!(AgentConfig::new_from_env().is_err());

        // Test invalid email
        env::set_var("TWITTER_EMAIL", "invalid_email");
        assert!(AgentConfig::new_from_env().is_err());
    }
}

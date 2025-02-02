use thiserror::Error;
use sqlx::error::Error as SqlxError;
use std::num::ParseFloatError;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Environment variable '{0}' not found")]
    MissingEnvVar(String),

    #[error("Invalid value for {0}: {1}")]
    InvalidConfig(String, String),

    #[error("Twitter API error: {0}")]
    TwitterApi(String),

    #[error("Trading error: {0}")]
    Trading(String),

    #[error("Database error: {0}")]
    Database(#[from] SqlxError),

    #[error("Market analysis error: {0}")]
    MarketAnalysis(String),

    #[error("Vector store error: {0}")]
    VectorStore(String),

    #[error("Birdeye API error: {0}")]
    BirdeyeApi(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Rate limit exceeded for {0}")]
    RateLimit(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<ParseFloatError> for AgentError {
    fn from(err: ParseFloatError) -> Self {
        AgentError::Parse(err.to_string())
    }
}

impl From<tracing_subscriber::filter::ParseError> for AgentError {
    fn from(err: tracing_subscriber::filter::ParseError) -> Self {
        AgentError::Parse(err.to_string())
    }
}

impl From<reqwest::Error> for AgentError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AgentError::Timeout(err.to_string())
        } else if err.is_connect() {
            AgentError::Network(err.to_string())
        } else {
            AgentError::Other(err.into())
        }
    }
}

pub type AgentResult<T> = Result<T, AgentError>;

// Helper functions for common error cases
impl AgentError {
    pub fn missing_env(var: &str) -> Self {
        AgentError::MissingEnvVar(var.to_string())
    }

    pub fn invalid_config<T: std::fmt::Display>(field: &str, message: T) -> Self {
        AgentError::InvalidConfig(field.to_string(), message.to_string())
    }

    pub fn validation<T: std::fmt::Display>(message: T) -> Self {
        AgentError::Validation(message.to_string())
    }

    pub fn transaction<T: std::fmt::Display>(message: T) -> Self {
        AgentError::Transaction(message.to_string())
    }

    pub fn rate_limit<T: std::fmt::Display>(service: T) -> Self {
        AgentError::RateLimit(service.to_string())
    }

    pub fn auth<T: std::fmt::Display>(message: T) -> Self {
        AgentError::Authentication(message.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversions() {
        // Test ParseFloatError conversion
        let parse_err: AgentError = "invalid float".parse::<f64>().unwrap_err().into();
        assert!(matches!(parse_err, AgentError::Parse(_)));

        // Test helper functions
        let missing_env = AgentError::missing_env("TEST_VAR");
        assert!(matches!(missing_env, AgentError::MissingEnvVar(_)));

        let invalid_config = AgentError::invalid_config("threshold", "must be positive");
        assert!(matches!(invalid_config, AgentError::InvalidConfig(_, _)));

        let validation = AgentError::validation("invalid input");
        assert!(matches!(validation, AgentError::Validation(_)));

        let transaction = AgentError::transaction("commit failed");
        assert!(matches!(transaction, AgentError::Transaction(_)));

        let rate_limit = AgentError::rate_limit("Birdeye API");
        assert!(matches!(rate_limit, AgentError::RateLimit(_)));

        let auth = AgentError::auth("invalid credentials");
        assert!(matches!(auth, AgentError::Authentication(_)));
    }

    #[test]
    fn test_error_display() {
        let err = AgentError::missing_env("TEST_VAR");
        assert_eq!(
            err.to_string(),
            "Environment variable 'TEST_VAR' not found"
        );

        let err = AgentError::invalid_config("threshold", "must be positive");
        assert_eq!(
            err.to_string(),
            "Invalid value for threshold: must be positive"
        );

        let err = AgentError::validation("invalid input");
        assert_eq!(err.to_string(), "Validation error: invalid input");
    }
}
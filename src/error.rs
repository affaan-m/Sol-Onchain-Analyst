use thiserror::Error;
use mongodb::error::Error as MongoError;
use std::num::ParseFloatError;
use std::fmt;
use std::error::Error as StdError;
#[derive(Error, Debug)]
pub enum Error {
    #[error("MongoDB error: {0}")]
    Mongo(#[from] MongoError),
    #[error("ParseFloat error: {0}")]
    ParseFloat(#[from] ParseFloatError),
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug)]
pub enum AgentError {
    Config(String),
    MissingEnvVar(String),
    InvalidConfig(String, String),
    TwitterApi(String),
    Trading(String),
    Database(MongoError),
    MarketAnalysis(String),
    VectorStore(String),
    BirdeyeApi(String),
    Transaction(String),
    Validation(String),
    Parse(String),
    RateLimit(String),
    Authentication(String),
    Network(String),
    Timeout(String),
    Conversion(String),
    Other(anyhow::Error),
    Mongo(mongodb::error::Error),
    InvalidInput(String),
    ApiError(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AgentError::MissingEnvVar(var) => write!(f, "Environment variable '{}' not found", var),
            AgentError::InvalidConfig(field, msg) => write!(f, "Invalid value for {}: {}", field, msg),
            AgentError::TwitterApi(msg) => write!(f, "Twitter API error: {}", msg),
            AgentError::Trading(msg) => write!(f, "Trading error: {}", msg),
            AgentError::Database(err) => write!(f, "Database error: {}", err),
            AgentError::MarketAnalysis(msg) => write!(f, "Market analysis error: {}", msg),
            AgentError::VectorStore(msg) => write!(f, "Vector store error: {}", msg),
            AgentError::BirdeyeApi(msg) => write!(f, "Birdeye API error: {}", msg),
            AgentError::Transaction(msg) => write!(f, "Transaction error: {}", msg),
            AgentError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AgentError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AgentError::RateLimit(service) => write!(f, "Rate limit exceeded for {}", service),
            AgentError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            AgentError::Network(msg) => write!(f, "Network error: {}", msg),
            AgentError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            AgentError::Conversion(msg) => write!(f, "Conversion error: {}", msg),
            AgentError::Other(err) => write!(f, "Other error: {}", err),
            AgentError::Mongo(err) => write!(f, "MongoDB error: {}", err),
            AgentError::InvalidInput(err) => write!(f, "Input error: {}", err),
            AgentError::ApiError(err) => write!(f, "Api error: {}", err),
        }
    }
}

impl StdError for AgentError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            AgentError::Database(err) => Some(err),
            AgentError::Mongo(err) => Some(err),
            _ => None,
        }
    }
}

impl From<MongoError> for AgentError {
    fn from(err: MongoError) -> Self {
        AgentError::Mongo(err)
    }
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
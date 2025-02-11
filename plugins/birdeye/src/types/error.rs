use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum BirdeyeError {
    #[error("HTTP request failed: {0}")]
    RequestError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Invalid time interval: {0}")]
    InvalidTimeInterval(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
}

impl From<reqwest::Error> for BirdeyeError {
    fn from(err: reqwest::Error) -> Self {
        BirdeyeError::RequestError(err.to_string())
    }
}

impl From<serde_json::Error> for BirdeyeError {
    fn from(err: serde_json::Error) -> Self {
        BirdeyeError::SerializationError(err.to_string())
    }
}

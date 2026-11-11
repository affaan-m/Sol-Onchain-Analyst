use anyhow::{anyhow, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client,
};
use serde_json::json;
use tracing::{error, info};

// Remove trait definition since we're not using trait objects
pub struct TwitterClient {
    client: Client,
    email: String,
    username: String,
    password: String,
    auth_token: Option<String>,
}

impl TwitterClient {
    pub fn new(email: String, username: String, password: String) -> Self {
        Self {
            client: Client::new(),
            email,
            username,
            password,
            auth_token: None,
        }
    }

    pub async fn login(&mut self) -> Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Direct auth endpoint
        let payload = json!({
            "email": self.email,
            "username": self.username,
            "password": self.password
        });

        let response = self
            .client
            .post("https://x.com/i/flow/login")
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            // Extract auth token from cookies
            if let Some(cookies) = response.headers().get("set-cookie") {
                if let Ok(cookie_str) = cookies.to_str() {
                    if let Some(auth_token) = extract_auth_token(cookie_str) {
                        info!("Successfully logged in to Twitter");
                        self.auth_token = Some(auth_token);
                        return Ok(());
                    }
                }
            }
            Err(anyhow!("No auth token found in response"))
        } else {
            let error_message = response.text().await.unwrap_or_default();
            error!("Failed to login to Twitter: {}", error_message);
            Err(anyhow!("Failed to login to Twitter: {}", error_message))
        }
    }

    pub async fn post_tweet(&self, text: &str) -> Result<()> {
        if self.auth_token.is_none() {
            return Err(anyhow!("Not authenticated"));
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Add auth token cookie
        headers.insert(
            "cookie",
            HeaderValue::from_str(&format!("auth_token={}", self.auth_token.as_ref().unwrap()))?,
        );

        let payload = json!({
            "text": text,
            "queryId": "PvJGyyJKzm2-aIsTo6tLSg"  // Twitter's internal query ID for posting tweets
        });

        let response = self
            .client
            .post("https://x.com/i/api/graphql/PvJGyyJKzm2-aIsTo6tLSg/CreateTweet")
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Successfully posted tweet");
            Ok(())
        } else {
            let error_message = response.text().await.unwrap_or_default();
            error!("Failed to post tweet: {}", error_message);
            Err(anyhow!("Failed to post tweet: {}", error_message))
        }
    }

    pub async fn delete_tweet(&self, tweet_id: &str) -> Result<()> {
        if self.auth_token.is_none() {
            return Err(anyhow!("Not authenticated"));
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Add auth token cookie
        headers.insert(
            "cookie",
            HeaderValue::from_str(&format!("auth_token={}", self.auth_token.as_ref().unwrap()))?,
        );

        let payload = json!({
            "tweet_id": tweet_id,
            "queryId": "VaenaVgh5q5ih7kvyVjgtg"  // Twitter's internal query ID for deleting tweets
        });

        let response = self
            .client
            .post("https://x.com/i/api/graphql/VaenaVgh5q5ih7kvyVjgtg/DeleteTweet")
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Successfully deleted tweet {}", tweet_id);
            Ok(())
        } else {
            let error_message = response.text().await.unwrap_or_default();
            error!("Failed to delete tweet {}: {}", tweet_id, error_message);
            Err(anyhow!("Failed to delete tweet: {}", error_message))
        }
    }
}

// Helper function to extract auth token from cookies
fn extract_auth_token(cookie_str: &str) -> Option<String> {
    cookie_str
        .split(';')
        .find(|s| s.trim().starts_with("auth_token="))
        .and_then(|s| s.trim().strip_prefix("auth_token="))
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_auth_token() {
        let cookie_str = "auth_token=abc123; Path=/; Domain=.x.com; Secure; HttpOnly";
        assert_eq!(extract_auth_token(cookie_str), Some("abc123".to_string()));
    }

    #[tokio::test]
    async fn test_auth_token_none() {
        let client = TwitterClient::new(
            "test@example.com".to_string(),
            "testuser".to_string(),
            "password".to_string(),
        );

        // Test that unauthorized operations fail
        let tweet_result = client.post_tweet("Test tweet").await;
        assert!(tweet_result.is_err());

        let delete_result = client.delete_tweet("123").await;
        assert!(delete_result.is_err());
    }
}

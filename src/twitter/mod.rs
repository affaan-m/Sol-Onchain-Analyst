use anyhow::{Result, anyhow};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::env;
use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

#[async_trait]
#[cfg_attr(test, automock)]
pub trait TwitterApi: Send + Sync {
    async fn login(&mut self) -> Result<()>;
    async fn post_tweet(&self, text: &str) -> Result<()>;
    async fn delete_tweet(&self, tweet_id: &str) -> Result<()>;
}

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
}

#[async_trait]
impl TwitterApi for TwitterClient {
    async fn login(&mut self) -> Result<()> {
        // First try to get auth token from environment
        if let Ok(token) = env::var("TWITTER_AUTH_TOKEN") {
            self.auth_token = Some(token);
            return Ok(());
        }

        // If no auth token in env, try to authenticate using username/password
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = json!({
            "username": self.username,
            "password": self.password
        });

        let response = self.client
            .post("https://api.twitter.com/2/oauth2/token")
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(token) = data.get("access_token").and_then(|t| t.as_str()) {
                self.auth_token = Some(token.to_string());
                Ok(())
            } else {
                Err(anyhow!("No access token in response"))
            }
        } else {
            let error_message = response.text().await.unwrap_or_default();
            tracing::error!("Failed to login to Twitter: {}", error_message);
            Err(anyhow!("Failed to login to Twitter: {}", error_message))
        }
    }

    async fn post_tweet(&self, text: &str) -> Result<()> {
        if self.auth_token.is_none() {
            return Err(anyhow!("Not authenticated"));
        }

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.auth_token.as_ref().unwrap()))?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = json!({
            "text": text
        });

        let response = self.client
            .post("https://api.twitter.com/2/tweets")
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_message = response.text().await.unwrap_or_default();
            tracing::error!("Failed to post tweet: {}", error_message);
            Err(anyhow!("Failed to post tweet: {}", error_message))
        }
    }

    async fn delete_tweet(&self, tweet_id: &str) -> Result<()> {
        if self.auth_token.is_none() {
            return Err(anyhow!("Not authenticated"));
        }

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.auth_token.as_ref().unwrap()))?);

        let response = self.client
            .delete(&format!("https://api.twitter.com/2/tweets/{}", tweet_id))
            .headers(headers)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_message = response.text().await.unwrap_or_default();
            tracing::error!("Failed to delete tweet: {}", error_message);
            Err(anyhow!("Failed to delete tweet: {}", error_message))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_twitter_client() -> Result<()> {
        let mut mock = MockTwitterApi::new();
        
        // Setup expectations
        mock.expect_login()
            .times(1)
            .returning(|| Box::pin(async { Ok(()) }));
            
        mock.expect_post_tweet()
            .with(eq("Test tweet"))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));
            
        mock.expect_delete_tweet()
            .with(eq("123456789"))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        // Execute test
        mock.login().await?;
        mock.post_tweet("Test tweet").await?;
        mock.delete_tweet("123456789").await?;

        Ok(())
    }
}


use solagent_core::{serde_json::Value, SolanaAgentKit};
use std::error::Error;

/// Retrieve popular content matching search query, created in time range {from} - {to} (YYYY-MM-DD dates).
///
/// # Parameters
///
/// - `agent`: An instance of `SolanaAgentKit`.
/// - `search_query`: Word or phrase to be searched for in text
/// - `from`: Only consider content created after given date, eg. 2025-01-01
/// - `to`: Only consider content created before given date, eg. 2025-01-20
///
/// # Returns
///
/// A `Result` that tweets details
pub async fn search_tweets(
    agent: &SolanaAgentKit,
    tweets: &str,
    from: &str,
    to: &str,
) -> Result<Value, Box<dyn Error>> {
    // Get the Cookie API key from the agent's configuration
    let api_key = match agent.config.cookie_api_key.as_ref() {
        Some(key) => key,
        None => return Err("Missing Cookie API key in agent.config.cookie_api_key".into()),
    };

    let api_url = "https://api.cookie.fun/v1/hackathon/search";
    let url = format!("{}/{}?from={}&to={}", api_url, tweets, from, to);
    let client = reqwest::Client::new();

    let response = client.get(&url).header("x-api-key", api_key).send().await?;

    let json: Value = response.json().await?;
    Ok(json)
}

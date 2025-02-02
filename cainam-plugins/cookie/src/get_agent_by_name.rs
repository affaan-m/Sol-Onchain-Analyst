

use solagent_core::{serde_json::Value, SolanaAgentKit};
use std::error::Error;

/// Retrieve agent details in specified interval by twitter username.
///
/// # Parameters
///
/// - `agent`: An instance of `SolanaAgentKit`.
/// - `twitter_name`: Twitter username of agent (matches case insensitive)
/// - `interval`: An optional Interval for twitter stats and deltas (_3Days, _7Days). If not provided, returns the _7Days.
///
/// # Returns
///
/// A `Result` that agent details
pub async fn get_agent_by_name(
    agent: &SolanaAgentKit,
    twitter_name: &str,
    interval: Option<u32>,
) -> Result<Value, Box<dyn Error>> {
    // Get the Cookie API key from the agent's configuration
    let api_key = match agent.config.cookie_api_key.as_ref() {
        Some(key) => key,
        None => return Err("Missing Cookie API key in agent.config.cookie_api_key".into()),
    };

    let api_url = "https://api.cookie.fun/v2/agents/twitterUsername";
    let url = format!("{}/{}?interval=_{}Days", api_url, twitter_name, interval.unwrap_or(7));
    let client = reqwest::Client::new();

    let response = client.get(&url).header("x-api-key", api_key).send().await?;

    let json: Value = response.json().await?;
    Ok(json)
}

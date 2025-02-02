

use serde::{Deserialize, Serialize};
use solagent_core::SolanaAgentKit;

#[derive(Debug, Serialize, Deserialize)]
pub struct HeliusWebhookIdResponse {
    pub wallet: String,
    pub webhook_url: String,
    pub transaction_types: Vec<String>,
    pub account_addresses: Vec<String>,
    pub webhook_type: String,
}

/// Retrieves a Helius Webhook by ID, returning only the specified fields.
///
/// # Arguments
/// * `agent` - An instance of SolanaAgentKit (with .config.HELIUS_API_KEY)
/// * `webhook_id` - The unique ID of the webhook to delete
///
/// # Returns
/// A HeliusWebhook object containing { wallet, webhookURL, transactionTypes, accountAddresses, webhookType }
pub async fn get_webhook(
    agent: &SolanaAgentKit,
    webhook_id: &str,
) -> Result<HeliusWebhookIdResponse, Box<dyn std::error::Error>> {
    // Get the Helius API key from the agent's configuration
    let api_key = match agent.config.helius_api_key.as_ref() {
        Some(key) => key,
        None => return Err("Missing Helius API key in agent.config.HELIUS_API_KEY".into()),
    };

    let client = reqwest::Client::new();
    let url = format!("https://api.helius.xyz/v0/webhooks/{}?api-key={}", webhook_id, api_key);

    let response = client.get(url).header("Content-Type", "application/json").send().await?;

    let data = response.json::<HeliusWebhookIdResponse>().await?;
    Ok(data)
}

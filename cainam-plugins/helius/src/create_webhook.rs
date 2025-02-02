

use serde::{Deserialize, Serialize};
use solagent_core::SolanaAgentKit;

#[derive(Deserialize, Serialize)]
pub struct HeliusWebhookResponse {
    pub webhook_url: String,
    pub webhook_id: String,
}

pub async fn create_webhook(
    agent: &SolanaAgentKit,
    account_addresses: Vec<String>,
    webhook_url: String,
) -> Result<HeliusWebhookResponse, Box<dyn std::error::Error>> {
    // Get the Helius API key from the agent's configuration
    let api_key = match agent.config.helius_api_key.as_ref() {
        Some(key) => key,
        None => return Err("Missing Helius API key in agent.config.HELIUS_API_KEY".into()),
    };

    let url = format!("https://api.helius.xyz/v0/webhooks?api-key={}", api_key);

    let body = serde_json::json!({
        "webhookURL": webhook_url,
        "transactionTypes": ["Any"],
        "accountAddresses": account_addresses,
        "webhookType": "enhanced",
        "txnStatus": "all",
    });

    let client = reqwest::Client::new();
    let response = client.post(url).header("Content-Type", "application/json").json(&body).send().await?;

    let data = response.json::<serde_json::Value>().await?;
    let webhook_url = data.get("webhookURL").expect("webhookURL field").as_str().expect("webhookURL text");
    let webhook_id = data.get("webhookID").expect("webhookID field").as_str().expect("webhookID text");

    Ok(HeliusWebhookResponse { webhook_url: webhook_url.to_string(), webhook_id: webhook_id.to_string() })
}

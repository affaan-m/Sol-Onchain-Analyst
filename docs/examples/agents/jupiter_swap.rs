

use solagent::{Config, SolanaAgentKit};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = Config { openai_api_key: Some("your_api_key".to_string()), ..Default::default() };
    let agent = Arc::new(SolanaAgentKit::new("private_key", "RPC_URL", config));
    //swap 0.01 SOL to USDC
    let swap = agent
        .trade(
            Some("So11111111111111111111111111111111111111112".to_string()),
            0.01,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            None,
        )
        .await
        .unwrap();
    println!("Signature: {}", swap);
}

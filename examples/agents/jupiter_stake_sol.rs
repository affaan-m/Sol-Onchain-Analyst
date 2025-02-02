

use solagent::{Config, SolanaAgentKit};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = Config { openai_api_key: Some("your_api_key".to_string()), ..Default::default() };
    let agent = Arc::new(SolanaAgentKit::new("private_key", "RPC_URL", config));
    //stake 0.01 SOL
    let stake = agent.stake_with_jup(0.01).await.unwrap();
    println!("Signature: {}", stake);
}

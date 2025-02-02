

use solagent::{Config, SolanaAgentKit};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = Config { openai_api_key: Some("your_api_key".to_string()), ..Default::default() };
    let agent = Arc::new(SolanaAgentKit::new("private_key", "RPC_URL", config));
    let data = agent.close_empty_token_accounts().await.unwrap();
    println!("Close data: {:?}", data);
}

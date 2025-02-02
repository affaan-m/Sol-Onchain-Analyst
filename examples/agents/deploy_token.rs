

use solagent::{Config, SolanaAgentKit};
use std::sync::Arc;

/// Example on devnet
/// Mint: 3kvSrsPwtYi6RkWymJocQcezwiDpqMfDjWazYAaibDmY
#[tokio::main]
async fn main() {
    let name = "Solagent".to_string();
    let uri = "solagent.rs".to_string();
    let symbol = "SOLA".to_string();
    let decimals = 1;
    let initial_supply = 1_000_000_000_u64;

    let config = Config { openai_api_key: Some("your_api_key".to_string()), ..Default::default() };
    let agent = Arc::new(SolanaAgentKit::new("private_key", "RPC_URL", config));
    let data = agent.deploy_token(name, uri, symbol, decimals, Some(initial_supply)).await;
    println!("Mint data: {:?}", data);
}



use solagent::{Config, SolanaAgentKit};

#[tokio::main]
async fn main() {
    let chain_id = "42161";
    let mint = "0xEa51801b8F5B88543DdaD3D1727400c15b209D8f";

    let config = Config { openai_api_key: Some("your_api_key".to_string()), ..Default::default() };
    let agent = SolanaAgentKit::new("private_key", "RPC_URL", config);
    let check = agent.get_token_security_info(chain_id, mint).await.unwrap();
    println!("Token check: {:?}", check);
}

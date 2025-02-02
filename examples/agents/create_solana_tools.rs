

use solagent::{create_solana_tools, Config, SolanaAgentKit};

#[tokio::main]
async fn main() {
    let config = Config { openai_api_key: Some("your_api_key".to_string()), ..Default::default() };
    let agent = SolanaAgentKit::new("private_key", "RPC_URL", config);
    let _tools = create_solana_tools(agent);
}

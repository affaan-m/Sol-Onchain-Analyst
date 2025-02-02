

use solagent::{Config, SolanaAgentKit};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let mint = "84VUXykQjNvPDm88oT5FRucXeNcrwdQGottJKjkAoqd1".into();

    let config = Config { openai_api_key: Some("your_api_key".to_string()), ..Default::default() };
    let agent = Arc::new(SolanaAgentKit::new("private_key", "RPC_URL", config));
    let check = agent.fetch_summary_report(mint).await.unwrap();
    println!("Token check: {:?}", check);
}

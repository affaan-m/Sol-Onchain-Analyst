

use solagent::{Config, NFTMetadata, SolanaAgentKit};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

/// Example on devnet
/// Mint: HHV3DX4UT4u3vBek2XCaZeAyox88zuhWfcLRJbFx1oYt
#[tokio::main]
async fn main() {
    let name = "Solagent Collection";
    let uri = "uri";
    let royalty_basis_points = Some(500);
    let creators = vec![(Pubkey::from_str_const("pubkey"), 100)];
    let options = NFTMetadata::new(name, uri, royalty_basis_points, Some(creators));

    let config = Config { openai_api_key: Some("your_api_key".to_string()), ..Default::default() };
    let agent = Arc::new(SolanaAgentKit::new("private_key", "RPC_URL", config));
    let data = agent.deploy_collection(options).await.unwrap();
    println!("Deploy Data: {:?}", data);
}

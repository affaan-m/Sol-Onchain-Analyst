

use solagent::{Config, NFTMetadata, SolanaAgentKit};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

/// Example on devnet
/// Mint: 5jcsea3EA3kX7mXpy7YvHVFYTDEJeSEXjyicgThnvWUm
/// https://explorer.solana.com/address/5jcsea3EA3kX7mXpy7YvHVFYTDEJeSEXjyicgThnvWUm?cluster=devnet
#[tokio::main]
async fn main() {
    let name = "My First SolanaAgentKit NFT";
    let uri = "uri";
    let royalty_basis_points = Some(500);
    let creators = vec![(Pubkey::from_str_const("pubkey"), 100)];
    let metadata = NFTMetadata::new(name, uri, royalty_basis_points, Some(creators));

    let collection = Pubkey::from_str_const("collection Mint");

    let config = Config { openai_api_key: Some("your_api_key".to_string()), ..Default::default() };
    let agent = Arc::new(SolanaAgentKit::new("private_key", "RPC_URL", config));
    let deployed_data = agent.mint_nft_to_collection(collection, metadata).await.unwrap();
    println!("Mint: {}", deployed_data.mint);
}

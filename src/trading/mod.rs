pub mod trading_engine;

use anyhow::Result;
use solana_client::rpc_client::RpcClient;

pub struct SolanaAgentKit {
    rpc_client: RpcClient,
    wallet_keypair: solana_sdk::signer::keypair::Keypair,
}

impl SolanaAgentKit {
    pub fn new(rpc_url: &str, wallet_keypair: solana_sdk::signer::keypair::Keypair) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url.to_string()),
            wallet_keypair,
        }
    }

    pub fn new_from_env() -> Result<Self> {
        let rpc_url = std::env::var("SOLANA_RPC_URL")?;
        let wallet_key = std::env::var("SOLANA_PRIVATE_KEY")?;

        // Parse the base58 private key
        let wallet_keypair = solana_sdk::signer::keypair::Keypair::from_base58_string(&wallet_key);

        Ok(Self::new(&rpc_url, wallet_keypair))
    }

    pub fn get_rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    pub fn get_wallet_keypair(&self) -> &solana_sdk::signer::keypair::Keypair {
        &self.wallet_keypair
    }
}

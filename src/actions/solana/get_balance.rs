use crate::SolanaAgentKit;
use anyhow::{Context, Result};
use solana_client::client_error::ClientError;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use std::str::FromStr;

/// Gets the balance of SOL or an SPL token for the agent's wallet.
///
/// # Parameters
///
/// - `agent`: An instance of `SolanaAgentKit`.
/// - `token_address`: An optional SPL token mint address. If not provided, returns the SOL balance.
///
/// # Returns
///
/// A `Result` that resolves to the balance as a number (in UI units) or an error if the account doesn't exist.
pub async fn get_balance(agent: &SolanaAgentKit, token_address: Option<String>) -> Result<f64, ClientError> {
    if let Some(token_address) = token_address {
        // Get SPL token account balance
        let pubkey = Pubkey::from_str(&token_address).map_err(|e| {
            ClientError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid token address: {}: {}", token_address, e),
            ))
        })?;

        let token_account = agent.connection.get_token_account_balance(&pubkey)?;
        let ui_amount = token_account.ui_amount.unwrap_or(0.0);
        return Ok(ui_amount);
    }

    // Get SOL balance
    let balance = agent.connection.get_balance(&agent.wallet.address)?;
    Ok(balance as f64 / LAMPORTS_PER_SOL as f64)
}


// Integration Tests:
#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;

    #[tokio::test]
    async fn test_get_sol_balance() -> Result<()> {
        // Replace with a testnet/devnet RPC URL and a wallet with known balance
        let rpc_url = "https://api.devnet.solana.com";
        let wallet = Keypair::new(); // Create a new keypair for testing
        let wallet_address = wallet.pubkey().to_string();
        Ok(())
    }

    #[tokio::test]
    async fn test_get_spl_token_balance() -> Result<()> {
        // Replace with a testnet/devnet RPC URL and a token with known balance
        let rpc_url = "https://api.devnet.solana.com";
        let wallet_address = "A5GecEJ";
        let token_address = "So11111111111111111111111111111111111111112";
        Ok(())
    }
}

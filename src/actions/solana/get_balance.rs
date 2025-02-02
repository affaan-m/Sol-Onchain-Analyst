use crate::SolanaAgentKit;
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
        if let Ok(pubkey) = Pubkey::from_str(&token_address) {
            let token_account = agent.connection.get_token_account_balance(&pubkey)?;
            let ui_amount = token_account.ui_amount.unwrap_or(0.0);
            return Ok(ui_amount);
        }
    }

    // Get SOL balance
    let balance = agent.connection.get_balance(&agent.wallet.address)?;
    Ok(balance as f64 / LAMPORTS_PER_SOL as f64)
}

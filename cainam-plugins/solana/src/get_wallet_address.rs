

use solagent_core::SolanaAgentKit;

/// Get the agent's wallet address.
///
/// # Parameters
/// - `agent`: A `SolanaAgentKit` instance.
///
/// # Returns
/// A string representing the wallet address in base58 format.
pub fn get_wallet_address(agent: &SolanaAgentKit) -> String {
    agent.wallet.address.to_string()
}

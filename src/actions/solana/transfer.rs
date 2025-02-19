use crate::SolanaAgentKit;
use anyhow::{Context, Result};
use solana_client::client_error::ClientError;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey, system_instruction, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_token::{instruction::transfer as transfer_instruct, state::Mint};
use std::str::FromStr;

/// Transfer SOL or SPL tokens to a recipient
///
/// `agent` - SolanaAgentKit instance
/// `to` - Recipient's public key
/// `amount` - Amount to transfer
/// `mint` - Optional mint address for SPL tokens
///
/// Returns the transaction signature.
pub async fn transfer(
    agent: &SolanaAgentKit,
    to: &str,
    amount: u64,
    mint: Option<String>,
) -> Result<String, ClientError> {
    match mint {
        Some(mint) => {
            // Transfer SPL Token
            let mint_pubkey = Pubkey::from_str(&mint).map_err(|e| {
                ClientError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid mint address: {}: {}", mint, e),
                ))
            })?;
            let to_pubkey = Pubkey::from_str(to).map_err(|e| {
                ClientError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid recipient address: {}: {}", to, e),
                ))
            })?;

            let from_ata = get_associated_token_address(&agent.wallet.address, &mint_pubkey);
            let to_ata = get_associated_token_address(&to_pubkey, &mint_pubkey);

            let account_info = agent.connection.get_account(&mint_pubkey).map_err(|e| {
                ClientError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to get account info for mint {}: {}", mint, e),
                ))
            })?;
            let mint_info = Mint::unpack_from_slice(&account_info.data).map_err(|e| {
                ClientError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to unpack mint info for {}: {}", mint, e),
                ))
            })?;

            let adjusted_amount = amount * 10u64.pow(mint_info.decimals as u32);

            let transfer_instruction = transfer_instruct(
                &spl_token::id(),
                &from_ata,
                &to_ata,
                &agent.wallet.address,
                &[&agent.wallet.address],
                adjusted_amount,
            )
            .map_err(|e| {
                ClientError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create transfer instruction: {}", e),
                ))
            })?;

            let recent_blockhash = agent
                .connection
                .get_latest_blockhash()
                .map_err(|e| ClientError::from(e))?;

            let transaction = Transaction::new_signed_with_payer(
                &[transfer_instruction],
                Some(&agent.wallet.address),
                &[&agent.wallet.wallet],
                recent_blockhash,
            );

            let signature = agent
                .connection
                .send_and_confirm_transaction(&transaction)
                .map_err(|e| ClientError::from(e))?;
            Ok(signature.to_string())
        }
        None => {
            // Transfer SOL
            let to_pubkey = Pubkey::from_str(to).map_err(|e| {
                ClientError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid recipient address: {}: {}", to, e),
                ))
            })?;
            let transfer_instruction =
                system_instruction::transfer(&agent.wallet.address, &to_pubkey, amount);

            let recent_blockhash = agent
                .connection
                .get_latest_blockhash()
                .map_err(|e| ClientError::from(e))?;

            let transaction = Transaction::new_signed_with_payer(
                &[transfer_instruction],
                Some(&agent.wallet.address),
                &[&agent.wallet.wallet],
                recent_blockhash,
            );

            let signature = agent
                .connection
                .send_and_confirm_transaction(&transaction)
                .map_err(|e| ClientError::from(e))?;
            Ok(signature.to_string())
        }
    }
}

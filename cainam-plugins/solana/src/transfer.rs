

use solagent_core::{
    solana_client::client_error::ClientError,
    solana_sdk::{program_pack::Pack, pubkey::Pubkey, system_instruction, transaction::Transaction},
    SolanaAgentKit,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::{instruction::transfer as transfer_instruct, state::Mint};

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
            let mint = Pubkey::from_str_const(&mint);
            let to = Pubkey::from_str_const(to);

            let from_ata = get_associated_token_address(&mint, &agent.wallet.address);
            let to_ata = get_associated_token_address(&mint, &to);

            let account_info = &agent.connection.get_account(&mint).expect("get_account");
            let mint_info = Mint::unpack_from_slice(&account_info.data).expect("unpack_from_slice");

            let adjusted_amount = amount * 10u64.pow(mint_info.decimals as u32);

            let transfer_instruction = transfer_instruct(
                &spl_token::id(),
                &from_ata,
                &to_ata,
                &from_ata,
                &[&agent.wallet.address],
                adjusted_amount,
            )
            .expect("transfer_instruct");

            let transaction = Transaction::new_signed_with_payer(
                &[transfer_instruction],
                Some(&agent.wallet.address),
                &[&agent.wallet.wallet],
                agent.connection.get_latest_blockhash().expect("new_signed_with_payer"),
            );

            let signature =
                agent.connection.send_and_confirm_transaction(&transaction).expect("send_and_confirm_transaction");
            Ok(signature.to_string())
        }
        None => {
            let transfer_instruction =
                system_instruction::transfer(&agent.wallet.address, &Pubkey::from_str_const(to), amount);
            let transaction = Transaction::new_signed_with_payer(
                &[transfer_instruction],
                Some(&agent.wallet.address),
                &[&agent.wallet.wallet],
                agent.connection.get_latest_blockhash().expect("get_latest_blockhash"),
            );

            let signature =
                agent.connection.send_and_confirm_transaction(&transaction).expect("send_and_confirm_transaction");
            Ok(signature.to_string())
        }
    }
}

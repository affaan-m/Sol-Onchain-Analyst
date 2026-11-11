use crate::{primitives::USDC, SolanaAgentKit};
use anyhow::{anyhow, Context, Result};
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, transaction::Transaction};
use spl_token::instruction::close_account;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize)]
pub struct Parsed {
    pub info: SplToken,
}

#[derive(serde::Deserialize)]
pub struct SplToken {
    pub mint: String,
    #[serde(rename(deserialize = "tokenAmount"))]
    pub token_amount: Amount,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct Amount {
    pub amount: String,
    #[serde(rename(deserialize = "uiAmountString"))]
    ui_amount_string: String,
    #[serde(rename(deserialize = "uiAmount"))]
    pub ui_amount: f64,
    pub decimals: u8,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CloseEmptyTokenAccountsData {
    pub signature: String,
    pub closed_size: usize,
}

impl CloseEmptyTokenAccountsData {
    pub fn new(signature: String, closed_size: usize) -> Self {
        CloseEmptyTokenAccountsData {
            signature,
            closed_size,
        }
    }
}

/// Close Empty SPL Token accounts of the agent.
///
/// # Parameters
///
/// - `agent`: An instance of `SolanaAgentKit`.
///
/// # Returns
///
/// Transaction signature and total number of accounts closed or an error if the account doesn't exist.
pub async fn close_empty_token_accounts(
    agent: &SolanaAgentKit,
) -> Result<CloseEmptyTokenAccountsData> {
    let max_instructions = 40_u32;
    let mut transaction: Vec<Instruction> = vec![];
    let mut closed_size = 0;
    let token_programs = vec![spl_token::ID, spl_token_2022::ID];

    for token_program in token_programs {
        let accounts = agent
            .connection
            .get_token_accounts_by_owner(
                &agent.wallet.address,
                TokenAccountsFilter::ProgramId(token_program.to_owned()),
            )
            .context("Failed to get token accounts by owner")?;

        closed_size += accounts.len();

        for account in accounts {
            if transaction.len() >= max_instructions as usize {
                break;
            }

            if let solana_account_decoder::UiAccountData::Json(d) = &account.account.data {
                let parsed = serde_json::from_value::<Parsed>(d.parsed.clone())
                    .context("Failed to parse token account data")?;

                if parsed.info.token_amount.amount.parse::<u32>().unwrap_or(0) == 0_u32
                    && parsed.info.mint != USDC
                {
                    let account_pubkey = Pubkey::from_str(&account.pubkey)
                        .context("Failed to parse account pubkey")?;

                    let instruct = close_account(
                        &token_program,
                        &account_pubkey,
                        &agent.wallet.address,
                        &agent.wallet.address,
                        &[&agent.wallet.address],
                    )
                    .context("Failed to create close_account instruction")?;
                    transaction.push(instruct);
                }
            }
        }
    }

    if transaction.is_empty() {
        return Ok(CloseEmptyTokenAccountsData::default());
    }

    let recent_blockhash = agent
        .connection
        .get_latest_blockhash()
        .context("Failed to get latest blockhash")?;
    let transaction = Transaction::new_signed_with_payer(
        &transaction,
        Some(&agent.wallet.address),
        &[&agent.wallet.wallet],
        recent_blockhash,
    );

    let signature = agent
        .connection
        .send_and_confirm_transaction(&transaction)
        .context("Failed to send and confirm transaction")?;
    let data = CloseEmptyTokenAccountsData::new(signature.to_string(), closed_size);
    Ok(data)
}

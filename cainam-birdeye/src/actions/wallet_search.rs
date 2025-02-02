use async_trait::async_trait;
use rig::{Action, ActionContext, ActionResult};
use serde::{Deserialize, Serialize};
use crate::{providers::BirdeyeProvider, types::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSearchAction {
    pub wallet: String,
}

#[async_trait]
impl Action for WalletSearchAction {
    type Output = WalletPortfolio;
    type Error = BirdeyeError;

    fn name(&self) -> &'static str {
        "wallet_search"
    }

    fn description(&self) -> &'static str {
        "Get wallet portfolio information from Birdeye"
    }

    async fn execute(&self, ctx: &ActionContext) -> ActionResult<Self::Output, Self::Error> {
        let api_key = ctx.get_secret("BIRDEYE_API_KEY")
            .ok_or_else(|| BirdeyeError::InvalidApiKey)?;

        let provider = BirdeyeProvider::new(api_key);

        let portfolio = provider.get_wallet_portfolio(&self.wallet).await?;

        Ok(portfolio)
    }
} 
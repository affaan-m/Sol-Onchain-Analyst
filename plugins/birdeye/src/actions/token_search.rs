use async_trait::async_trait;
use rig::{Action, ActionContext, ActionResult};
use serde::{Deserialize, Serialize};
use crate::{providers::BirdeyeProvider, types::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSearchAction {
    pub keyword: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<TokenSortBy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_type: Option<SortType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[async_trait]
impl Action for TokenSearchAction {
    type Output = Vec<TokenMarketData>;
    type Error = BirdeyeError;

    fn name(&self) -> &'static str {
        "token_search"
    }

    fn description(&self) -> &'static str {
        "Search for tokens on Solana using Birdeye"
    }

    async fn execute(&self, ctx: &ActionContext) -> ActionResult<Self::Output, Self::Error> {
        let api_key = ctx.get_secret("BIRDEYE_API_KEY")
            .ok_or_else(|| BirdeyeError::InvalidApiKey)?;

        let provider = BirdeyeProvider::new(api_key);

        let params = TokenSearchParams {
            keyword: self.keyword.clone(),
            sort_by: self.sort_by,
            sort_type: self.sort_type,
            offset: None,
            limit: self.limit,
        };

        let tokens = provider.search_tokens(params).await?;

        Ok(tokens)
    }
} 
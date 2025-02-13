pub mod providers;
pub mod types;

use crate::providers::{birdeye::BirdeyeProvider, cache::CachedClient};
use crate::types::{
    api::{
        LiquidityAnalysis, MarketImpact, PricePoint, TokenInfo, TokenOverview, TokenSearchParams,
        WalletPortfolio,
    },
    error::BirdeyeError,
    TimeInterval,
};
use anyhow::Result;
use types::api::WalletPortfolioResponse;
use std::sync::Arc;

pub struct BirdeyeClient {
    provider: Arc<CachedClient<BirdeyeProvider>>,
}

impl BirdeyeClient {
    pub fn new(api_key: String) -> Self {
        let provider = BirdeyeProvider::new(api_key);
        Self {
            provider: Arc::new(CachedClient::new(provider)),
        }
    }

    pub async fn search_tokens(
        &self,
        params: TokenSearchParams,
    ) -> Result<Vec<TokenInfo>, BirdeyeError> {
        let cache_key = format!("search_tokens_{:?}", params);
        let params = params.clone();

        self.provider
            .execute(
                &cache_key,
                move |provider| async move { provider.search_tokens(params).await },
                60,
            )
            .await
    }

    pub async fn get_token_overview(&self, address: String) -> Result<TokenOverview, BirdeyeError> {
        let cache_key = format!("overview:{}", address);
        let address = address.clone();

        self.provider
            .execute(
                &cache_key,
                move |provider| async move { provider.get_token_overview(address).await },
                60,
            )
            .await
    }

    pub async fn analyze_liquidity(
        &self,
        address: String,
    ) -> Result<LiquidityAnalysis, BirdeyeError> {
        let cache_key = format!("liquidity:{}", address);
        let address = address.clone();

        self.provider
            .execute(
                &cache_key,
                move |provider| async move { provider.analyze_liquidity(&address).await },
                60,
            )
            .await
    }

    pub async fn get_market_impact(
        &self,
        address: String,
        size_usd: f64,
    ) -> Result<MarketImpact, BirdeyeError> {
        let cache_key = format!("impact:{}:{}", address, size_usd);
        let address = address.clone();

        self.provider
            .execute(
                &cache_key,
                move |provider| async move { provider.get_market_impact(&address, size_usd).await },
                60,
            )
            .await
    }

    pub async fn get_price_history(
        &self,
        address: String,
        interval: TimeInterval,
    ) -> Result<Vec<PricePoint>, BirdeyeError> {
        let cache_key = format!("history:{}:{}", address, interval);
        let address = address.clone();

        self.provider
            .execute(
                &cache_key,
                move |provider| async move { provider.get_price_history(&address, interval).await },
                60,
            )
            .await
    }

    pub async fn get_wallet_portfolio(
        &self,
        wallet_address: String,
    ) -> Result<WalletPortfolioResponse, BirdeyeError> {
        let cache_key = format!("portfolio:{}", wallet_address);
        let address = wallet_address.clone();

        self.provider
            .execute(
                &cache_key,
                move |provider| async move { provider.get_wallet_portfolio(&address).await },
                60,
            )
            .await
    }
}

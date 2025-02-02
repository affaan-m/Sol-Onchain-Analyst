#[derive(Debug, Deserialize)]
pub struct TokenMarketResponse {
    pub data: TokenMarketData,
    pub success: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct TokenMarketData {
    pub address: String,
    pub price: f64,
    pub volume_24h: f64,
    pub decimals: u8,
    pub price_sol: f64,
    pub market_cap: f64,
    pub fully_diluted_market_cap: f64,
    pub circulating_supply: f64,
    pub total_supply: f64,
    pub price_change_24h: f64,
    pub volume_change_24h: f64,
}

impl BirdeyeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    pub async fn get_market_data(&self, token_address: &str) -> Result<TokenMarketData, AgentError> {
        let url = format!(
            "https://public-api.birdeye.so/public/market_data?address={}",
            token_address
        );

        let response = self
            .client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await
            .map_err(|e| AgentError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::ApiError(format!(
                "Failed to get market data: {}",
                response.status()
            )));
        }

        let market_data = response
            .json::<TokenMarketResponse>()
            .await
            .map_err(|e| AgentError::ApiError(e.to_string()))?;

        if !market_data.success {
            return Err(AgentError::ApiError("Token not found".to_string()));
        }

        Ok(market_data.data)
    }

    pub async fn get_token_info_by_address(&self, token_address: &str) -> Result<TokenInfo, AgentError> {
        let market_data = self.get_market_data(token_address).await?;

        Ok(TokenInfo {
            address: market_data.address,
            price: market_data.price,
            volume_24h: market_data.volume_24h,
            decimals: market_data.decimals,
            price_sol: market_data.price_sol,
            market_cap: market_data.market_cap,
            fully_diluted_market_cap: market_data.fully_diluted_market_cap,
            circulating_supply: market_data.circulating_supply,
            total_supply: market_data.total_supply,
            price_change_24h: market_data.price_change_24h,
            volume_change_24h: market_data.volume_change_24h,
        })
    }
}

#[async_trait]
impl BirdeyeApi for BirdeyeClient {
    async fn get_token_info(&self, token_address: &str) -> Result<TokenInfo, AgentError> {
        self.get_token_info_by_address(token_address).await
    }
} 
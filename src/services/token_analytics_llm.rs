use crate::error::{AgentError, AgentResult};
use crate::models::token_analytics::TokenAnalytics;
use crate::services::token_analytics::TokenAnalyticsService;
use rig::agent::Agent as RigAgent;
use rig::completion::Prompt;
use rig::providers::openai::{Client as OpenAIClient, CompletionModel};
use std::sync::Arc;
use tracing::{debug, error};

pub struct TokenAnalyticsLLM {
    analytics_service: Arc<TokenAnalyticsService>,
    openai_client: OpenAIClient,
    agent: RigAgent<CompletionModel>,
}

impl TokenAnalyticsLLM {
    pub fn new(analytics_service: Arc<TokenAnalyticsService>, openai_api_key: &str) -> Self {
        let openai_client = OpenAIClient::new(openai_api_key);
        let agent = openai_client
            .agent(crate::config::get_openai_model())
            .preamble(include_str!("../prompts/analytics_system.txt"))
            .build();

        Self {
            analytics_service,
            openai_client,
            agent,
        }
    }

    pub async fn analyze_query(&self, query: &str) -> AgentResult<String> {
        debug!("Processing analytics query: {}", query);

        // Get relevant token analytics based on the query
        let analytics = match self.analytics_service.get_relevant_analytics(query).await {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to get relevant analytics: {}", e);
                return Err(AgentError::MarketAnalysis(format!(
                    "Failed to get analytics: {}",
                    e
                )));
            }
        };

        // Format analytics data for LLM
        let formatted_data = self.format_analytics_data(&analytics)?;

        // Create prompt for analysis
        let prompt = format!(
            "Based on the following token analytics data, please answer this question: {}\n\nData:\n{}",
            query,
            formatted_data
        );

        // Get LLM analysis
        match self.agent.prompt(prompt).await {
            Ok(analysis) => Ok(analysis),
            Err(e) => {
                error!("Failed to get LLM analysis: {}", e);
                Err(AgentError::MarketAnalysis(format!(
                    "Failed to get LLM analysis: {}",
                    e
                )))
            }
        }
    }

    pub async fn get_market_insights(&self) -> AgentResult<String> {
        debug!("Generating market insights");

        // Get trending tokens
        let trending = self.analytics_service.get_trending_tokens(10).await?;

        // Format trending data
        let formatted_data = self.format_analytics_data(&trending)?;

        // Create prompt for insights
        let prompt = format!(
            "Please analyze these trending tokens and provide key market insights:\n\n{}",
            formatted_data
        );

        // Get LLM analysis
        let insights = self.agent.prompt(prompt).await.map_err(|e| {
            AgentError::MarketAnalysis(format!("Failed to get market insights: {}", e))
        })?;

        Ok(insights)
    }

    pub async fn compare_tokens(&self, token_addresses: Vec<&str>) -> AgentResult<String> {
        debug!("Comparing tokens: {:?}", token_addresses);

        // Get analytics for all tokens
        let mut analytics = Vec::new();
        for address in token_addresses {
            if let Some(token_data) = self.analytics_service.get_token_analytics(address).await? {
                analytics.push(token_data);
            }
        }

        // Format comparison data
        let formatted_data = self.format_analytics_data(&analytics)?;

        // Create prompt for comparison
        let prompt = format!(
            "Please compare these tokens and provide a detailed analysis of their relative strengths and weaknesses:\n\n{}",
            formatted_data
        );

        // Get LLM analysis
        let comparison =
            self.agent.prompt(prompt).await.map_err(|e| {
                AgentError::MarketAnalysis(format!("Failed to compare tokens: {}", e))
            })?;

        Ok(comparison)
    }

    fn format_analytics_data(&self, analytics: &[TokenAnalytics]) -> AgentResult<String> {
        let mut formatted = String::new();

        for token in analytics {
            formatted.push_str(&format!(
                "Token: {} ({})\n",
                token.token_name, token.token_symbol
            ));
            formatted.push_str(&format!("Address: {}\n", token.token_address));
            formatted.push_str(&format!("Price: ${}\n", token.price));

            if let Some(price_change) = &token.price_change_24h {
                formatted.push_str(&format!("24h Change: {}%\n", price_change));
            }

            if let Some(volume) = &token.volume_24h {
                formatted.push_str(&format!("24h Volume: ${}\n", volume));
            }

            if let Some(market_cap) = &token.market_cap {
                formatted.push_str(&format!("Market Cap: ${}\n", market_cap));
            }

            if let Some(holder_count) = token.holder_count {
                formatted.push_str(&format!("Holders: {}\n", holder_count));
            }

            formatted.push_str("\n");
        }

        Ok(formatted)
    }
}

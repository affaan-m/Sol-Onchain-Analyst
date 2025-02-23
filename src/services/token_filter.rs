use crate::birdeye::api::{BirdeyeApi, TokenV3Response};
use crate::config::mongodb::{MongoDbPool, TokenAnalyticsDataExt};
use anyhow::{Context, Result};
use mongodb::bson::{self, doc, Document};
use rig::{
    completion::{CompletionModel, CompletionRequest},
    message::{Message, UserContent},
    one_or_many::OneOrMany,
    providers::openai::{self, Client as OpenAIClient},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

const DEFAULT_CHUNK_SIZE: i64 = 20;
const INITIAL_FILTER_PROMPT: &str = include_str!("../prompts/token_filter_initial.txt");
const MARKET_FILTER_PROMPT: &str = include_str!("../prompts/token_filter_market.txt");
const METADATA_FILTER_PROMPT: &str = include_str!("../prompts/token_filter_metadata.txt");
const MODEL: &str = openai::O1_MINI;  // Using O1_MINI until O3_MINI is available in RIG

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BirdeyeFilters {
    pub sort_by: String,
    pub sort_type: String,
    pub limit: i64,
    pub offset: Option<i64>,
    #[serde(flatten)]
    pub filters: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenMetrics {
    pub social_metrics: Option<SocialMetrics>,
    pub dev_metrics: Option<DevMetrics>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SocialMetrics {
    pub twitter_quality: f64,
    pub community_engagement: f64,
    pub sentiment: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DevMetrics {
    pub github_activity: f64,
    pub wallet_patterns: f64,
    pub contract_quality: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenAnalysis {
    pub address: String,
    pub symbol: String,
    pub score: f64,
    pub analysis: Analysis,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Analysis {
    pub market_score: f64,
    pub social_score: f64,
    pub dev_score: f64,
    pub risk_score: f64,
    pub metrics: Option<TokenMetrics>,
    pub key_strengths: Vec<String>,
    pub key_risks: Vec<String>,
    pub final_recommendation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterSummary {
    pub total_analyzed: i64,
    pub total_passed: i64,
    pub avg_market_score: f64,
    pub avg_social_score: f64,
    pub avg_dev_score: f64,
    pub avg_risk_score: f64,
    pub market_conditions: String,
    pub risk_assessment: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterResponse {
    pub filtered_tokens: Vec<TokenAnalysis>,
    pub summary: FilterSummary,
}

pub struct TokenFilterService {
    birdeye: Arc<dyn BirdeyeApi>,
    db_pool: Arc<MongoDbPool>,
    openai_client: OpenAIClient,
}

impl TokenFilterService {
    pub fn new(
        birdeye: Arc<dyn BirdeyeApi>,
        db_pool: Arc<MongoDbPool>,
        openai_api_key: &str,
    ) -> Self {
        let openai_client = OpenAIClient::new(openai_api_key);

        Self {
            birdeye,
            db_pool,
            openai_client,
        }
    }

    async fn get_completion(&self, prompt: &str) -> Result<String> {
        debug!("Sending completion request with prompt: {}", prompt);
        debug!("Using model: {}", MODEL);
        
        let request = CompletionRequest {
            prompt: Message::User {
                content: OneOrMany::one(UserContent::text(prompt.to_string())),
            },
            chat_history: vec![],
            preamble: None,
            tools: vec![],
            temperature: None,  // Remove temperature since some models don't support it
            additional_params: None,
            documents: vec![],
            max_tokens: None,  // Let the model decide max tokens
        };
        
        let completion = match self.openai_client
            .completion_model(MODEL)
            .completion(request)
            .await {
                Ok(c) => {
                    debug!("Got successful completion response");
                    c
                },
                Err(e) => {
                    error!("Completion request failed: {:?}", e);
                    return Err(anyhow::anyhow!("Failed to get completion: {}", e));
                }
            };

        // Extract the first text content from the response
        let result = completion.choice
            .into_iter()
            .find_map(|content| match content {
                rig::message::AssistantContent::Text(text) => {
                    debug!("Found text content: {}", text.text);
                    Some(text.text)
                },
                _other => {
                    debug!("Found non-text content type");
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("No text response from completion"))?;

        debug!("Extracted result: {}", result);
        Ok(result)
    }

    pub async fn filter_tokens(&self, page: i64, _limit: Option<i64>) -> Result<FilterResponse> {
        // Step 1: Get LLM to choose BirdEye filters
        info!("Getting LLM to choose BirdEye filters");
        let filters = self.get_birdeye_filters().await
            .context("Failed to get BirdEye filters")?;
        
        // Step 2: Get initial token list from Birdeye v3
        info!("Fetching token list from Birdeye v3 API");
        let token_list = self.birdeye
            .get_token_list_v3(page, filters.limit, Some(&filters.filters))
            .await
            .context("Failed to fetch token list")?;

        // Step 3: First LLM analysis of market data
        info!("Performing initial market analysis");
        let initial_analysis = self.analyze_market_data(&token_list.data.items).await
            .context("Failed to analyze market data")?;

        // Step 4: Get additional metadata for promising tokens
        info!("Fetching metadata for filtered tokens");
        let mut enriched_tokens = Vec::new();
        for token in &initial_analysis.filtered_tokens {
            if token.score >= 0.7 {
                match self.birdeye.get_token_metadata_v3(&token.address).await {
                    Ok(metadata) => enriched_tokens.push((token.clone(), metadata)),
                    Err(e) => {
                        error!("Failed to fetch metadata for token {}: {}", token.address, e);
                        continue;
                    }
                }
            }
        }

        // Step 5: Final LLM analysis with social and dev metrics
        info!("Performing final analysis with metadata");
        let final_analysis = self.analyze_metadata(&enriched_tokens).await
            .context("Failed to analyze metadata")?;

        // Step 6: Store results in MongoDB
        info!("Storing analysis results");
        self.store_analysis_results(&final_analysis).await
            .context("Failed to store analysis results")?;

        Ok(final_analysis)
    }

    async fn get_birdeye_filters(&self) -> Result<BirdeyeFilters> {
        debug!("Loading initial filter prompt from file");
        debug!("Initial filter prompt content: {}", INITIAL_FILTER_PROMPT);
        
        let prompt = format!(
            "Return BirdEye filter parameters as JSON.\n\n{}",
            INITIAL_FILTER_PROMPT
        );
        
        debug!("Constructed prompt: {}", prompt);
        
        let response = match self.get_completion(&prompt).await {
            Ok(resp) => {
                debug!("Got successful response from completion");
                resp
            },
            Err(e) => {
                error!("Failed to get completion response: {:?}", e);
                return Err(anyhow::anyhow!("Failed to get LLM response: {}", e));
            }
        };

        debug!("Attempting to parse response as JSON: {}", response);

        // Strip markdown code block markers if present
        let json_str = response
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();

        let filters = match serde_json::from_str(json_str) {
            Ok(f) => {
                debug!("Successfully parsed JSON response");
                f
            },
            Err(e) => {
                error!("Failed to parse JSON response: {}\nResponse was: {}", e, response);
                return Err(anyhow::anyhow!("Failed to parse BirdEye filters from LLM response: {}", e));
            }
        };
        
        info!("Generated BirdEye filters: {:?}", filters);
        Ok(filters)
    }

    async fn analyze_market_data(&self, tokens: &[TokenV3Response]) -> Result<FilterResponse> {
        let tokens_json = serde_json::to_string(tokens)
            .context("Failed to serialize tokens for market analysis")?;

        debug!("Analyzing market data for {} tokens", tokens.len());
        debug!("Market analysis input: {}", tokens_json);

        let prompt = format!(
            "As a Solana token analyst, analyze these tokens and provide results in JSON format.\n\n{}\n\nTokens to analyze: {}",
            MARKET_FILTER_PROMPT,
            tokens_json
        );
        
        let response = self.get_completion(&prompt).await?;

        debug!("Received market analysis response: {}", response);

        let analysis: FilterResponse = serde_json::from_str(&response)
            .context("Failed to parse market analysis from LLM response")?;

        info!("Completed market analysis with {} filtered tokens", analysis.filtered_tokens.len());
        Ok(analysis)
    }

    async fn analyze_metadata(&self, tokens: &[(TokenAnalysis, TokenV3Response)]) -> Result<FilterResponse> {
        let analysis_json = serde_json::to_string(tokens)
            .context("Failed to serialize tokens with analysis")?;

        debug!("Analyzing metadata for {} tokens", tokens.len());
        debug!("Metadata analysis input: {}", analysis_json);

        let prompt = format!(
            "As a Solana token analyst, analyze these tokens with their metadata and provide results in JSON format.\n\n{}\n\nTokens with metadata to analyze: {}",
            METADATA_FILTER_PROMPT,
            analysis_json
        );

        let response = self.get_completion(&prompt).await?;

        debug!("Received metadata analysis response: {}", response);

        let analysis: FilterResponse = serde_json::from_str(&response)
            .context("Failed to parse metadata analysis from LLM response")?;

        info!("Completed metadata analysis with {} final filtered tokens", analysis.filtered_tokens.len());
        Ok(analysis)
    }

    async fn store_analysis_results(&self, analysis: &FilterResponse) -> Result<()> {
        let collection = "token_analytics";
        let timestamp = bson::DateTime::now();

        debug!("Preparing to store analysis results for {} tokens", analysis.filtered_tokens.len());

        let documents: Vec<Document> = analysis
            .filtered_tokens
            .iter()
            .map(|token| {
                doc! {
                    "token_address": &token.address,
                    "token_symbol": &token.symbol,
                    "analysis_type": "filtered",
                    "timestamp": timestamp,
                    "scores": {
                        "overall": token.score,
                        "market": token.analysis.market_score,
                        "social": token.analysis.social_score,
                        "development": token.analysis.dev_score,
                        "risk": token.analysis.risk_score
                    },
                    "metrics": token.analysis.metrics.as_ref().map(|m| doc! {
                        "social": {
                            "twitter_quality": m.social_metrics.as_ref().map(|s| s.twitter_quality),
                            "community_engagement": m.social_metrics.as_ref().map(|s| s.community_engagement),
                            "sentiment": m.social_metrics.as_ref().map(|s| s.sentiment)
                        },
                        "development": {
                            "github_activity": m.dev_metrics.as_ref().map(|d| d.github_activity),
                            "wallet_patterns": m.dev_metrics.as_ref().map(|d| d.wallet_patterns),
                            "contract_quality": m.dev_metrics.as_ref().map(|d| d.contract_quality)
                        }
                    }),
                    "analysis": {
                        "strengths": &token.analysis.key_strengths,
                        "risks": &token.analysis.key_risks,
                        "recommendation": &token.analysis.final_recommendation
                    },
                    "market_context": {
                        "total_analyzed": analysis.summary.total_analyzed,
                        "total_passed": analysis.summary.total_passed,
                        "market_conditions": &analysis.summary.market_conditions,
                        "risk_assessment": &analysis.summary.risk_assessment
                    }
                }
            })
            .collect();

        debug!("Storing {} documents in MongoDB collection '{}'", documents.len(), collection);

        self.db_pool
            .insert_token_analytics_documents(collection, documents)
            .await
            .context("Failed to store token analysis results")?;

        info!("Successfully stored analysis results in MongoDB");
        Ok(())
    }
} 
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

const INITIAL_FILTER_PROMPT: &str = include_str!("../prompts/token_filter_initial.txt");
const MODEL: &str = openai::O1_MINI;  // Using O1_MINI until O3_MINI is available in RIG

const PIPELINE_STEP_1: &str = "PIPELINE STEP 1: BirdEye Filter Selection";
const PIPELINE_STEP_2: &str = "PIPELINE STEP 2: Token List Retrieval";
const PIPELINE_STEP_3: &str = "PIPELINE STEP 3: Market Analysis";
const PIPELINE_STEP_4: &str = "PIPELINE STEP 4: Metadata Analysis";
const PIPELINE_STEP_5: &str = "PIPELINE STEP 5: Final Filtering & Storage";

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
        debug!("Using model: {}", MODEL);
        
        let request = CompletionRequest {
            prompt: Message::User {
                content: OneOrMany::one(UserContent::text(prompt.to_string())),
            },
            chat_history: vec![],
            preamble: None,
            tools: vec![],
            temperature: None,
            additional_params: None,
            documents: vec![],
            max_tokens: None,
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

        let result = completion.choice
            .into_iter()
            .find_map(|content| match content {
                rig::message::AssistantContent::Text(text) => Some(text.text),
                _other => None
            })
            .ok_or_else(|| anyhow::anyhow!("No text response from completion"))?;

        Ok(result)
    }

    pub async fn filter_tokens(&self, page: i64, _limit: Option<i64>) -> Result<FilterResponse> {
        info!("\n{}\n{}", PIPELINE_STEP_1, "=".repeat(50));
        
        // Get BirdEye filters
        let filters = self.get_birdeye_filters().await?;
        info!("Selected filters: min_liquidity={}, min_market_cap={}, min_holder={}", 
            filters.filters.get("min_liquidity").unwrap_or(&serde_json::Value::Null),
            filters.filters.get("min_market_cap").unwrap_or(&serde_json::Value::Null),
            filters.filters.get("min_holder").unwrap_or(&serde_json::Value::Null)
        );

        info!("\n{}\n{}", PIPELINE_STEP_2, "=".repeat(50));
        
        // Get token list using v3 endpoint with correct parameters
        let tokens = self.birdeye
            .get_token_list_v3(
                page,
                filters.limit,
                Some(&filters.filters)
            )
            .await?;
            
        info!("Retrieved {} tokens from BirdEye API", tokens.data.items.len());

        info!("\n{}\n{}", PIPELINE_STEP_3, "=".repeat(50));
        
        // Analyze market data
        let market_analysis = self.analyze_market_data(&tokens.data.items).await?;
        
        // Get metadata for filtered tokens
        let token_pairs: Vec<(TokenAnalysis, TokenV3Response)> = market_analysis
            .filtered_tokens
            .into_iter()
            .filter_map(|analysis| {
                tokens.data.items
                    .iter()
                    .find(|t| t.address == analysis.address)
                    .map(|t| (analysis, t.clone()))
            })
            .collect();

        info!("\n{}\n{}", PIPELINE_STEP_4, "=".repeat(50));
        
        // Analyze metadata
        let metadata_analysis = self.analyze_metadata(&token_pairs).await?;

        info!("\n{}\n{}", PIPELINE_STEP_5, "=".repeat(50));
        
        // Store results
        self.store_analysis_results(&metadata_analysis).await?;
        info!("Analysis complete - {} tokens stored in recommendations", metadata_analysis.filtered_tokens.len());
        
        Ok(metadata_analysis)
    }

    async fn get_birdeye_filters(&self) -> Result<BirdeyeFilters> {
        let prompt = format!(
            "Return BirdEye filter parameters as JSON.\n\n{}",
            INITIAL_FILTER_PROMPT
        );
        
        let response = self.get_completion(&prompt).await?;
        
        // Clean the response by removing markdown code blocks
        let clean_response = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
            
        // Parse the raw filter parameters into a HashMap
        let filter_params: std::collections::HashMap<String, serde_json::Value> = 
            serde_json::from_str(clean_response)
                .context("Failed to parse filter parameters")?;
                
        // Construct BirdeyeFilters with mandatory parameters
        let filters = BirdeyeFilters {
            sort_by: "liquidity".to_string(),
            sort_type: "desc".to_string(),
            limit: 100,
            offset: Some(0),
            filters: filter_params,
        };
            
        Ok(filters)
    }

    async fn analyze_market_data(&self, tokens: &[TokenV3Response]) -> Result<FilterResponse> {
        let prompt = format!(
            r#"You are an expert Solana trader analyzing market data for potential investments.

Analyze these tokens based on market metrics and return a FilterResponse with:
1. filtered_tokens: Array of tokens that pass analysis
2. summary: Overall market analysis summary

For each token calculate:
- Liquidity Score (0.0-1.0): Based on liquidity depth
- Volume Score (0.0-1.0): Based on trading volume
- Momentum Score (0.0-1.0): Based on price action

Return response in this format:
{{
  "filtered_tokens": [
    {{
      "address": "token_address",
      "symbol": "TOKEN",
      "score": 0.75,
      "analysis": {{
        "market_score": 0.8,
        "social_score": 0.0,
        "dev_score": 0.0,
        "risk_score": 0.7,
        "metrics": null,
        "key_strengths": ["Good liquidity", "Active trading"],
        "key_risks": ["New token", "Limited history"],
        "final_recommendation": "Consider small position"
      }}
    }}
  ],
  "summary": {{
    "total_analyzed": 10,
    "total_passed": 3,
    "avg_market_score": 0.65,
    "avg_social_score": 0.0,
    "avg_dev_score": 0.0,
    "avg_risk_score": 0.7,
    "market_conditions": "Mixed with some promising candidates",
    "risk_assessment": "High risk due to limited history"
  }}
}}

Tokens to analyze: {}"#,
            serde_json::to_string_pretty(tokens)?
        );

        debug!("Sending market analysis prompt...");
        let response = self.get_completion(&prompt).await?;
        
        // Clean and parse response
        let clean_response = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
            
        debug!("Parsing market analysis response...");
        let analysis: FilterResponse = serde_json::from_str(clean_response)
            .context("Failed to parse market analysis response")?;

        info!("Market analysis complete - {} of {} tokens passed initial filtering", 
            analysis.summary.total_passed, 
            analysis.summary.total_analyzed
        );
        
        Ok(analysis)
    }

    async fn analyze_metadata(&self, tokens: &[(TokenAnalysis, TokenV3Response)]) -> Result<FilterResponse> {
        let prompt = format!(
            r#"You are an expert Solana trader analyzing token metadata and social signals.

Analyze these tokens and return a FilterResponse with:
1. filtered_tokens: Array of tokens that pass analysis
2. summary: Overall analysis summary

For each token analyze:
Social & Community Metrics:
- Twitter Quality (engagement, followers)
- Community Activity (telegram, discord)
- Overall Sentiment

Development Metrics:
- GitHub Activity
- Contract Quality
- Wallet Patterns

Risk Factors:
- Team Transparency
- Token Distribution
- Smart Contract Security
- Market Manipulation Risk

Return response in this format:
{{
  "filtered_tokens": [
    {{
      "address": "token_address",
      "symbol": "TOKEN",
      "score": 0.75,
      "analysis": {{
        "market_score": 0.8,
        "social_score": 0.4,
        "dev_score": 0.2,
        "risk_score": 0.7,
        "metrics": null,
        "key_strengths": ["Good liquidity", "Active community"],
        "key_risks": ["Limited development", "High manipulation risk"],
        "final_recommendation": "Consider small position"
      }}
    }}
  ],
  "summary": {{
    "total_analyzed": 4,
    "total_passed": 2,
    "avg_market_score": 0.65,
    "avg_social_score": 0.3,
    "avg_dev_score": 0.2,
    "avg_risk_score": 0.7,
    "market_conditions": "Mixed with some promising candidates",
    "risk_assessment": "High risk due to limited history"
  }}
}}

Return strictly JSON with no commentary.

Tokens to analyze: {}"#,
            serde_json::to_string_pretty(tokens)?
        );

        debug!("Sending metadata analysis prompt...");
        let response = self.get_completion(&prompt).await?;
        
        // Clean and parse response
        let clean_response = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
            
        debug!("Parsing metadata analysis response...");
        let analysis: FilterResponse = serde_json::from_str(clean_response)
            .context("Failed to parse metadata analysis response")?;

        info!("Metadata analysis complete - {} of {} tokens passed final filtering", 
            analysis.summary.total_passed, 
            analysis.summary.total_analyzed
        );
        
        Ok(analysis)
    }

    async fn store_analysis_results(&self, analysis: &FilterResponse) -> Result<()> {
        let collection = "token_recommendations";
        let timestamp = bson::DateTime::now();

        debug!("Preparing to store token recommendations for {} tokens", analysis.filtered_tokens.len());

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
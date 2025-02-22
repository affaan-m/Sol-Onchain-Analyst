use crate::birdeye::api::{BirdeyeApi, TokenV3ListResponse, TokenV3Response};
use crate::config::mongodb::{MongoDbPool, TokenAnalyticsDataExt};
use anyhow::{anyhow, Context, Result};
use mongodb::bson::{self, doc, Document};
use rig::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

const DEFAULT_CHUNK_SIZE: i64 = 20;
const INITIAL_FILTER_PROMPT_PATH: &str = "src/prompts/token_filter_initial.txt";
const MARKET_FILTER_PROMPT_PATH: &str = "src/prompts/token_filter_market.txt";
const METADATA_FILTER_PROMPT_PATH: &str = "src/prompts/token_filter_metadata.txt";

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
    llm: Arc<dyn CompletionModel>,
}

impl TokenFilterService {
    pub fn new(
        birdeye: Arc<dyn BirdeyeApi>,
        db_pool: Arc<MongoDbPool>,
        llm: Arc<dyn CompletionModel>,
    ) -> Self {
        Self {
            birdeye,
            db_pool,
            llm,
        }
    }

    pub async fn filter_tokens(&self, page: i64, limit: Option<i64>) -> Result<FilterResponse> {
        // Step 1: Get LLM to choose BirdEye filters
        info!("Getting LLM to choose BirdEye filters");
        let filters = self.get_birdeye_filters(limit).await
            .context("Failed to get BirdEye filters")?;
        
        // Step 2: Get initial token list from Birdeye v3
        info!("Fetching token list from Birdeye v3 API");
        let token_list = self.birdeye.get_token_list_v3(page, filters.limit).await
            .context("Failed to fetch token list")?;

        // Step 3: First LLM analysis of market data
        info!("Performing initial market analysis");
        let initial_analysis = self.analyze_market_data(&token_list).await
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

    async fn get_birdeye_filters(&self, limit: Option<i64>) -> Result<BirdeyeFilters> {
        let prompt = std::fs::read_to_string(INITIAL_FILTER_PROMPT_PATH)
            .context("Failed to read initial filter prompt")?;

        let response = self.llm.complete(&[
            Message::system(&prompt),
            Message::user("Choose the optimal filter parameters for finding promising early-stage Solana memecoins."),
        ]).await.context("Failed to get LLM response")?;

        let filters: BirdeyeFilters = serde_json::from_str(&response.content)
            .context("Failed to parse LLM filter response")?;

        Ok(BirdeyeFilters {
            limit: limit.unwrap_or(filters.limit),
            ..filters
        })
    }

    async fn analyze_market_data(&self, token_list: &TokenV3ListResponse) -> Result<FilterResponse> {
        let prompt = std::fs::read_to_string(MARKET_FILTER_PROMPT_PATH)
            .context("Failed to read market filter prompt")?;

        let tokens_json = serde_json::to_string(&token_list.data)
            .context("Failed to serialize token list")?;

        let response = self.llm.complete(&[
            Message::system(&prompt),
            Message::user(&format!("Analyze these tokens based on market metrics: {}", tokens_json)),
        ]).await.context("Failed to get LLM response")?;

        let filter_response: FilterResponse = serde_json::from_str(&response.content)
            .context("Failed to parse LLM response")?;

        Ok(filter_response)
    }

    async fn analyze_metadata(&self, tokens: &[(TokenAnalysis, TokenV3Response)]) -> Result<FilterResponse> {
        let prompt = std::fs::read_to_string(METADATA_FILTER_PROMPT_PATH)
            .context("Failed to read metadata filter prompt")?;

        let analysis_json = serde_json::to_string(tokens)
            .context("Failed to serialize tokens with analysis")?;

        let response = self.llm.complete(&[
            Message::system(&prompt),
            Message::user(&format!(
                "Perform deep analysis on these tokens with full market data, social metrics, and development activity: {}",
                analysis_json
            )),
        ]).await.context("Failed to get LLM response")?;

        let filter_response: FilterResponse = serde_json::from_str(&response.content)
            .context("Failed to parse LLM response")?;

        Ok(filter_response)
    }

    async fn store_analysis_results(&self, analysis: &FilterResponse) -> Result<()> {
        let collection_name = "token_analysis";
        let timestamp = bson::DateTime::now();

        let documents: Vec<Document> = analysis.filtered_tokens.iter().map(|token| {
            doc! {
                "token_address": &token.address,
                "token_symbol": &token.symbol,
                "score": token.score,
                "market_score": token.analysis.market_score,
                "social_score": token.analysis.social_score,
                "dev_score": token.analysis.dev_score,
                "risk_score": token.analysis.risk_score,
                "metrics": token.analysis.metrics.as_ref().map(|m| doc! {
                    "social_metrics": {
                        "twitter_quality": m.social_metrics.as_ref().map(|s| s.twitter_quality),
                        "community_engagement": m.social_metrics.as_ref().map(|s| s.community_engagement),
                        "sentiment": m.social_metrics.as_ref().map(|s| s.sentiment),
                    },
                    "dev_metrics": {
                        "github_activity": m.dev_metrics.as_ref().map(|d| d.github_activity),
                        "wallet_patterns": m.dev_metrics.as_ref().map(|d| d.wallet_patterns),
                        "contract_quality": m.dev_metrics.as_ref().map(|d| d.contract_quality),
                    }
                }),
                "key_strengths": &token.analysis.key_strengths,
                "key_risks": &token.analysis.key_risks,
                "recommendation": &token.analysis.final_recommendation,
                "timestamp": timestamp,
                "summary": {
                    "total_analyzed": analysis.summary.total_analyzed,
                    "total_passed": analysis.summary.total_passed,
                    "avg_market_score": analysis.summary.avg_market_score,
                    "avg_social_score": analysis.summary.avg_social_score,
                    "avg_dev_score": analysis.summary.avg_dev_score,
                    "avg_risk_score": analysis.summary.avg_risk_score,
                    "market_conditions": &analysis.summary.market_conditions,
                    "risk_assessment": &analysis.summary.risk_assessment,
                }
            }
        }).collect();

        self.db_pool.insert_token_analytics_documents(collection_name, documents).await
            .context("Failed to store analysis results in MongoDB")?;

        Ok(())
    }
} 
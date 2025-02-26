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
use serde_json::json;
use crate::services::wallet_tracker::{WalletTrackerService, TokenRecommendation, DecisionReasoning};
use chrono::Utc;

const INITIAL_FILTER_PROMPT: &str = include_str!("../prompts/token_filter_initial.txt");
const MODEL: &str = openai::O3_MINI;  // Using O3_MINI which is now available in RIG

const PIPELINE_STEP_1: &str = "PIPELINE STEP 1: BirdEye Filter Selection";
const PIPELINE_STEP_2: &str = "PIPELINE STEP 2: Token List Retrieval";
const PIPELINE_STEP_3: &str = "PIPELINE STEP 3: Market Analysis";
const PIPELINE_STEP_4: &str = "PIPELINE STEP 4: Metadata Analysis";
const PIPELINE_STEP_5: &str = "PIPELINE STEP 5: Final Filtering & Storage";
const PIPELINE_STEP_6: &str = "PIPELINE STEP 6: KOL Ownership Analysis";

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
    wallet_tracker: Option<WalletTrackerService>,
}

impl TokenFilterService {
    pub fn new(
        birdeye: Arc<dyn BirdeyeApi>,
        db_pool: Arc<MongoDbPool>,
        openai_api_key: &str,
    ) -> Self {
        let openai_client = OpenAIClient::new(openai_api_key);
        let wallet_tracker = Some(WalletTrackerService::new(db_pool.clone()));

        Self {
            birdeye,
            db_pool,
            openai_client,
            wallet_tracker,
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
        
        info!("\n{}\n{}", PIPELINE_STEP_2, "=".repeat(50));
        
        // Get token list using v3 endpoint with correct parameters
        let tokens = self.birdeye
            .get_token_list_v3(
                page,
                filters.limit,
                Some(&filters.filters)
            )
            .await?;
            
        // Handle edge case of empty tokens list
        if tokens.data.items.is_empty() {
            info!("No tokens returned from BirdEye API");
            return Ok(FilterResponse {
                filtered_tokens: Vec::new(),
                summary: FilterSummary {
                    total_analyzed: 0,
                    total_passed: 0,
                    avg_market_score: 0.0,
                    avg_social_score: 0.0,
                    avg_dev_score: 0.0,
                    avg_risk_score: 0.0,
                    market_conditions: "No tokens to analyze".to_string(),
                    risk_assessment: "N/A".to_string(),
                }
            });
        }
            
        info!("Retrieved {} tokens from BirdEye API", tokens.data.items.len());

        info!("\n{}\n{}", PIPELINE_STEP_3, "=".repeat(50));
        
        // Filter out tokens with null fields
        let valid_tokens: Vec<_> = tokens.data.items.into_iter()
            .filter(|token| {
                // Check for null or invalid values in essential fields
                let is_valid = 
                    token.address.is_some() && 
                    !token.address.as_ref().unwrap().is_empty() &&
                    token.symbol.is_some() && 
                    !token.symbol.as_ref().unwrap().is_empty() &&
                    token.liquidity.is_some();
                
                if !is_valid {
                    debug!("Filtering out token with missing essential data");
                }
                
                is_valid
            })
            .collect();
            
        info!("Filtered to {} valid tokens after removing entries with missing data", valid_tokens.len());
        
        // Analyze market data with valid tokens only
        let market_analysis = self.analyze_market_data(&valid_tokens).await?;
        
        // Get metadata for filtered tokens
        let token_pairs: Vec<(TokenAnalysis, TokenV3Response)> = market_analysis
            .filtered_tokens
            .into_iter()
            .filter_map(|analysis| {
                valid_tokens
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
        
        // Check KOL wallet ownership
        if let Some(wallet_tracker) = &self.wallet_tracker {
            info!("\n{}\n{}", PIPELINE_STEP_6, "=".repeat(50));
            
            let token_count = metadata_analysis.filtered_tokens.len();
            info!("Checking KOL ownership for {} filtered tokens", token_count);
            
            for (i, token) in metadata_analysis.filtered_tokens.iter().enumerate() {
                info!("Checking KOL ownership for token {}/{}: {} ({})", 
                     i+1, token_count, token.symbol, token.address);
                     
                match wallet_tracker.update_token_recommendation_with_kol_data(&token.address).await {
                    Ok(_) => info!("KOL ownership check complete for {}", token.symbol),
                    Err(e) => error!("Error checking KOL ownership for {}: {}", token.symbol, e),
                }
            }
            
            info!("KOL ownership analysis complete");
        }
        
        Ok(metadata_analysis)
    }

    /// Get BirdEye filter parameters using LLM analysis
    pub async fn get_birdeye_filters(&self) -> Result<BirdeyeFilters> {
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
            
        // Parse the filter parameters directly from LLM response
        let filter_params: std::collections::HashMap<String, serde_json::Value> = 
            serde_json::from_str(clean_response)
                .context("Failed to parse filter parameters")?;
        
        // Extract sorting and pagination parameters
        let sort_by = filter_params.get("sort_by")
            .and_then(|v| v.as_str())
            .unwrap_or("liquidity")
            .to_string();
            
        let sort_type = filter_params.get("sort_type")
            .and_then(|v| v.as_str())
            .unwrap_or("desc")
            .to_string();
            
        let limit = filter_params.get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(100);
            
        let offset = filter_params.get("offset")
            .and_then(|v| v.as_i64());
                
        // Create filters map excluding sort/pagination parameters
        let mut filters = std::collections::HashMap::new();
        for (key, value) in filter_params {
            if !["sort_by", "sort_type", "limit", "offset"].contains(&key.as_str()) {
                filters.insert(key, value);
            }
        }
        
        // Log chosen filters
        info!("LLM selected filters:");
        for (key, value) in &filters {
            info!("- {}: {}", key, value);
        }
            
        // Construct BirdeyeFilters with LLM chosen parameters
        let filters = BirdeyeFilters {
            sort_by,
            sort_type,
            limit,
            offset,
            filters,
        };
        
        Ok(filters)
    }

    async fn analyze_market_data(&self, tokens: &[TokenV3Response]) -> Result<FilterResponse> {
        let prompt = format!(
            r#"You are an expert Solana cryptocurrency analyst specializing in early-stage tokens. 
Your task is to analyze the market metrics of these tokens and identify those with promising market dynamics.

## Analysis Guidelines:

### Liquidity Analysis:
- Examine liquidity depth versus token age
- Calculate liquidity-to-market cap ratios (healthy range: 10%-40%)
- Identify unusual liquidity movements (sudden spikes or drops)
- Assess liquidity concentration across DEXs

### Volume Analysis:
- Analyze 24h/1h volume ratios
- Evaluate volume distribution patterns
- Detect potential wash trading (suspiciously uniform trade sizes/timing)
- Compare volume to similar market cap tokens

### Price Action Analysis:
- Identify momentum patterns (bullish/bearish divergences)
- Evaluate price stability during market fluctuations
- Detect potential manipulation patterns
- Assess price discovery phases

### Holder Distribution:
- Examine holder count growth rates
- Identify concerning wallet concentration
- Analyze new holder acquisition rate
- Detect suspicious wallet patterns

Your analysis should be comprehensive, specific to each token, and backed by concrete metrics.
For promising tokens, provide scores between 0.6-0.9 (reserve scores >0.9 for exceptional cases only).
For concerning tokens, provide scores between 0.0-0.5.

Return response in this precise format:
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
        "key_strengths": ["Detailed strength point 1", "Detailed strength point 2", "Detailed strength point 3"],
        "key_risks": ["Detailed risk point 1", "Detailed risk point 2"],
        "final_recommendation": "Comprehensive recommendation with specific entry/exit conditions"
      }}
    }}
  ],
  "summary": {{
    "total_analyzed": total_tokens_count,
    "total_passed": tokens_meeting_criteria_count,
    "avg_market_score": average_market_score,
    "avg_social_score": 0.0,
    "avg_dev_score": 0.0,
    "avg_risk_score": average_risk_score,
    "market_conditions": "Detailed market context assessment",
    "risk_assessment": "Comprehensive risk evaluation across analyzed tokens"
  }}
}}

Return strictly JSON with no commentary. Focus only on market metrics at this stage.

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
            r#"You are an expert Solana cryptocurrency analyst conducting comprehensive social and development analysis on early-stage tokens.
You've already performed market analysis on these tokens, and now need to evaluate their social signals, community engagement, 
development activity, and overall token quality.

## Analysis Guidelines:

### Social Signal Evaluation:
- Examine Twitter profile quality, follower authenticity, and engagement metrics
- Assess Discord/Telegram community activity (meaningful interactions vs. bot activity)
- Evaluate sentiment trends across social platforms
- Identify red flags (excessive bot activity, coordinated shilling, misleading claims)
- Detect artificial engagement patterns

### Development Analysis:
- Assess contract code quality and security practices
- Evaluate development team transparency and track record
- Identify concerning contract patterns (backdoors, unusual privileges)
- Analyze wallet behavior patterns for insider activity

### Risk Assessment:
- Evaluate token distribution patterns (whale concentration, team allocations)
- Identify potential regulatory concerns
- Assess market manipulation indicators
- Evaluate liquidity lock status and vesting schedules
- Flag potential security vulnerabilities

### Comprehensive Scoring:
- Assign social_score (0.0-1.0) based on community authenticity and engagement
- Assign dev_score (0.0-1.0) based on technical fundamentals and team quality
- Assign risk_score (0.0-1.0) - higher scores mean LOWER risk
- Update the overall score considering all factors
- Provide detailed strengths and risks for each token

Your analysis should integrate with the market data already present, creating a holistic view of each token's potential.
Remain objective and data-driven, flagging both positive and negative indicators.

For each token pair provided, I'm including:
1. The existing TokenAnalysis with market metrics already scored
2. The raw TokenV3Response with additional metadata

Return response using this exact format:
{{
  "filtered_tokens": [
    {{
      "address": "token_address",
      "symbol": "TOKEN",
      "score": updated_score_based_on_all_factors,
      "analysis": {{
        "market_score": existing_market_score,
        "social_score": detailed_social_score,
        "dev_score": detailed_dev_score,
        "risk_score": detailed_risk_score,
        "metrics": {{
          "social_metrics": {{
            "twitter_quality": score,
            "community_engagement": score,
            "sentiment": score
          }},
          "dev_metrics": {{
            "github_activity": score,
            "wallet_patterns": score,
            "contract_quality": score
          }}
        }},
        "key_strengths": ["Detailed strength point 1", "Detailed strength point 2", "Detailed strength point 3"],
        "key_risks": ["Detailed risk point 1", "Detailed risk point 2", "Detailed risk point 3"],
        "final_recommendation": "Comprehensive recommendation with risk assessment and position sizing guidance"
      }}
    }}
  ],
  "summary": {{
    "total_analyzed": total_tokens_analyzed,
    "total_passed": tokens_meeting_all_criteria,
    "avg_market_score": average_market_score,
    "avg_social_score": average_social_score,
    "avg_dev_score": average_dev_score,
    "avg_risk_score": average_risk_score,
    "market_conditions": "Detailed market context",
    "risk_assessment": "Comprehensive risk evaluation"
  }}
}}

Return strictly JSON with no commentary. Provide a complete analysis that could guide professional investment decisions.

Token pairs to analyze: {}"#,
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
        let db = self.db_pool.get_database()?;
        let collection = db.collection::<Document>("token_recommendations");
        
        for token in &analysis.filtered_tokens {
            // Generate detailed decision reasoning
            let decision_reasoning = self.generate_decision_reasoning(token).await?;
            
            // Convert to TokenRecommendation
            let recommendation = TokenRecommendation {
                id: None,
                token_address: token.address.clone(),
                symbol: token.symbol.clone(),
                name: String::new(), // Would need to get from token metadata
                decimals: 0,         // Would need to get from token metadata
                logo_uri: None,      // Would need to get from token metadata
                analysis_date: Utc::now(),
                overall_score: token.score,
                market_score: token.analysis.market_score,
                social_score: token.analysis.social_score,
                dev_score: token.analysis.dev_score,
                risk_score: token.analysis.risk_score,
                price: 0.0,           // Would need to get from token data
                liquidity: 0.0,       // Would need to get from token data
                market_cap: 0.0,      // Would need to get from token data
                volume_24h: 0.0,      // Would need to get from token data
                holders: 0,           // Would need to get from token data
                strengths: token.analysis.key_strengths.clone(),
                risks: token.analysis.key_risks.clone(),
                recommendation: token.analysis.final_recommendation.clone(),
                kol_ownership: None,  // Will be populated by wallet tracker
                decision_reasoning: Some(decision_reasoning),
                embedding: None,      // Would be generated if vector search is used
            };
            
            // Convert to BSON document
            let doc = mongodb::bson::to_document(&recommendation)?;
            
            // Insert or update
            let filter = doc! { "token_address": &token.address };
            let options = mongodb::options::UpdateOptions::builder()
                .upsert(true)
                .build();
                
            collection.update_one(filter, doc! { "$set": doc }, options)
                .await
                .context("Failed to update token recommendation")?;
                
            info!("Stored analysis for token: {}", token.symbol);
            
            // Check for KOL ownership if wallet tracker is available
            if let Some(wallet_tracker) = &self.wallet_tracker {
                wallet_tracker.update_token_recommendation_with_kol_data(&token.address).await?;
            }
        }
        
        Ok(())
    }

    async fn generate_decision_reasoning(&self, token: &TokenAnalysis) -> Result<DecisionReasoning> {
        // Get token recommendation prompt template
        let prompt_template = include_str!("../prompts/token_filter_reasoning.txt");
        
        // Create a prompt with token details
        let prompt = prompt_template
            .replace("{{symbol}}", &token.symbol)
            .replace("{{address}}", &token.address)
            .replace("{{overall_score}}", &format!("{:.2}", token.score))
            .replace("{{market_score}}", &format!("{:.2}", token.analysis.market_score))
            .replace("{{social_score}}", &format!("{:.2}", token.analysis.social_score))
            .replace("{{dev_score}}", &format!("{:.2}", token.analysis.dev_score))
            .replace("{{risk_score}}", &format!("{:.2}", token.analysis.risk_score))
            .replace("{{strengths}}", &token.analysis.key_strengths.join("\n"))
            .replace("{{risks}}", &token.analysis.key_risks.join("\n"));
        
        // Get completion from LLM
        debug!("Generating detailed decision reasoning for {}", token.symbol);
        let response = self.get_completion(&prompt).await?;
        
        // Clean and parse response
        let clean_response = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
            
        // Parse into DecisionReasoning
        let reasoning: DecisionReasoning = serde_json::from_str(clean_response)
            .context("Failed to parse decision reasoning")?;
        
        Ok(reasoning)
    }
} 
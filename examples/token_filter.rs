use cainam_core::{
    birdeye::api::BirdeyeClient,
    config::mongodb::{MongoConfig, MongoDbPool},
    services::token_filter::TokenFilterService,
};
use anyhow::Result;
use rig::prelude::*;
use std::sync::Arc;
use tracing::{info, warn, error};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use serde_json;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize detailed logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("cainam_core=debug".parse()?)
            .add_directive("token_filter=debug".parse()?))
        .with_span_events(FmtSpan::FULL)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting token filter pipeline...");

    // Initialize MongoDB
    info!("Initializing MongoDB connection...");
    let mongo_config = MongoConfig::from_env();
    let db_pool = match MongoDbPool::create_pool(mongo_config).await {
        Ok(pool) => {
            info!("Successfully connected to MongoDB");
            pool
        },
        Err(e) => {
            error!("Failed to connect to MongoDB: {}", e);
            return Err(e.into());
        }
    };

    // Initialize Birdeye client
    info!("Initializing Birdeye client...");
    let birdeye_api_key = match std::env::var("BIRDEYE_API_KEY") {
        Ok(key) => {
            info!("Successfully loaded Birdeye API key");
            key
        },
        Err(e) => {
            error!("Failed to load Birdeye API key: {}", e);
            return Err(e.into());
        }
    };
    let birdeye = Arc::new(BirdeyeClient::new(birdeye_api_key));

    // Initialize LLM (using OpenAI GPT-4)
    info!("Initializing OpenAI LLM...");
    let openai_api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) => {
            info!("Successfully loaded OpenAI API key");
            key
        },
        Err(e) => {
            error!("Failed to load OpenAI API key: {}", e);
            return Err(e.into());
        }
    };
    let llm = Arc::new(OpenAICompletionModel::new(
        "gpt-4-turbo-preview".to_string(),
        openai_api_key,
    ));

    // Create token filter service
    info!("Creating token filter service...");
    let token_filter = TokenFilterService::new(birdeye, db_pool, llm);

    // Filter tokens (page 1, default chunk size of 20)
    info!("Starting token filtering process...");
    match token_filter.filter_tokens(1, None).await {
        Ok(analysis) => {
            // Log the full analysis as JSON for debugging
            info!("Raw analysis result: {}", serde_json::to_string_pretty(&analysis)?);

            // Print summary results
            println!("\n=== Analysis Summary ===");
            println!("Total tokens analyzed: {}", analysis.summary.total_analyzed);
            println!("Total tokens passed: {}", analysis.summary.total_passed);
            println!("\nAverage Scores:");
            println!("  Market: {:.2}", analysis.summary.avg_market_score);
            println!("  Social: {:.2}", analysis.summary.avg_social_score);
            println!("  Development: {:.2}", analysis.summary.avg_dev_score);
            println!("  Risk: {:.2}", analysis.summary.avg_risk_score);
            println!("\nMarket Conditions: {}", analysis.summary.market_conditions);
            println!("Risk Assessment: {}", analysis.summary.risk_assessment);

            println!("\n=== Filtered Tokens ===");
            for token in analysis.filtered_tokens {
                println!("\nToken: {} ({})", token.symbol, token.address);
                println!("Overall score: {:.2}", token.score);
                println!("\nAnalysis Scores:");
                println!("  Market score: {:.2}", token.analysis.market_score);
                println!("  Social score: {:.2}", token.analysis.social_score);
                println!("  Dev score: {:.2}", token.analysis.dev_score);
                println!("  Risk score: {:.2}", token.analysis.risk_score);

                if let Some(metrics) = token.analysis.metrics {
                    if let Some(social) = metrics.social_metrics {
                        println!("\nSocial Metrics:");
                        println!("  Twitter Quality: {:.2}", social.twitter_quality);
                        println!("  Community Engagement: {:.2}", social.community_engagement);
                        println!("  Sentiment: {:.2}", social.sentiment);
                    }

                    if let Some(dev) = metrics.dev_metrics {
                        println!("\nDev Metrics:");
                        println!("  GitHub Activity: {:.2}", dev.github_activity);
                        println!("  Wallet Patterns: {:.2}", dev.wallet_patterns);
                        println!("  Contract Quality: {:.2}", dev.contract_quality);
                    }
                }

                println!("\nStrengths:");
                for strength in token.analysis.key_strengths {
                    println!("  - {}", strength);
                }
                println!("\nRisks:");
                for risk in token.analysis.key_risks {
                    println!("  - {}", risk);
                }
                println!("\nRecommendation: {}", token.analysis.final_recommendation);
                println!("\n{}", "-".repeat(80));
            }

            info!("Token filtering process completed successfully");
        },
        Err(e) => {
            error!("Failed to filter tokens: {}", e);
            return Err(e);
        }
    }

    Ok(())
} 
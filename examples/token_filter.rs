use cainam_core::{
    birdeye::api::BirdeyeClient,
    config::mongodb::{MongoConfig, MongoDbPool},
    services::token_filter::TokenFilterService,
};
use anyhow::Result;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use tracing::{info, warn, error, debug};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use serde_json;
use dotenvy::dotenv;

const SLEEP_DURATION: Duration = Duration::from_secs(300); // 5 minutes between runs

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize detailed logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("cainam_core=debug".parse()?)
            .add_directive("token_filter=debug".parse()?))
        .with_span_events(FmtSpan::FULL)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .init();

    info!("Starting token filter pipeline in continuous mode...");

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

    // Initialize Birdeye client with validation
    info!("Initializing Birdeye client...");
    let birdeye_api_key = match std::env::var("BIRDEYE_API_KEY") {
        Ok(key) if !key.trim().is_empty() => {
            info!("Successfully loaded Birdeye API key");
            key
        },
        Ok(_) => {
            error!("Birdeye API key is empty");
            return Err(anyhow::anyhow!("Empty Birdeye API key"));
        },
        Err(e) => {
            error!("Failed to load Birdeye API key: {}", e);
            return Err(e.into());
        }
    };
    let birdeye = Arc::new(BirdeyeClient::new(birdeye_api_key));

    // Initialize OpenAI API key with validation
    info!("Loading OpenAI API key...");
    let openai_api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) if !key.trim().is_empty() => {
            info!("Successfully loaded OpenAI API key");
            key
        },
        Ok(_) => {
            error!("OpenAI API key is empty");
            return Err(anyhow::anyhow!("Empty OpenAI API key"));
        },
        Err(e) => {
            error!("Failed to load OpenAI API key: {}", e);
            return Err(e.into());
        }
    };

    // Create token filter service
    info!("Creating token filter service...");
    let filter_service = TokenFilterService::new(
        birdeye,
        db_pool,
        &openai_api_key,
    );

    info!("Entering continuous processing loop...");
    let mut page = 1;
    loop {
        info!("Starting token filtering process for page {}...", page);
        match filter_service.filter_tokens(page, None).await {
            Ok(analysis) => {
                // Log detailed input/output at each step
                debug!(
                    "Raw analysis result for page {}: {}",
                    page,
                    serde_json::to_string_pretty(&analysis)?
                );

                // Print summary results
                println!("\n=== Analysis Summary for Page {} ===", page);
                println!("Total tokens analyzed: {}", analysis.summary.total_analyzed);
                println!("Total tokens passed: {}", analysis.summary.total_passed);
                println!("\nAverage Scores:");
                println!("  Market: {:.2}", analysis.summary.avg_market_score);
                println!("  Social: {:.2}", analysis.summary.avg_social_score);
                println!("  Development: {:.2}", analysis.summary.avg_dev_score);
                println!("  Risk: {:.2}", analysis.summary.avg_risk_score);

                // Increment page if we got results, otherwise reset to 1
                if !analysis.filtered_tokens.is_empty() {
                    page += 1;
                } else {
                    info!("No more tokens found, resetting to page 1");
                    page = 1;
                }

                info!("Token filtering process completed successfully for page {}", page);
                info!("Sleeping for {} seconds before next run...", SLEEP_DURATION.as_secs());
                sleep(SLEEP_DURATION).await;
            },
            Err(e) => {
                error!("Failed to filter tokens on page {}: {}", page, e);
                // On error, wait a shorter time and try again
                warn!("Retrying in 60 seconds...");
                sleep(Duration::from_secs(60)).await;
            }
        }
    }
}
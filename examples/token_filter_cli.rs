use anyhow::Result;
use cainam_core::{
    birdeye::api::{BirdeyeApi, BirdeyeClient},
    cli,
    config::{
        mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig},
        AgentConfig,
    },
    services::token_filter::TokenFilterService,
};
use clap::{Parser, Subcommand};
use colored::*;
use dotenvy::dotenv;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use mongodb::bson::doc;

const SLEEP_DURATION: Duration = Duration::from_secs(180); // 3 minutes

#[derive(Parser)]
#[command(author, version, about = "Token Filter Pipeline CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the complete token filter pipeline
    Filter {
        /// Page number to start from
        #[arg(default_value = "1")]
        page: i64,

        /// Number of tokens to analyze per page
        #[arg(default_value = "100")]
        limit: i64,

        /// Run continuously
        #[arg(long, default_value = "false")]
        continuous: bool,
    },

    /// Show the last analysis results
    LastResults,

    /// Monitor specific tokens
    Monitor {
        /// Token addresses (comma-separated)
        #[arg(value_delimiter = ',')]
        addresses: Vec<String>,

        /// Monitoring interval in seconds
        #[arg(default_value = "300")]
        interval: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .init();

    // Load environment variables
    dotenv().ok();

    // Clear screen and show welcome message
    cli::clear_screen();
    cli::print_section_header("Cainam Token Filter Pipeline");
    println!("Initializing pipeline components...\n");

    // Initialize services with progress indicators
    let progress = cli::CliProgress::new("Loading configuration");
    let config = AgentConfig::new_from_env()?;
    progress.finish_with_message("✓ Configuration loaded");

    let progress = cli::CliProgress::new("Connecting to MongoDB");
    let mongodb_uri = dotenvy::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let mongodb_database = dotenvy::var("MONGODB_DATABASE").expect("MONGODB_DATABASE must be set");
    let mongo_config = MongoConfig {
        uri: mongodb_uri,
        database: mongodb_database,
        app_name: Some("token-filter-cli".to_string()),
        pool_config: MongoPoolConfig::default(),
    };
    let db_pool = MongoDbPool::create_pool(mongo_config).await?;
    progress.finish_with_message("✓ MongoDB connected");

    let progress = cli::CliProgress::new("Initializing BirdEye API");
    let birdeye: Arc<dyn BirdeyeApi> = Arc::new(BirdeyeClient::new(config.birdeye_api_key.clone()));
    progress.finish_with_message("✓ BirdEye API initialized");

    let progress = cli::CliProgress::new("Setting up filter service");
    let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let filter_service = TokenFilterService::new(
        birdeye.clone(),
        db_pool,
        &openai_api_key,
    );
    progress.finish_with_message("✓ Filter service ready");

    // Parse command line arguments
    let cli = Cli::parse();

    match cli.command {
        Commands::Filter { page, limit, continuous } => {
            let mut current_page = page;

            loop {
                cli::print_section_header(&format!("Processing Page {}", current_page));

                // Stage 1: BirdEye Filter Selection
                let progress = cli::CliProgress::new("Stage 1: Selecting BirdEye filters");
                let filters = filter_service.get_birdeye_filters().await?;
                progress.finish_with_message("✓ Filter parameters selected");
                
                println!("\nSelected Filters:");
                println!("Sort by: {}", filters.sort_by);
                println!("Sort type: {}", filters.sort_type);
                println!("Min liquidity: {}", 
                    filters.filters.get("min_liquidity")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0)
                );
                println!("Min market cap: {}", 
                    filters.filters.get("min_market_cap")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0)
                );

                // Stage 2: Token List Retrieval
                let progress = cli::CliProgress::new("Stage 2: Retrieving token list");
                let analysis = filter_service.filter_tokens(current_page, Some(limit)).await?;
                progress.finish_with_message(&format!("✓ Retrieved {} tokens", analysis.summary.total_analyzed));

                // Stage 3 & 4: Market & Metadata Analysis
                cli::print_analysis_summary(
                    analysis.summary.total_analyzed,
                    analysis.summary.total_passed,
                    analysis.summary.avg_market_score,
                    analysis.summary.avg_social_score,
                    analysis.summary.avg_dev_score,
                    analysis.summary.avg_risk_score,
                );

                // Stage 5: Final Results
                cli::print_section_header("Filtered Tokens");
                for token in &analysis.filtered_tokens {
                    println!("\n{}", &token.symbol.bold());
                    println!("Address: {}", &token.address.dimmed());
                    println!("Score: {}", format!("{:.2}", token.score).cyan());
                    
                    // Get full token recommendation with reasoning from DB
                    match db_pool.get_database().unwrap().collection::<Document>("token_recommendations")
                        .find_one(doc! { "token_address": &token.address }, None)
                        .await
                    {
                        Ok(Some(doc)) => {
                            // Display KOL ownership if available
                            if let Ok(kol_ownership) = doc.get_array("kol_ownership") {
                                if !kol_ownership.is_empty() {
                                    println!("\n{}", "KOL Ownership:".yellow().bold());
                                    for kol in kol_ownership {
                                        if let Ok(kol_doc) = kol.as_document() {
                                            println!("  • {} ({}) - Position: {}", 
                                                kol_doc.get_str("name").unwrap_or("Unknown"),
                                                kol_doc.get_str("wallet_address").unwrap_or("Unknown address"),
                                                kol_doc.get_f64("position_size").unwrap_or(0.0)
                                            );
                                        }
                                    }
                                }
                            }
                            
                            // Display decision reasoning if available
                            if let Ok(reasoning) = doc.get_document("decision_reasoning") {
                                println!("\n{}", "Detailed Analysis:".green().bold());
                                
                                if let Ok(market) = reasoning.get_str("market_analysis") {
                                    println!("\n{}", "Market Analysis:".cyan());
                                    println!("{}", market);
                                }
                                
                                if let Ok(sentiment) = reasoning.get_str("sentiment_analysis") {
                                    println!("\n{}", "Sentiment Analysis:".cyan());
                                    println!("{}", sentiment);
                                }
                                
                                if let Ok(risk) = reasoning.get_str("risk_assessment") {
                                    println!("\n{}", "Risk Assessment:".red());
                                    println!("{}", risk);
                                }
                                
                                if let Ok(final_rec) = reasoning.get_str("final_reasoning") {
                                    println!("\n{}", "Final Recommendation:".yellow());
                                    println!("{}", final_rec);
                                }
                            }
                        },
                        _ => {
                            // Fall back to basic display if detailed record not found
                            println!("\nStrengths:");
                            for strength in &token.analysis.key_strengths {
                                println!("  • {}", strength.green());
                            }
                            
                            println!("\nRisks:");
                            for risk in &token.analysis.key_risks {
                                println!("  • {}", risk.red());
                            }
                            
                            println!("\nRecommendation: {}", &token.analysis.final_recommendation.yellow());
                        }
                    }
                    
                    println!("\n{}", "=".repeat(50));
                }

                if !continuous {
                    break;
                }

                current_page += 1;
                println!("\nWaiting {} seconds before next analysis...", SLEEP_DURATION.as_secs());
                sleep(SLEEP_DURATION).await;
                cli::clear_screen();
            }
        }

        Commands::LastResults => {
            cli::print_section_header("Last Analysis Results");
            let progress = cli::CliProgress::new("Fetching latest results");
            
            // TODO: Implement fetching last results from MongoDB
            progress.finish_with_message("✓ Results retrieved");
        }

        Commands::Monitor { addresses, interval } => {
            cli::print_section_header("Token Monitor");
            println!(
                "Monitoring {} tokens every {} seconds\nPress Ctrl+C to stop.",
                addresses.len(),
                interval
            );

            loop {
                for address in &addresses {
                    let progress = cli::CliProgress::new(&format!("Analyzing {}", address));
                    match birdeye.get_token_overview(address).await {
                        Ok(token) => {
                            progress.finish_with_message("✓ Analysis complete");
                            cli::print_token_info(
                                &token.name,
                                &token.symbol,
                                token.price,
                                token.market_cap,
                                token.v24h_usd,
                                token.price_change_24h_percent,
                            );
                        }
                        Err(e) => {
                            progress.finish_with_message(&format!("✗ Failed: {}", e));
                        }
                    }
                }

                sleep(Duration::from_secs(interval)).await;
                cli::clear_screen();
            }
        }
    }

    Ok(())
} 
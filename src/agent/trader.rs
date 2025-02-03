use std::sync::Arc;
use crate::{
    config::{AgentConfig, MarketConfig},
    error::{AgentError, AgentResult},
    MarketSignal, SignalType,
    services::token_analytics::TokenAnalyticsService,
    trading::trading_engine::TradingEngine,
    birdeye::BirdeyeClient,
    twitter::{TwitterClient, TwitterApi},
    SolanaAgentKit,
    utils::f64_to_decimal,
};
use rig::{
    agent::Agent,
    providers::openai::{Client as OpenAIClient, CompletionModel},
};
use tokio::time::{sleep, Duration};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn, error};
use sqlx::PgPool;
use bigdecimal::BigDecimal;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: u64 = 1000; // 1 second

pub struct TradingAgent {
    agent: Agent<CompletionModel>,
    trading_engine: TradingEngine,
    twitter_client: Box<dyn TwitterApi>,
    analytics_service: Arc<TokenAnalyticsService>,
    config: AgentConfig,
    running: Arc<AtomicBool>,
}

impl TradingAgent {
    pub async fn new(config: AgentConfig, db: Arc<PgPool>, solana_agent: SolanaAgentKit) -> AgentResult<Self> {
        info!("Initializing TradingAgent...");
        
        // Initialize OpenAI client
        let openai_client = OpenAIClient::new(&config.openai_api_key);
        
        info!("Creating GPT-4 agent...");
        let agent = openai_client
            .agent(crate::config::get_openai_model())
            .preamble(include_str!("../prompts/system.txt"))
            .build();

        // Initialize components
        let trading_engine = TradingEngine::new(
            config.trade_min_confidence,
            config.trade_max_amount,
            solana_agent,
        );
        
        info!("Initializing Twitter client...");
        let mut twitter_client = TwitterClient::new(
            config.twitter_email.clone(),
            config.twitter_username.clone(),
            config.twitter_password.clone(),
        );

        // Retry Twitter login with exponential backoff
        let mut retry_count = 0;
        loop {
            match twitter_client.login().await {
                Ok(_) => {
                    info!("Successfully logged in to Twitter");
                    break;
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= MAX_RETRIES {
                        error!("Failed to login to Twitter after {} attempts", MAX_RETRIES);
                        return Err(AgentError::TwitterApi(format!("Login failed: {}", e)));
                    }
                    warn!("Failed to login to Twitter (attempt {}), retrying...", retry_count);
                    sleep(Duration::from_millis(RETRY_DELAY * 2u64.pow(retry_count))).await;
                }
            }
        }

        info!("Initializing Birdeye clients...");
        let birdeye = Arc::new(BirdeyeClient::new(config.birdeye_api_key.clone()));
        let birdeye_extended = Arc::new(cainam_birdeye::BirdeyeClient::new(config.birdeye_api_key.clone()));
        
        // Initialize market config
        let market_config = MarketConfig::new_from_env()?;

        // Initialize analytics service
        let analytics_service = Arc::new(TokenAnalyticsService::new(
            db,
            birdeye,
            birdeye_extended,
            Some(market_config),
        ));

        Ok(Self {
            agent,
            trading_engine,
            twitter_client: Box::new(twitter_client),
            analytics_service,
            config,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub async fn analyze_market(&self, symbol: &str, address: &str) -> AgentResult<Option<MarketSignal>> {
        info!("Starting market analysis for {}", symbol);
        
        // Fetch and store token analytics
        let analytics = self.analytics_service
            .fetch_and_store_token_info(symbol, address)
            .await
            .map_err(|e| AgentError::MarketAnalysis(format!("Failed to fetch token info: {}", e)))?;
        
        info!("Market Analysis for {}:", symbol);
        info!("Current Price: ${:.4}", analytics.price);
        if let Some(ref volume) = analytics.volume_24h {
            info!("24h Volume: ${:.2}", volume);
        }
        
        // Generate market signals
        let signal = self.analytics_service
            .generate_market_signals(&analytics)
            .await
            .map_err(|e| AgentError::MarketAnalysis(format!("Failed to generate signals: {}", e)))?;
        
        if let Some(signal) = &signal {
            info!(
                "Market signal generated: {:?} (confidence: {:.2})",
                signal.signal_type,
                signal.confidence
            );
        }
        
        Ok(signal)
    }

    pub async fn process_signal(&self, signal: &MarketSignal) -> AgentResult<Option<String>> {
        let zero = BigDecimal::from(0);
        let action = match signal.signal_type {
            SignalType::PriceSpike if signal.price > zero => "BUY",
            SignalType::StrongBuy => "BUY",
            SignalType::Buy => "BUY",
            SignalType::VolumeSurge if signal.volume_change > zero => "BUY",
            SignalType::PriceDrop => "SELL",
            SignalType::StrongSell => "SELL",
            SignalType::Sell => "SELL",
            SignalType::Hold => "HOLD",
            _ => return Ok(None),
        };

        // Convert f64 config values to BigDecimal
        let threshold = f64_to_decimal(self.config.trade_min_confidence);
        let max_amount = f64_to_decimal(self.config.trade_max_amount);

        if signal.confidence >= threshold {
            let amount = (max_amount.clone() * signal.confidence.clone()).min(max_amount.clone());

            match action {
                "BUY" | "SELL" => {
                    info!("Executing {} trade for {} with amount {}", action, signal.asset_address, amount);
                    self.trading_engine.execute_trade(signal)
                        .await
                        .map_err(|e| AgentError::Trading(format!("Trade execution failed: {}", e)))?;
                }
                _ => {}
            }
        }

        Ok(Some(action.to_string()))
    }

    pub async fn execute_trade(&self, _symbol: &str, signal: &MarketSignal) -> AgentResult<String> {
        self.trading_engine.execute_trade(signal)
            .await
            .map_err(|e| AgentError::Trading(format!("Trade execution failed: {}", e)))
    }

    pub async fn post_trade_update(
        &self,
        _symbol: &str,
        _action: &str,
        _amount: f64,
        _signal_type: &SignalType
    ) -> AgentResult<()> {
        // TODO: Implement post-trade updates
        // - Update portfolio state
        // - Log trade details
        // - Send notifications
        Ok(())
    }

    pub async fn run(&self) -> AgentResult<()> {
        info!("Starting trading agent...");
        self.running.store(true, Ordering::SeqCst);

        let tokens = [
            ("SOL", "So11111111111111111111111111111111111111112"),
            ("BONK", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
        ];

        while self.running.load(Ordering::SeqCst) {
            for (symbol, address) in tokens.iter() {
                match self.analyze_market(symbol, address).await {
                    Ok(Some(signal)) => {
                        let min_confidence = f64_to_decimal(self.config.trade_min_confidence);
                        if signal.confidence >= min_confidence {
                            if let Err(e) = self.process_signal(&signal).await {
                                error!("Error processing signal: {}", e);
                            }
                        } else {
                            info!("Signal confidence too low for trading");
                        }
                    }
                    Ok(None) => {
                        info!("No trading signals generated");
                    }
                    Err(e) => {
                        error!("Market analysis failed for {}: {}", symbol, e);
                    }
                }
            }

            info!("Waiting for next analysis interval ({:?})...", self.config.analysis_interval);
            sleep(self.config.analysis_interval).await;
            info!("Starting next analysis cycle");
        }

        info!("Trading agent stopped");
        Ok(())
    }

    pub fn stop(&self) {
        info!("Stopping trading agent...");
        self.running.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::twitter::MockTwitterApi;
    use crate::birdeye::MockBirdeyeApi;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_db() -> Arc<PgPool> {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for tests");
            
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create database pool");
            
        Arc::new(pool)
    }

    async fn setup_mocks() -> (Box<MockTwitterApi>, Box<MockBirdeyeApi>) {
        let mut twitter_mock = Box::new(MockTwitterApi::new());
        twitter_mock.expect_login()
            .times(1)
            .returning(|| Box::pin(async { Ok(()) }));

        let mut birdeye_mock = Box::new(MockBirdeyeApi::new());
        birdeye_mock.expect_get_token_info()
            .returning(|_| {
                Box::pin(async {
                    Ok(crate::birdeye::TokenInfo {
                        price: 100.0,
                        volume24h: 1000000.0,
                        price_change_24h: 5.0,
                        liquidity: 500000.0,
                        trade24h: 1000,
                    })
                })
            });

        (twitter_mock, birdeye_mock)
    }

    #[tokio::test]
    async fn test_market_analysis() -> AgentResult<()> {
        let db = setup_test_db().await;
        let solana_agent = SolanaAgentKit::new_from_env()?;
        
        let config = AgentConfig::new_from_env()?;
        let agent = TradingAgent::new(config, db, solana_agent).await?;
        
        let signal = agent.analyze_market(
            "SOL",
            "So11111111111111111111111111111111111111112"
        ).await?;
        
        assert!(signal.is_some());
        Ok(())
    }
}
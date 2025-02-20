// use crate::models::trade::Trade;
use crate::{
    config::AgentConfig,
    error::{AgentError, AgentResult},
    models::market_signal::{MarketSignal, SignalType},
    services::TokenAnalyticsService,
    utils::f64_to_decimal,
    trading::{SolanaAgentKit, trading_engine::TradingEngine},
    config::mongodb::MongoDbPool,
};
use bigdecimal::BigDecimal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::sleep;
use tracing::{error, info, warn};
use bson;

pub struct TradingAgent {
    analytics_service: Arc<TokenAnalyticsService>,
    config: AgentConfig,
    running: Arc<AtomicBool>,
    engine: TradingEngine,
    db_pool: Arc<MongoDbPool>,
}

impl TradingAgent {
    pub async fn new(
        config: AgentConfig,
        analytics_service: Arc<TokenAnalyticsService>,
        db_pool: Arc<MongoDbPool>,
        solana_agent: SolanaAgentKit,
    ) -> AgentResult<Self> {
        info!("Initializing TradingAgent...");

        let engine = TradingEngine::new(
            config.trade_min_confidence,
            config.trade_max_amount,
            solana_agent,
        );

        Ok(Self {
            analytics_service,
            config,
            running: Arc::new(AtomicBool::new(false)),
            engine,
            db_pool,
        })
    }

    pub async fn analyze_market(
        &self,
        symbol: &str,
        address: &str,
    ) -> AgentResult<Option<MarketSignal>> {
        info!("Starting market analysis for {}", symbol);

        // Fetch and store token analytics
        let analytics = self
            .analytics_service
            .fetch_and_store_token_info(symbol, address)
            .await
            .map_err(|e| {
                AgentError::MarketAnalysis(format!("Failed to fetch token info: {}", e))
            })?;

        info!("Market Analysis for {}:", symbol);
        info!("Current Price: ${:.4}", analytics.price);
        if let Some(ref volume) = analytics.volume_24h {
            info!("24h Volume: ${:.2}", volume);
        }

        // Generate market signals
        let signal = self
            .analytics_service
            .generate_market_signals(&analytics)
            .await
            .map_err(|e| {
                AgentError::MarketAnalysis(format!("Failed to generate signals: {}", e))
            })?;

        if let Some(signal) = &signal {
            info!(
                "Market signal generated: {:?} (confidence: {:.2})",
                signal.signal_type, signal.confidence
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

        if signal.confidence >= threshold {
            info!(
                "Signal meets confidence threshold for {}: {} (confidence: {:.2})",
                signal.asset_address, action, signal.confidence
            );
        }

        Ok(Some(action.to_string()))
    }

    pub async fn execute_trade(&self, symbol: &str, signal: &MarketSignal) -> AgentResult<String> {
        info!("Executing trade for {}", symbol);
        self.engine.execute_trade(signal)
            .await
            .map_err(|e| AgentError::Trading(format!("Trade execution failed: {}", e)))
    }

    pub async fn post_trade_update(
        &self,
        symbol: &str,
        action: &str,
        amount: f64,
        signal_type: &SignalType,
    ) -> AgentResult<()> {
        info!(
            "Posting trade update - Symbol: {}, Action: {}, Amount: {}, Type: {:?}",
            symbol, action, amount, signal_type
        );

        // Create trade update document
        let trade_update = bson::doc! {
            "symbol": symbol,
            "action": action,
            "amount": amount,
            "signal_type": format!("{:?}", signal_type),
            "timestamp": bson::DateTime::now(),
        };

        // Insert into trades collection
        self.db_pool
            .database("cainam")  // Use a constant or config value for database name
            .collection("trades")
            .insert_one(trade_update)
            .await
            .map_err(|e| AgentError::Database(e.into()))?;

        info!("Trade update posted successfully");
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

            info!(
                "Waiting for next analysis interval ({:?})...",
                self.config.analysis_interval
            );
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::birdeye::MockBirdeyeApi;
//     use crate::twitter::MockTwitterApi;

//     async fn setup_test_db() -> Arc<MongoDbPool> {
//         MongoDbPool::new_from_uri("mongodb://localhost:32770", "cainam_test")
//             .await
//             .expect("Failed to create test database pool")
//             .into()
//     }

//     async fn setup_mocks() -> (Box<MockTwitterApi>, Box<MockBirdeyeApi>) {
//         let mut twitter_mock = Box::new(MockTwitterApi::new());
//         twitter_mock
//             .expect_login()
//             .times(1)
//             .returning(|| Box::pin(async { Ok(()) }));

//         let mut birdeye_mock = Box::new(MockBirdeyeApi::new());
//         birdeye_mock.expect_get_token_info().returning(|_| {
//             Box::pin(async {
//                 Ok(crate::birdeye::TokenInfo {
//                     price: 100.0,
//                     volume_24h: 1000000.0,
//                     price_change_24h: 5.0,
//                     liquidity: 500000.0,
//                     trade_24h: 1000,
//                 })
//             })
//         });

//         (twitter_mock, birdeye_mock)
//     }

//     #[tokio::test]
//     async fn test_market_analysis() -> AgentResult<()> {
//         let db = setup_test_db().await;
//         let solana_agent = SolanaAgentKit::new_from_env()?;

//         let config = AgentConfig::new_from_env()?;
//         let agent = TradingAgent::new(config, db, solana_agent).await?;

//         let signal = agent
//             .analyze_market("SOL", "So11111111111111111111111111111111111111112")
//             .await?;

//         assert!(signal.is_some());
//         Ok(())
//     }
// }

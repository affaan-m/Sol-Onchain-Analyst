use crate::personality::StoicPersonality;
use crate::market_data::{DataProvider, MarketTrend};
use crate::twitter::TwitterClient;
use crate::strategy::{TradeAction, TradeRecommendation, TradingStrategy};
use crate::dex::jupiter::JupiterDex;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};
use rig::completion::CompletionModel;
use solana_sdk::signature::Keypair;
use uuid::Uuid;
use mongodb::{MongoConfig, MongoDbPool, MongoPoolConfig};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenState {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub price_usd: f64,
    pub price_sol: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub price_change_24h: f64,
    pub volume_change_24h: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct SyncCollection {
    pool: Arc<MongoDbPool>,
    database: String,
}

impl SyncCollection {
    pub fn new(pool: Arc<MongoDbPool>, database: String) -> Self {
        Self { pool, database }
    }
    
    pub async fn save_token_state(&self, state: &TokenState) -> Result<()> {
        let collection = self.pool
            .database(&self.database)
            .collection("token_states");
            
        collection.insert_one(state, None).await?;
        Ok(())
    }
    
    pub async fn get_token_state(&self, token_address: &str) -> Result<Option<TokenState>> {
        let collection = self.pool
            .database(&self.database)
            .collection("token_states");
            
        let filter = doc! {
            "address": token_address
        };
        
        let options = rig_mongodb::options::FindOneOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .build();
            
        collection.find_one(filter, options)
            .await
            .map_err(anyhow::Error::from)
    }
    
    pub async fn get_token_history(
        &self,
        token_address: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<TokenState>> {
        let collection = self.pool
            .database(&self.database)
            .collection("token_states");
            
        let filter = doc! {
            "address": token_address,
            "timestamp": {
                "$gte": start_time,
                "$lte": end_time
            }
        };
        
        let options = rig_mongodb::options::FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .build();
            
        let cursor = collection.find(filter, options).await?;
        cursor.try_collect().await.map_err(anyhow::Error::from)
    }
    
    pub async fn cleanup_old_data(&self, retention_days: i64) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days);
        let collection = self.pool
            .database(&self.database)
            .collection::<TokenState>("token_states");
            
        let filter = doc! {
            "timestamp": { "$lt": cutoff }
        };
        
        let result = collection.delete_many(filter, None).await?;
        debug!("Cleaned up {} old token state records", result.deleted_count);
        Ok(result.deleted_count)
    }
}

pub struct DataSyncService<M: CompletionModel> {
    db: Arc<SyncCollection>,
    data_provider: Box<dyn DataProvider>,
    twitter: TwitterClient,
    trading_strategy: TradingStrategy<M>,
    dex: JupiterDex,
    personality: StoicPersonality,
    wallet: Keypair,
    sync_interval: u64,
}

impl<M: CompletionModel> DataSyncService<M> {
    pub fn new(
        db: SyncCollection,
        data_provider: Box<dyn DataProvider>,
        twitter: TwitterClient,
        trading_strategy: TradingStrategy<M>,
        dex: JupiterDex,
        wallet: Keypair,
        sync_interval: u64,
    ) -> Self {
        Self {
            db: Arc::new(db),
            data_provider,
            twitter,
            trading_strategy,
            dex,
            personality: StoicPersonality::new(),
            wallet,
            sync_interval,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting data sync service");
        loop {
            if let Err(e) = self.sync_market_data().await {
                tracing::error!("Error syncing market data: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(self.sync_interval)).await;
        }
    }

    pub async fn sync_market_data(&self) -> Result<()> {
        info!("Starting market data sync cycle");
        
        // Fetch trending tokens
        info!("Fetching trending tokens from BirdEye");
        let trends = self.data_provider.get_token_trending(20).await?;
        info!("Found {} trending tokens", trends.len());

        // Insert token states and analyze trading opportunities
        for trend in trends {
            info!(
                "Processing token {} ({}) - Price: ${:.4}, 24h Change: {:.2}%, Volume: ${:.2}M",
                trend.metadata.name,
                trend.metadata.symbol,
                trend.metadata.price_usd,
                trend.price_change_24h,
                trend.metadata.volume_24h / 1_000_000.0
            );

            let state = self.market_trend_to_token_state(trend.clone());
            info!("Inserting token state into PostgreSQL");
            self.db.save_token_state(&state)?;

            // Format market data for LLM analysis
            let prompt = format!(
                "Analyze trading opportunity for {} ({}). Price: ${:.4}, 24h Change: {:.2}%, Volume: ${:.2}M",
                trend.metadata.name,
                trend.metadata.symbol,
                trend.metadata.price_usd,
                trend.price_change_24h,
                trend.metadata.volume_24h / 1_000_000.0
            );

            // Analyze trading opportunity
            info!("Analyzing trading opportunity with LLM");
            if let Ok(analysis) = self.trading_strategy.analyze_trading_opportunity(&prompt, 1.0).await {
                // Parse the analysis into a trade recommendation
                if let Ok(trade) = serde_json::from_str::<TradeRecommendation>(&analysis) {
                    info!(
                        "Received trade recommendation: Action={:?}, Amount={} SOL, Confidence={:.2}, Risk={}",
                        trade.action, trade.amount_in_sol, trade.confidence, trade.risk_assessment
                    );
                    
                    // Execute trade if confidence is high enough
                    if trade.confidence >= 0.8 {
                        match trade.action {
                            TradeAction::Buy => {
                                info!("Executing BUY order for {} SOL worth of {}", 
                                    trade.amount_in_sol, trend.metadata.symbol);
                                
                                if let Ok(signature) = self.dex.execute_swap(
                                    "So11111111111111111111111111111111111111112", // SOL
                                    &trade.token_address,
                                    trade.amount_in_sol as u64,
                                    &self.wallet,
                                ).await {
                                    info!("Trade executed successfully. Signature: {}", signature);

                                    // Generate and post tweet about the trade
                                    info!("Generating tweet for successful buy");
                                    let tweet = self.personality.generate_trade_tweet(
                                        &format!(
                                            "Action: Buy\nAmount: {} SOL\nToken: {}\nPrice: ${:.4}\nMarket Cap: ${:.2}M\n24h Volume: ${:.2}M\n24h Change: {:.2}%\nContract: {}\nTransaction: {}\nAnalysis: {}\nRisk Assessment: {}\nMarket Analysis:\n- Volume: {}\n- Price Trend: {}\n- Liquidity: {}\n- Momentum: {}",
                                            trade.amount_in_sol,
                                            trend.metadata.symbol,
                                            trend.metadata.price_usd,
                                            trend.metadata.market_cap / 1_000_000.0,
                                            trend.metadata.volume_24h / 1_000_000.0,
                                            trend.price_change_24h,
                                            trend.token_address,
                                            signature,
                                            trade.reasoning,
                                            trade.risk_assessment,
                                            trade.market_analysis.volume_analysis,
                                            trade.market_analysis.price_trend,
                                            trade.market_analysis.liquidity_assessment,
                                            trade.market_analysis.momentum_indicators
                                        ),
                                        "buy",
                                        trade.confidence,
                                    ).await?;
                                    
                                    info!("Posting tweet: {}", tweet);
                                    if let Err(e) = self.twitter.post_tweet(&tweet).await {
                                        warn!("Failed to post trade tweet: {}", e);
                                    }
                                } else {
                                    warn!("Failed to execute buy order");
                                }
                            },
                            TradeAction::Sell => {
                                info!("Executing SELL order for {} SOL worth of {}", 
                                    trade.amount_in_sol, trend.metadata.symbol);
                                
                                if let Ok(signature) = self.dex.execute_swap(
                                    &trade.token_address,
                                    "So11111111111111111111111111111111111111112", // SOL
                                    trade.amount_in_sol as u64,
                                    &self.wallet,
                                ).await {
                                    info!("Trade executed successfully. Signature: {}", signature);

                                    // Generate and post tweet about the trade
                                    info!("Generating tweet for successful sell");
                                    let tweet = self.personality.generate_trade_tweet(
                                        &format!(
                                            "Action: Sell\nAmount: {} SOL\nToken: {}\nPrice: ${:.4}\nMarket Cap: ${:.2}M\n24h Volume: ${:.2}M\n24h Change: {:.2}%\nContract: {}\nTransaction: {}\nAnalysis: {}\nRisk Assessment: {}\nMarket Analysis:\n- Volume: {}\n- Price Trend: {}\n- Liquidity: {}\n- Momentum: {}",
                                            trade.amount_in_sol,
                                            trend.metadata.symbol,
                                            trend.metadata.price_usd,
                                            trend.metadata.market_cap / 1_000_000.0,
                                            trend.metadata.volume_24h / 1_000_000.0,
                                            trend.price_change_24h,
                                            trend.token_address,
                                            signature,
                                            trade.reasoning,
                                            trade.risk_assessment,
                                            trade.market_analysis.volume_analysis,
                                            trade.market_analysis.price_trend,
                                            trade.market_analysis.liquidity_assessment,
                                            trade.market_analysis.momentum_indicators
                                        ),
                                        "sell",
                                        trade.confidence,
                                    ).await?;
                                    
                                    info!("Posting tweet: {}", tweet);
                                    if let Err(e) = self.twitter.post_tweet(&tweet).await {
                                        warn!("Failed to post trade tweet: {}", e);
                                    }
                                } else {
                                    warn!("Failed to execute sell order");
                                }
                            },
                            TradeAction::Hold => {
                                info!("Decision: HOLD {} - {}", 
                                    trend.metadata.symbol, trade.reasoning);
                            }
                        }
                    } else {
                        info!("Skipping trade due to low confidence: {:.2}", trade.confidence);
                    }
                } else {
                    warn!("Failed to parse trade recommendation");
                }
            } else {
                warn!("Failed to get trading analysis from LLM");
            }
        }

        info!("Market data sync cycle complete");
        Ok(())
    }

    fn market_trend_to_token_state(&self, trend: MarketTrend) -> TokenState {
        TokenState {
            address: trend.token_address,
            symbol: trend.metadata.symbol,
            name: trend.metadata.name,
            price_sol: trend.metadata.price_sol,
            price_usd: trend.metadata.price_usd,
            market_cap: trend.metadata.market_cap,
            volume_24h: trend.metadata.volume_24h,
            price_change_24h: trend.price_change_24h,
            volume_change_24h: 0.0, // Placeholder, update as needed
            timestamp: Utc::now(),
        }
    }
}
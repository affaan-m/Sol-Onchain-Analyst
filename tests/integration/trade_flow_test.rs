use cainam_core::{
    agent::trader::{AgentConfig, TradingAgent},
    config::{MarketConfig, mongodb::MongoConfig, pool::MongoPoolConfig},
    error::AgentResult,
    models::{
        market_signal::{MarketSignal, SignalType},
        token_analytics::TokenAnalytics,
    },
    services::token_analytics::TokenAnalyticsService,
    SolanaAgentKit,
};
use chrono::Utc;
use rig_mongodb::MongoDbPool;
use std::sync::Arc;
use bigdecimal::BigDecimal;
use std::time::Duration;

async fn setup_test_db() -> Arc<MongoDbPool> {
    let config = MongoConfig {
        database: "cainam_test".to_string(),
        pool: MongoPoolConfig {
            min_pool_size: 1,
            max_pool_size: 2,
            connect_timeout: Duration::from_secs(5),
        },
        ..Default::default()
    };
    
    config.create_pool()
        .await
        .expect("Failed to create database pool")
}

async fn cleanup_test_db(pool: &MongoDbPool) {
    pool.database("cainam_test")
        .drop()
        .await
        .expect("Failed to cleanup test database");
}

async fn setup_test_config() -> AgentConfig {
    AgentConfig {
        openai_api_key: "test_key".to_string(),
        birdeye_api_key: "test_key".to_string(),
        twitter_email: "test@example.com".to_string(),
        twitter_username: "test_user".to_string(),
        twitter_password: "test_pass".to_string(),
        analysis_interval: std::time::Duration::from_secs(1),
        trade_min_confidence: 0.7,
        trade_max_amount: 1000.0,
    }
}

#[tokio::test]
async fn test_full_trade_flow() -> AgentResult<()> {
    // Setup
    let db = setup_test_db().await;
    let config = setup_test_config().await;
    let solana_agent = SolanaAgentKit::new_from_env()?;
    
    // Initialize trading agent
    let agent = TradingAgent::new(config, db.clone(), solana_agent).await?;
    
    // Test market analysis
    let signal = agent.analyze_market(
        "SOL",
        "So11111111111111111111111111111111111111112"
    ).await?;
    
    assert!(signal.is_some());
    
    // Test signal processing
    if let Some(signal) = signal {
        let action = agent.process_signal(&signal).await?;
        assert!(action.is_some());
        
        // Test trade execution
        if let Some(action) = action {
            match action.as_str() {
                "BUY" | "SELL" => {
                    let result = agent.execute_trade("SOL", &signal).await;
                    assert!(result.is_ok());
                    
                    // Test post-trade update
                    let update_result = agent.post_trade_update(
                        "SOL",
                        &action,
                        100.0,
                        &signal.signal_type
                    ).await;
                    assert!(update_result.is_ok());
                }
                _ => {}
            }
        }
    }
    
    // Cleanup test data
    cleanup_test_db(&db).await;
    Ok(())
}

#[tokio::test]
async fn test_concurrent_market_analysis() -> AgentResult<()> {
    let db = setup_test_db().await;
    let config = setup_test_config().await;
    let solana_agent = SolanaAgentKit::new_from_env()?;
    
    let agent = TradingAgent::new(config, db.clone(), solana_agent).await?;
    
    // Run multiple market analyses concurrently
    let handles: Vec<_> = vec![
        ("SOL", "So11111111111111111111111111111111111111112"),
        ("BONK", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
    ]
    .into_iter()
    .map(|(symbol, address)| {
        let agent = agent.clone();
        tokio::spawn(async move {
            agent.analyze_market(symbol, address).await
        })
    })
    .collect();
    
    // Wait for all analyses to complete
    for handle in handles {
        let result = handle.await.expect("Task panicked")?;
        assert!(result.is_some());
    }
    
    cleanup_test_db(&db).await;
    Ok(())
}

#[tokio::test]
async fn test_error_recovery() -> AgentResult<()> {
    let db = setup_test_db().await;
    let config = setup_test_config().await;
    let solana_agent = SolanaAgentKit::new_from_env()?;
    
    let agent = TradingAgent::new(config, db.clone(), solana_agent).await?;
    
    // Start the agent
    let agent_handle = {
        let agent = agent.clone();
        tokio::spawn(async move {
            agent.run().await
        })
    };
    
    // Let it run for a bit
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Stop the agent
    agent.stop();
    
    // Verify clean shutdown
    let result = agent_handle.await.expect("Task panicked");
    assert!(result.is_ok());
    
    cleanup_test_db(&db).await;
    Ok(())
}

#[tokio::test]
async fn test_performance() -> AgentResult<()> {
    use tokio::time::Instant;
    
    let db = setup_test_db().await;
    let config = setup_test_config().await;
    let solana_agent = SolanaAgentKit::new_from_env()?;
    
    let agent = TradingAgent::new(config, db.clone(), solana_agent).await?;
    
    // Measure market analysis performance
    let start = Instant::now();
    let signal = agent.analyze_market(
        "SOL",
        "So11111111111111111111111111111111111111112"
    ).await?;
    let duration = start.elapsed();
    
    // Analysis should complete within reasonable time
    assert!(duration.as_secs() < 5);
    assert!(signal.is_some());
    
    cleanup_test_db(&db).await;
    Ok(())
}
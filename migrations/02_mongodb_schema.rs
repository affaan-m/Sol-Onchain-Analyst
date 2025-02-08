use rig_mongodb::{MongoDbPool, bson::doc};
use anyhow::Result;
use tracing::info;
use crate::config::mongodb::MongoConfig;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    info!("Running MongoDB schema migration...");
    
    let config = MongoConfig::from_env();
    let pool = config.create_pool().await?;
    let db = pool.database(&config.database);

    // Create market signals collection with timeseries optimization
    db.create_collection("market_signals", Some(doc! {
        "timeseries": {
            "timeField": "timestamp",
            "metaField": "asset_address",
            "granularity": "minutes"
        },
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["asset_address", "signal_type", "confidence", "timestamp"],
                "properties": {
                    "asset_address": { "bsonType": "string" },
                    "signal_type": { 
                        "enum": ["BUY", "SELL", "HOLD", "STRONG_BUY", "STRONG_SELL", 
                                "PRICE_SPIKE", "PRICE_DROP", "VOLUME_SURGE"] 
                    },
                    "confidence": { "bsonType": "decimal128" },
                    "price_change_24h": { "bsonType": "decimal128" },
                    "volume_change_24h": { "bsonType": "decimal128" },
                    "risk_score": { "bsonType": "decimal128" },
                    "metadata": { "bsonType": "object" },
                    "timestamp": { "bsonType": "date" }
                }
            }
        }
    })).await?;

    // Create trade executions collection with timeseries
    db.create_collection("trade_executions", Some(doc! {
        "timeseries": {
            "timeField": "execution_time",
            "metaField": "asset_address",
            "granularity": "minutes"
        },
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["asset_address", "amount", "price", "status", "execution_time"],
                "properties": {
                    "asset_address": { "bsonType": "string" },
                    "amount": { "bsonType": "decimal128" },
                    "price": { "bsonType": "decimal128" },
                    "status": { 
                        "enum": ["PENDING", "EXECUTED", "FAILED", "CANCELLED"] 
                    },
                    "tx_signature": { "bsonType": "string" },
                    "metadata": { "bsonType": "object" },
                    "execution_time": { "bsonType": "date" }
                }
            }
        }
    })).await?;

    // Create token analytics collection with timeseries
    db.create_collection("token_analytics", Some(doc! {
        "timeseries": {
            "timeField": "timestamp",
            "metaField": "token_address",
            "granularity": "minutes"
        },
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["token_address", "token_name", "price", "timestamp"],
                "properties": {
                    "token_address": { "bsonType": "string" },
                    "token_name": { "bsonType": "string" },
                    "token_symbol": { "bsonType": "string" },
                    "price": { "bsonType": "decimal128" },
                    "volume_24h": { "bsonType": "decimal128" },
                    "market_cap": { "bsonType": "decimal128" },
                    "holder_count": { "bsonType": "int" },
                    "metadata": { "bsonType": "object" },
                    "timestamp": { "bsonType": "date" }
                }
            }
        }
    })).await?;

    // Create agent performance collection
    db.create_collection("agent_performance", Some(doc! {
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["period_start", "period_end", "total_trades", "success_rate"],
                "properties": {
                    "period_start": { "bsonType": "date" },
                    "period_end": { "bsonType": "date" },
                    "total_trades": { "bsonType": "int" },
                    "success_rate": { "bsonType": "decimal128" },
                    "pnl": { "bsonType": "decimal128" },
                    "metadata": { "bsonType": "object" }
                }
            }
        }
    })).await?;

    // Create test collections with timeseries optimization
    db.create_collection("test_market_signals", Some(doc! {
        "timeseries": {
            "timeField": "timestamp",
            "metaField": "asset_address",
            "granularity": "minutes"
        },
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["asset_address", "signal_type", "confidence", "timestamp"],
                "properties": {
                    "asset_address": { "bsonType": "string" },
                    "signal_type": { 
                        "enum": ["BUY", "SELL", "HOLD", "STRONG_BUY", "STRONG_SELL", 
                                "PRICE_SPIKE", "PRICE_DROP", "VOLUME_SURGE"] 
                    },
                    "confidence": { "bsonType": "decimal128" },
                    "price_change_24h": { "bsonType": "decimal128" },
                    "volume_change_24h": { "bsonType": "decimal128" },
                    "risk_score": { "bsonType": "decimal128" },
                    "metadata": { "bsonType": "object" },
                    "timestamp": { "bsonType": "date" }
                }
            }
        }
    })).await?;

    // Create trade executions collection
    db.create_collection("test_trade_executions", Some(doc! {
        "timeseries": {
            "timeField": "execution_time",
            "metaField": "asset_address",
            "granularity": "minutes"
        }
    })).await?;

    // Create time-based indexes for efficient querying
    db.collection("market_signals").create_index(
        doc! {
            "asset_address": 1,
            "timestamp": -1
        },
        None,
    ).await?;

    db.collection("trade_executions").create_index(
        doc! {
            "asset_address": 1,
            "execution_time": -1
        },
        None,
    ).await?;

    db.collection("token_analytics").create_index(
        doc! {
            "token_address": 1,
            "timestamp": -1
        },
        None,
    ).await?;

    info!("MongoDB schema migration completed successfully!");
    Ok(())
}
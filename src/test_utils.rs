use rig_mongodb::{MongoDbPool, bson::doc};
use std::sync::Arc;
use anyhow::Result;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use crate::config::mongodb::MongoConfig;

// Ensure test databases are unique per test
static TEST_DB_COUNTER: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(0));

pub async fn get_unique_test_db_name() -> String {
    let mut counter = TEST_DB_COUNTER.lock().await;
    let db_name = format!("test_db_{}", *counter);
    *counter += 1;
    db_name
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::config::{mongodb::MongoConfig, pool::MongoPoolConfig};
    use rig_mongodb::MongoDbPool;
    use std::sync::Arc;
    use anyhow::Result;
    use std::time::Duration;

    pub async fn setup_test_db() -> Result<(Arc<MongoDbPool>, String)> {
        let db_name = get_unique_test_db_name().await;
        
        let config = MongoConfig {
            database: db_name.clone(),
            pool: crate::config::pool::MongoPoolConfig {
                min_pool_size: 1,
                max_pool_size: 2,
                connect_timeout: std::time::Duration::from_secs(5),
            },
            ..Default::default()
        };
        
        let pool = config.create_pool().await?;
        
        // Initialize test collections
        setup_test_collections(&pool, &db_name).await?;
        
        Ok((pool, db_name))
    }
    
    pub async fn cleanup_test_db(pool: &MongoDbPool, db_name: &str) -> Result<()> {
        pool.database(db_name).drop().await?;
        Ok(())
    }
    
    async fn setup_test_collections(pool: &MongoDbPool, db_name: &str) -> Result<()> {
        let db = pool.database(db_name);
        
        // Create minimal test collections with indexes
        db.create_collection("test_market_signals", Some(doc! {
            "timeseries": {
                "timeField": "timestamp",
                "metaField": "asset_address",
                "granularity": "minutes"
            }
        })).await?;
        
        db.collection("test_market_signals").create_index(
            doc! {
                "asset_address": 1,
                "timestamp": -1
            },
            None,
        ).await?;
        
        Ok(())
    }
    
    pub async fn insert_test_data(pool: &MongoDbPool, db_name: &str, collection: &str, data: Vec<bson::Document>) -> Result<()> {
        let coll = pool.database(db_name).collection(collection);
        coll.insert_many(data, None).await?;
        Ok(())
    }
}
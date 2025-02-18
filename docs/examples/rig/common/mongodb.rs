use rig_mongodb::MongoDbPool;
use anyhow::Result;
use std::sync::Arc;
use crate::config::mongodb::MongoConfig;

pub async fn create_mongo_pool() -> Result<Arc<MongoDbPool>> {
    let config = MongoConfig::from_env();
    config.create_pool().await
}

pub async fn validate_connection(pool: &MongoDbPool) -> Result<()> {
    pool.database("admin")
        .run_command(rig_mongodb::bson::doc! { "ping": 1 }, None)
        .await?;
    Ok(())
}
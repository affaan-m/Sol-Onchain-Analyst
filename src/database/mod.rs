use rig_mongodb::{MongoDbPool, bson::{self, doc}, Collection, Database};
use crate::config::mongodb::MongoConfig;
use anyhow::Result;
use std::sync::Arc;

pub mod positions;
pub mod sync;

pub async fn init_mongodb() -> Result<Arc<MongoDbPool>> {
    let config = MongoConfig::from_env();
    let pool = config.create_pool().await?;
    
    // Ping the database to ensure connection
    pool.database("admin")
        .run_command(doc! { "ping": 1 }, None)
        .await?;

    Ok(pool)
}

// Helper to get vector index for collections
pub async fn get_vector_index(pool: &MongoDbPool, collection_name: &str) -> Result<MongoDbVectorIndex> {
    let collection = pool.collection(collection_name);
    let index = MongoDbVectorIndex::new(
        collection,
        rig::providers::openai::TEXT_EMBEDDING_3_SMALL,
        "vector_index",
        rig_mongodb::SearchParams::new()
    ).await?;
    
    Ok(index)
}
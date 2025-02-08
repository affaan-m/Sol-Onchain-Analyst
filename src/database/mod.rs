pub use rig_mongodb::{
    Collection, MongoDbPool, MongoDbError, MongoDbVectorIndex,
    options::{FindOptions, FindOneOptions},
    bson::{self, doc, Document, DateTime, ObjectId},
};

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use crate::config::mongodb::MongoConfig;
use tracing::error;

pub mod mongodb;
pub mod sync;

#[derive(Clone)]
pub struct DatabaseManager {
    pool: MongoDbPool,
    config: MongoConfig,
}

impl DatabaseManager {
    pub async fn new(config: MongoConfig) -> Result<Self> {
        let pool = config.create_pool().await?;
        Ok(Self { pool, config })
    }

    pub fn get_pool(&self) -> &MongoDbPool {
        &self.pool
    }

    pub fn get_database(&self) -> Database {
        self.pool.database(&self.config.database)
    }
}

pub trait MongoDbExtensions {
    fn get_collection<T>(&self, name: &str) -> Collection<T> 
    where 
        T: Serialize + DeserializeOwned;

    async fn find_one_by_id<T>(&self, collection: &str, id: bson::oid::ObjectId) -> Result<Option<T>> 
    where 
        T: Serialize + DeserializeOwned;

    async fn find_one_by_filter<T>(&self, collection: &str, filter: bson::Document) -> Result<Option<T>>
    where 
        T: Serialize + DeserializeOwned;

    async fn find_with_sort<T>(&self, collection: &str, filter: bson::Document, sort: bson::Document, limit: Option<i64>) -> Result<Vec<T>>
    where 
        T: Serialize + DeserializeOwned;
}

impl MongoDbExtensions for Database {
    fn get_collection<T>(&self, name: &str) -> Collection<T> 
    where 
        T: Serialize + DeserializeOwned 
    {
        self.collection(name)
    }

    async fn find_one_by_id<T>(&self, collection: &str, id: bson::oid::ObjectId) -> Result<Option<T>> 
    where 
        T: Serialize + DeserializeOwned 
    {
        let filter = doc! { "_id": id };
        Ok(self.collection(collection).find_one(filter, None).await?)
    }

    async fn find_one_by_filter<T>(&self, collection: &str, filter: bson::Document) -> Result<Option<T>>
    where 
        T: Serialize + DeserializeOwned 
    {
        Ok(self.collection(collection).find_one(filter, None).await?)
    }

    async fn find_with_sort<T>(&self, collection: &str, filter: bson::Document, sort: bson::Document, limit: Option<i64>) -> Result<Vec<T>>
    where 
        T: Serialize + DeserializeOwned 
    {
        let options = FindOptions::builder()
            .sort(sort)
            .limit(limit)
            .build();

        let mut cursor = self.collection(collection).find(filter, options).await?;
        let mut results = Vec::new();
        
        while let Some(doc) = cursor.try_next().await? {
            results.push(doc);
        }
        
        Ok(results)
    }
}

// Vector store configuration helper
pub fn create_vector_search_params() -> SearchParams {
    SearchParams::new()
        .with_distance_metric("cosine")
        .with_embedding_field("vector")
        .with_index_type("hnsw")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db;

    #[tokio::test]
    async fn test_database_extensions() {
        let (pool, db_name) = setup_test_db().await.unwrap();
        let db = pool.database(&db_name);

        // Test find_one_by_filter
        let filter = doc! { "test_field": "test_value" };
        let result = db.find_one_by_filter::<Document>("test_collection", filter).await;
        assert!(result.is_ok());
    }
}
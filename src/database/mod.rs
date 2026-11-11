use std::sync::Arc;

use async_trait::async_trait;
pub use mongodb::{
    Collection,
    options::{FindOptions, FindOneOptions},
    bson::{self, doc, Document, DateTime},
};
use crate::config::mongodb::{MongoConfig, MongoDbPool};
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
// pub mod sync;

#[derive(Clone)]
pub struct DatabaseManager {
    pool: Arc<MongoDbPool>,
}

impl DatabaseManager {
    pub async fn new(config: MongoConfig) -> Result<Self> {
        let pool = MongoDbPool::create_pool(config).await?;
        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &MongoDbPool {
        &self.pool
    }

    pub fn get_database(&self, name: &str) -> mongodb::Database {
        self.pool.database(name)
    }
}

#[async_trait]
pub trait MongoDbExtensions {
    fn get_collection<T>(&self, name: &str) -> Collection<T> 
    where 
    T: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static;

    async fn find_one_by_id<T>(&self, collection: &str, id: bson::oid::ObjectId) -> Result<Option<T>> 
    where 
    T: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static;

    async fn find_one_by_filter<T>(&self, collection: &str, filter: bson::Document) -> Result<Option<T>>
    where 
    T: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static;

    async fn find_with_sort<T>(&self, collection: &str, filter: bson::Document, sort: bson::Document, limit: Option<i64>) -> Result<Vec<T>>
    where 
    T: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static;
}

// impl MongoDbExtensions for mongodb::Database {
//     fn get_collection<T>(&self, name: &str) -> Collection<T> 
//     where 
//     T: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static
//     {
//         self.collection(name)
//     }

//     async fn find_one_by_id<T>(&self, collection: &str, id: bson::oid::ObjectId) -> Result<Option<T>>
//     where
//         T: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static, // Crucial change
//     {
//         let filter = doc! { "_id": id };
//         let collection: Collection<T> = self.collection(collection); // Type hint for clarity
//         Ok(collection.find_one(filter).await?)
//     }

//     async fn find_one_by_filter<T>(&self, collection: &str, filter: bson::Document) -> Result<Option<T>>
//     where 
//     T: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static, // Crucial change
//     {
//         Ok(self.collection(collection).find_one(filter).await?)
//     }

//     async fn find_with_sort<T>(&self, collection: &str, filter: bson::Document, sort: bson::Document, limit: Option<i64>) -> Result<Vec<T>>
//     where 
//     T: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static, // Crucial change
//     {
//         let options = FindOptions::builder()
//             .sort(sort)
//             .limit(limit)
//             .build();

//         let mut cursor = self.collection(collection).find(filter).await?;
//         let mut results = Vec::new();
        
//         while let Some(doc) = cursor.try_next().await? {
//             results.push(doc);
//         }
        
//         Ok(results)
//     }
// }

// Vector store configuration helper
// pub fn create_vector_search_params() -> SearchParams {
//     SearchParams::new()
//         .with_distance_metric("cosine")
//         .with_embedding_field("vector")
//         .with_index_type("hnsw")
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::test_utils::setup_test_db;

//     #[tokio::test]
//     async fn test_database_extensions() {
//         let (pool, db_name) = setup_test_db().await.unwrap();
//         let db = pool.database(&db_name);

//         // Test find_one_by_filter
//         let filter = doc! { "test_field": "test_value" };
//         let result = db.find_one_by_filter::<Document>("test_collection", filter).await;
//         assert!(result.is_ok());
//     }
// }
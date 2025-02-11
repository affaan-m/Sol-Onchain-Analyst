use anyhow::{Context, Result};
use rig_core::vector_store::{Document, Store};
use rig_mongodb::MongoStore;
use std::sync::Arc;
use tracing::{info, warn};
use crate::config::mongodb::{MongoConfig, MongoDbPool};
use serde_json;

pub struct VectorStore {
    store: Arc<MongoStore>,
}

impl VectorStore {
    pub async fn new() -> Result<Self> {
        // Use centralized MongoDB configuration
        let config = MongoConfig::from_env();
        info!("Initializing vector store connection");
        
        let pool = MongoDbPool::create_pool(config.clone())
            .await
            .context("Failed to create MongoDB pool")?;
            
        // Configure vector store with optimized search parameters and fields
        let fields = serde_json::json!({
            "fields": [{
                "path": "embedding",
                "numDimensions": 1536,
                "similarity": "cosine"
            }]
        });
            
        let store = MongoStore::new(
            pool.client(), 
            &config.database, 
            "token_analytics",
            fields
        ).await
            .context("Failed to create vector store")?;

        Ok(Self {
            store: Arc::new(store),
        })
    }

    pub async fn insert_documents<T>(&self, documents: Vec<T>) -> Result<()> 
    where
        T: Send + Sync + 'static + serde::Serialize + Document,
    {
        info!("Inserting documents into vector store");
        self.store.insert_documents(&documents)
            .await
            .context("Failed to insert documents into vector store")?;
        Ok(())
    }

    pub async fn top_n<T>(&self, query: &str, limit: usize) -> Result<Vec<(f32, T)>>
    where
        T: Send + Sync + for<'de> serde::de::Deserialize<'de> + 'static,
    {
        if limit == 0 {
            warn!("top_n called with limit=0, defaulting to 1");
            let limit = 1;
        }
        
        info!("Performing vector similarity search with limit {}", limit);
        let results = self.store.search::<T>(query, limit)
            .await
            .context("Failed to perform vector similarity search")?;
            
        info!("Found {} matching documents", results.len());
        Ok(results)
    }

    #[cfg(test)]
    pub async fn cleanup_test_data(&self) -> Result<()> {
        // Implement cleanup logic for MongoDB if necessary
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    #[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    struct TestDocument {
        id: String,
        content: String,
    }

    impl Document for TestDocument {
        fn text(&self) -> &str {
            &self.content
        }
    }

    fn init_test_logging() {
        let _ = tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()))
            .with(tracing_subscriber::fmt::layer())
            .try_init();
    }

    #[tokio::test]
    async fn test_vector_store() -> Result<()> {
        init_test_logging();
        dotenvy::dotenv().ok();
        
        let store = VectorStore::new()
            .await
            .context("Failed to create vector store")?;
            
        // Clean up any existing test data
        store.cleanup_test_data()
            .await
            .context("Failed to cleanup existing test data")?;
        
        let docs = vec![
            TestDocument {
                id: "1".to_string(),
                content: "Test document one".to_string(),
            },
            TestDocument {
                id: "2".to_string(), 
                content: "Test document two".to_string(),
            },
        ];

        store.insert_documents(docs)
            .await
            .context("Failed to insert test documents")?;

        let results = store.top_n::<TestDocument>("test document", 2)
            .await
            .context("Failed to perform similarity search")?;
            
        assert!(!results.is_empty(), "Expected non-empty search results");
        assert_eq!(results.len(), 2, "Expected exactly 2 search results");

        // Clean up test data
        store.cleanup_test_data()
            .await
            .context("Failed to cleanup test data")?;

        Ok(())
    }
}

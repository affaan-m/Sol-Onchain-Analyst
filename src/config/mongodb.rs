use rig_mongodb::{MongoDbPool, options::ClientOptions};
use std::sync::Arc;
use anyhow::Result;
use crate::config::pool::MongoPoolConfig;

#[derive(Debug, Clone)]
pub struct MongoConfig {
    pub uri: String,
    pub database: String,
    pub app_name: Option<String>,
    pub pool: MongoPoolConfig,
}

impl Default for MongoConfig {
    fn default() -> Self {
        Self {
            uri: "mongodb://localhost:27017".to_string(),
            database: "cainam".to_string(),
            app_name: Some("cainam-core".to_string()),
            pool: MongoPoolConfig::default(),
        }
    }
}

impl MongoConfig {
    pub fn from_env() -> Self {
        Self {
            uri: std::env::var("MONGODB_URI")
                .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
            database: std::env::var("MONGODB_DATABASE")
                .unwrap_or_else(|_| "cainam".to_string()),
            app_name: std::env::var("MONGODB_APP_NAME").ok(),
            pool: MongoPoolConfig::from_env(),
        }
    }

    pub async fn create_pool(&self) -> Result<Arc<MongoDbPool>> {
        let mut client_options = ClientOptions::parse(&self.uri).await?;
        
        if let Some(app_name) = &self.app_name {
            client_options.app_name = Some(app_name.clone());
        }
        
        // Apply pool configuration
        self.pool.apply_to_options(&mut client_options);
        
        let pool = MongoDbPool::new_with_options(&self.uri, client_options).await?;
        Ok(Arc::new(pool))
    }
}
use std::sync::Arc;
use anyhow::Result;
use mongodb::{
    options::ClientOptions,
    Client,
    bson::doc,
};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MongoPoolConfig {
    pub min_pool_size: u32,
    pub max_pool_size: u32,
    pub connect_timeout: Duration,
}

impl Default for MongoPoolConfig {
    fn default() -> Self {
        Self {
            min_pool_size: 5,
            max_pool_size: 10,
            connect_timeout: Duration::from_secs(20),
        }
    }
}

impl MongoPoolConfig {
    pub fn from_env() -> Self {
        Self {
            min_pool_size: std::env::var("MONGODB_MIN_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            max_pool_size: std::env::var("MONGODB_MAX_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            connect_timeout: Duration::from_millis(
                std::env::var("MONGODB_CONNECT_TIMEOUT_MS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(20000)
            ),
        }
    }

    pub fn apply_to_options(&self, options: &mut ClientOptions) {
        options.min_pool_size = Some(self.min_pool_size);
        options.max_pool_size = Some(self.max_pool_size);
        options.connect_timeout = Some(self.connect_timeout);
    }
}

#[derive(Debug, Clone)]
pub struct MongoConfig {
    pub uri: String,
    pub database: String,
    pub app_name: Option<String>,
    pub pool_config: MongoPoolConfig,
}

impl Default for MongoConfig {
    fn default() -> Self {
        Self {
            uri: "mongodb://localhost:32770".to_string(),
            database: "cainam".to_string(),
            app_name: Some("cainam-core".to_string()),
            pool_config: MongoPoolConfig::default(),
        }
    }
}

impl MongoConfig {
    pub fn from_env() -> Self {
        Self {
            uri: std::env::var("MONGODB_URI")
                .unwrap_or_else(|_| "mongodb://localhost:32770".to_string()),
            database: std::env::var("MONGODB_DATABASE")
                .unwrap_or_else(|_| "cainam".to_string()),
            app_name: std::env::var("MONGODB_APP_NAME").ok(),
            pool_config: MongoPoolConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MongoDbPool {
    client: Client,
    config: MongoConfig,
}

impl MongoDbPool {
    pub async fn create_pool(config: MongoConfig) -> Result<Arc<MongoDbPool>> {
        let mut client_options = ClientOptions::parse(&config.uri).await?;
        
        if let Some(app_name) = &config.app_name {
            client_options.app_name = Some(app_name.clone());
        }
        
        // Set server API version to ensure compatibility
        client_options.server_api = Some(mongodb::options::ServerApi::builder().version(mongodb::options::ServerApiVersion::V1).build());
        
        // Apply pool configuration
        config.pool_config.apply_to_options(&mut client_options);
                
        let client = Client::with_options(client_options)?;

        // Test the connection
        client.database("admin").run_command(doc! {"ping": 1}).await?;

        Ok(Arc::new(MongoDbPool {
            client,
            config,
        }))
    }

    pub fn database(&self, name: &str) -> mongodb::Database {
        self.client.database(name)
    }

    pub fn get_config(&self) -> &MongoConfig {
        &self.config
    }
}
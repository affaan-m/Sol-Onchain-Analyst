use std::sync::Arc;
use anyhow::Result;
use mongodb::{
    bson::doc, options::ClientOptions, Client, Collection
};
use rig::{embeddings::EmbeddingsBuilder, providers::openai::EmbeddingModel, vector_store::VectorStoreIndexDyn, Embed, Embed as TEmbed};
use rig_mongodb::{MongoDbVectorIndex, SearchParams};
// use rig_derive::Embed;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
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

    pub fn client(&self) -> &Client {
        &self.client
    }
}


#[derive(Embed, Clone, Deserialize, Debug)]
pub struct TokenAnalyticsData {
    #[serde(rename = "_id", deserialize_with = "deserialize_object_id")]
    id: String,

    #[embed]
    token_address: String,

    #[embed]
    token_name: String,

    #[embed]
    token_symbol: String,
}

fn deserialize_object_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Object(map) => {
            if let Some(Value::String(oid)) = map.get("$oid") {
                Ok(oid.to_string())
            } else {
                Err(serde::de::Error::custom(
                    "Expected $oid field with string value",
                ))
            }
        }
        _ => Err(serde::de::Error::custom(
            "Expected string or object with $oid field",
        )),
    }
}

/*
* Move insert_documents & top_n functions from vector_store to mongodb.

TODO:
* need more tests
* fix hardcoded 
*/

impl MongoDbPool {
    pub async fn insert_token_analytics_documents<T: TEmbed + Send>(&self, collection: &str, model: EmbeddingModel, documents: Vec<TokenAnalyticsData>) -> Result<()> {
        let collection: Collection<bson::Document> = self.client
        .database("cainam")
        .collection(collection);

        let embeddings = EmbeddingsBuilder::new(model.clone())
        .documents(documents)?
        .build()
        .await?;

        let mongo_documents = embeddings
        .iter()
        .map(|(TokenAnalyticsData { id, token_address, .. }, embedding)| {
            doc! {
                "id": id.clone(),
                "token_address": token_address.clone(),
                "embedding": embedding.first().vec.clone(),
            }
        })
        .collect::<Vec<_>>();

        collection.insert_many(mongo_documents).await?;

        Ok(())
    }

    pub async fn top_n(&self, collection: &str, model: EmbeddingModel, query: &str, limit: usize) -> Result<Vec<(f64, String, Value)>>
    {
        let collection: Collection<bson::Document> = self.client
        .database("cainam")
        .collection(collection);

        let index =
        MongoDbVectorIndex::new(collection, model, "vector_index", SearchParams::new()).await.expect("msg");

        // Query the index
        let data = index.top_n(query, limit).await?;

        Ok(data)
    }
}
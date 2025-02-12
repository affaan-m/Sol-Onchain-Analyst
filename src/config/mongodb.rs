use anyhow::{Result, Context, anyhow};
use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client, Database,
};
use rig::{
    embeddings::{EmbeddingsBuilder, Embed},
    providers::openai::EmbeddingModel,
    vector_store::VectorStoreIndexDyn,
};
use rig_mongodb::{MongoDbVectorIndex, SearchParams};
use serde::{Deserialize, Serialize, Deserializer};
use serde_json::Value;
use std::{env, sync::Arc, time::Duration};
use async_trait::async_trait;
use futures::TryStreamExt;

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
                    .unwrap_or(20000),
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
        let uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
        let database = env::var("MONGODB_DATABASE").expect("MONGODB_DATABASE must be set");

        Self {
            uri,
            database,
            app_name: None,
            pool_config: MongoPoolConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenAnalyticsData {
    #[serde(rename = "_id", deserialize_with = "deserialize_object_id")]
    pub id: String,
    pub token_address: String,
    pub token_name: String,
    pub token_symbol: String,
    pub embedding: Vec<f32>,
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

#[derive(Clone)]
pub struct MongoDbPool {
    client: Client,
    config: MongoConfig,
    db: Database,
}

impl MongoDbPool {
    pub async fn create_pool(config: MongoConfig) -> Result<Arc<MongoDbPool>> {
        let mut client_options = ClientOptions::parse(&config.uri).await?;

        if let Some(app_name) = &config.app_name {
            client_options.app_name = Some(app_name.clone());
        }

        // Set server API version to ensure compatibility
        client_options.server_api = Some(
            mongodb::options::ServerApi::builder()
                .version(mongodb::options::ServerApiVersion::V1)
                .build(),
        );

        // Apply pool configuration
        config.pool_config.apply_to_options(&mut client_options);

        let client = Client::with_options(client_options)?;
        let db = client.database(&config.database);

        // Test the connection
        client
            .database("admin")
            .run_command(doc! {"ping": 1})
            .await?;

        Ok(Arc::new(MongoDbPool { client, config, db }))
    }

    pub fn database(&self, name: &str) -> mongodb::Database {
        self.db.clone()
    }

    pub fn get_config(&self) -> &MongoConfig {
        &self.config
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}

#[async_trait]
pub trait TokenAnalyticsDataExt {
    async fn insert_token_analytics_documents<T>(
        &self,
        collection_name: &str,
        embedding_model: EmbeddingModel,
        documents: Vec<T>,
    ) -> Result<()>
    where
        T: Serialize + Send + Sync + Embed;

    async fn top_n(
        &self,
        collection_name: &str,
        embedding_model: EmbeddingModel,
        query: &str,
        limit: i64,
    ) -> Result<Vec<Document>>;
}

#[async_trait]
impl TokenAnalyticsDataExt for MongoDbPool {
    async fn insert_token_analytics_documents<T>(
        &self,
        collection_name: &str,
        embedding_model: EmbeddingModel,
        documents: Vec<T>,
    ) -> Result<()>
    where
        T: Serialize + Send + Sync + Embed,
    {
        let collection = self.db.collection::<Document>(collection_name);

        /* let index_options = IndexOptions::builder()
            .name("vector_index".to_string())
            .build();
        let index_model = IndexModel::builder()
            .keys(doc! { "embedding": "vector" })
            .options(index_options)
            .build();

        if let Err(e) = collection.create_index(index_model).await {
            if !e.to_string().contains("Index already exists") {
                return Err(e.into());
            }
        } */

        let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
            .documents(documents)?
            .build()
            .await?;

        for (doc, embedding) in embeddings {
            let token_data_doc = bson::to_document(&doc)
                .map_err(|e| anyhow!("Serialization error: {}", e))?;
            let mut doc_with_embedding = token_data_doc;
            doc_with_embedding.insert("embedding", bson::to_bson(&embedding)?);
            collection.insert_one(doc_with_embedding).await?;
        }

        Ok(())
    }

    async fn top_n(
        &self,
        collection_name: &str,
        embedding_model: EmbeddingModel,
        query: &str,
        limit: i64,
    ) -> Result<Vec<Document>> {
        let collection = self.db.collection::<TokenAnalyticsData>(collection_name);
        
        let index = MongoDbVectorIndex::<_, TokenAnalyticsData>::new(
            collection,
            embedding_model,
            "vector_index",
            SearchParams::new()
        ).await?;

        let results = index
            .top_n(query, limit as usize)
            .await
            .context("Failed to perform vector search")?;

        let documents = results
            .into_iter()
            .map(|(_, _, doc)| bson::to_document(&doc))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(documents)
    }
}

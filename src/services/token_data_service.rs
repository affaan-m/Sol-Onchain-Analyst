use anyhow::Result;
use bson::DateTime;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc},
    Client, Collection, Database,
};
use std::sync::Arc;
use std::time::SystemTime;
use mongodb::options::FindOptions;
use chrono::{DateTime as ChronoDateTime, Utc};

use crate::{
    birdeye::{api::{BirdeyeApi, BirdeyeClient}},
    config::mongodb::{TokenAnalyticsData, MongoDbPool},
    error::AgentResult,
};

const COLLECTION_NAME: &str = "token_analytics";

pub struct TokenDataService {
    database: Database,
    birdeye_client: Arc<dyn BirdeyeApi>,
    collection: Collection<TokenAnalyticsData>,
}

impl TokenDataService {
    pub async fn new(mongo_uri: String, birdeye_api_key: String) -> Result<Self> {
        let client = Client::with_uri_str(&mongo_uri).await?;
        let database = client.database("cainam");
        let collection = database.collection(COLLECTION_NAME);
        
        let birdeye_client = Arc::new(BirdeyeClient::new(birdeye_api_key)) as Arc<dyn BirdeyeApi>;

        Ok(Self {
            database,
            birdeye_client,
            collection,
        })
    }

    pub async fn new_with_pool(pool: Arc<MongoDbPool>, birdeye_api_key: String) -> Result<Self> {
        let database = pool.database("");
        let collection = database.collection(COLLECTION_NAME);
        
        let birdeye_client = Arc::new(BirdeyeClient::new(birdeye_api_key)) as Arc<dyn BirdeyeApi>;

        Ok(Self {
            database,
            birdeye_client,
            collection,
        })
    }

    pub async fn get_latest_token_data(&self, token_address: &str) -> Result<Option<TokenAnalyticsData>> {
        let filter = doc! { "token_address": token_address };
        let options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(1)
            .build();

        let mut cursor = self.collection.find(filter).await?;
        Ok(cursor.try_next().await?)
    }

    pub async fn get_token_history(
        &self,
        token_address: &str,
        start_time: ChronoDateTime<Utc>,
        end_time: ChronoDateTime<Utc>,
    ) -> Result<Vec<TokenAnalyticsData>> {
        let filter = doc! {
            "token_address": token_address,
            "timestamp": {
                "$gte": DateTime::from_millis(start_time.timestamp_millis()),
                "$lte": DateTime::from_millis(end_time.timestamp_millis())
            }
        };

        let cursor = self.collection.find(filter).await?;
        Ok(cursor.try_collect().await?)
    }

    pub async fn store_token_data(&self, token_data: TokenAnalyticsData) -> AgentResult<()> {
        self.collection.insert_one(token_data).await?;
        Ok(())
    }

    pub async fn get_token_data(&self, filter: bson::Document) -> AgentResult<Option<TokenAnalyticsData>> {
        let mut cursor = self.collection.find(filter).await?;
        Ok(cursor.try_next().await?)
    }

    pub async fn get_historical_data(
        &self,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> AgentResult<Vec<TokenAnalyticsData>> {
        let filter = doc! {
            "timestamp": {
                "$gte": DateTime::from_system_time(start_time),
                "$lte": DateTime::from_system_time(end_time)
            }
        };

        let mut cursor = self.collection.find(filter).await?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await? {
            results.push(doc);
        }
        Ok(results)
    }
} 
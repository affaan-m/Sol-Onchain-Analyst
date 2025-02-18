use anyhow::Result;
use cainam_core::birdeye::BirdeyeClient;
// use futures::TryStreamExt; // No longer needed here (it was for the commented-out function)
use mongodb::{
    bson::{doc, Document},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use tracing::info;

// Define a struct to represent the token data we'll store.  Adapt this to your actual data.
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenData {
    pub symbol: String,
    pub address: String,
    pub price: f64,
    // Add other fields as needed, e.g., volume, market_cap, etc.
    pub timestamp: mongodb::bson::DateTime,
}

pub struct TokenDataService {
    client: mongodb::Client,
    db: mongodb::Database,
    birdeye_client: BirdeyeClient,
    collection: Collection<TokenData>, // Use TokenData here
}

impl TokenDataService {
    pub async fn new(mongo_uri: String, birdeye_api_key: String) -> Result<Self> {
        let client = Client::with_uri_str(&mongo_uri).await?;
        let db = client.database("cainam");
        let collection = db.collection::<TokenData>("market_data"); // Use TokenData
        let birdeye_client = BirdeyeClient::new(birdeye_api_key);

        Ok(Self {
            client,
            db,
            birdeye_client,
            collection,
        })
    }

    pub async fn update_token_data(&self, address: &str, symbol: &str) -> Result<()> {
        let market_data = match self.birdeye_client.get_market_data(address).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Error fetching market data from Birdeye: {}", e);
                return Err(e);
            }
        };

        let token_data = TokenData {
            symbol: symbol.to_string(),
            address: address.to_string(),
            price: market_data.price,
            timestamp: mongodb::bson::DateTime::now(),
        };

        self.collection.insert_one(token_data, None).await?;

        info!("Updated market data for {} ({})", symbol, address);
        Ok(())
    }

    // Commenting out the unused function to fix compiler errors.
    // pub async fn get_token_analytics(
    //     &self,
    //     address: &str,
    //     start_time: chrono::DateTime<chrono::Utc>,
    //     end_time: chrono::DateTime<chrono::Utc>,
    // ) -> Result<Vec<TokenAnalyticsData>> {
    //     let filter = doc! {
    //         "address": address,
    //         "timestamp": {
    //             "$gte": bson::DateTime::now(), // Corrected usage
    //             "$lte": bson::DateTime::now()  // Corrected usage
    //         }
    //     };
    //     let find_options = mongodb::options::FindOptions::builder()
    //         .sort(doc! { "timestamp": 1 })
    //         .build();

    //     let mut cursor = self.collection.find(filter, None).await?; // Corrected: No options needed
    //     let mut analytics_data = Vec::new();
    //     while let Some(result) = cursor.try_next().await? {  // Corrected: try_next requires TryStreamExt
    //          let data: TokenAnalyticsData = bson::from_document(result)?;
    //          analytics_data.push(data);
    //     }
    //     Ok(analytics_data)
    // }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAnalyticsData {
    pub symbol: String,
    pub address: String,
    pub price: f64,
    pub timestamp: mongodb::bson::DateTime,
} 
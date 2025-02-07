use rig_mongodb::{MongoDbPool, bson::doc};
use anyhow::Result;
use tracing::info;
use crate::config::mongodb::MongoConfig;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    info!("Running vector store migration...");
    
    let config = MongoConfig::from_env();
    let pool = config.create_pool().await?;
    let db = pool.database(&config.database);

    // Create vector collections with proper schemas for different embedding types
    
    // Market data vectors
    db.create_collection("market_data_vectors", Some(doc! {
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["document", "embedding", "timestamp"],
                "properties": {
                    "document": { "bsonType": "object" },
                    "embedding": { "bsonType": "array" },
                    "metadata": { "bsonType": "object" },
                    "timestamp": { "bsonType": "date" }
                }
            }
        }
    })).await?;

    // Trade history vectors
    db.create_collection("trade_history_vectors", Some(doc! {
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["document", "embedding", "timestamp"],
                "properties": {
                    "document": { "bsonType": "object" },
                    "embedding": { "bsonType": "array" },
                    "metadata": { "bsonType": "object" },
                    "timestamp": { "bsonType": "date" }
                }
            }
        }
    })).await?;

    // Risk model vectors
    db.create_collection("risk_model_vectors", Some(doc! {
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["document", "embedding", "timestamp"],
                "properties": {
                    "document": { "bsonType": "object" },
                    "embedding": { "bsonType": "array" },
                    "metadata": { "bsonType": "object" },
                    "timestamp": { "bsonType": "date" }
                }
            }
        }
    })).await?;

    // Sentiment analysis vectors
    db.create_collection("sentiment_vectors", Some(doc! {
        "validator": {
            "$jsonSchema": {
                "bsonType": "object",
                "required": ["document", "embedding", "timestamp"],
                "properties": {
                    "document": { "bsonType": "object" },
                    "embedding": { "bsonType": "array" },
                    "metadata": { "bsonType": "object" },
                    "timestamp": { "bsonType": "date" }
                }
            }
        }
    })).await?;

    // Create vector search indexes for each collection
    let vector_search_options = doc! {
        "numDimensions": 1536,  // OpenAI embedding dimensions
        "similarity": "cosine"
    };

    // Market data vectors index
    db.run_command(doc! {
        "createSearchIndex": "market_data_vectors",
        "definition": {
            "mappings": {
                "dynamic": true,
                "fields": {
                    "embedding": {
                        "type": "knnVector",
                        "dimensions": 1536,
                        "similarity": "cosine"
                    },
                    "timestamp": { "type": "date" }
                }
            }
        }
    }, None).await?;

    // Trade history vectors index
    db.run_command(doc! {
        "createSearchIndex": "trade_history_vectors",
        "definition": {
            "mappings": {
                "dynamic": true,
                "fields": {
                    "embedding": {
                        "type": "knnVector",
                        "dimensions": 1536,
                        "similarity": "cosine"
                    },
                    "timestamp": { "type": "date" }
                }
            }
        }
    }, None).await?;

    // Risk model vectors index
    db.run_command(doc! {
        "createSearchIndex": "risk_model_vectors",
        "definition": {
            "mappings": {
                "dynamic": true,
                "fields": {
                    "embedding": {
                        "type": "knnVector",
                        "dimensions": 1536,
                        "similarity": "cosine"
                    },
                    "timestamp": { "type": "date" }
                }
            }
        }
    }, None).await?;

    // Sentiment vectors index
    db.run_command(doc! {
        "createSearchIndex": "sentiment_vectors",
        "definition": {
            "mappings": {
                "dynamic": true,
                "fields": {
                    "embedding": {
                        "type": "knnVector",
                        "dimensions": 1536,
                        "similarity": "cosine"
                    },
                    "timestamp": { "type": "date" }
                }
            }
        }
    }, None).await?;

    // Create regular indexes for metadata filtering
    for collection in ["market_data_vectors", "trade_history_vectors", "risk_model_vectors", "sentiment_vectors"].iter() {
        db.collection(collection).create_index(
            doc! {
                "metadata.token_address": 1,
                "timestamp": -1
            },
            None,
        ).await?;
    }

    info!("Vector store migration completed successfully!");
    Ok(())
}
use anyhow::Result;
use mongodb::{
    bson::{doc, Document},
    Client, IndexModel,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    info!("Initializing vector store collections...");

    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let client = Client::with_uri_str(&uri).await?;
    let db = client.database("cainam");

    // Vector collections to create
    let collections = vec![
        "market_data_vectors",
        "trade_history_vectors",
        "risk_model_vectors",
        "sentiment_vectors",
    ];

    for collection_name in collections {
        info!("Creating collection: {}", collection_name);
        
        // Create collection with schema validation
        let command = doc! {
            "create": collection_name,
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
        };

        match db.run_command(command).await {
            Ok(_) => info!("Created collection: {}", collection_name),
            Err(e) if e.to_string().contains("already exists") => {
                info!("Collection {} already exists", collection_name)
            }
            Err(e) => return Err(e.into()),
        }

        // Check if vector search index exists
        let list_indexes_command = doc! {
            "listSearchIndexes": collection_name
        };
        
        let index_exists = match db.run_command(list_indexes_command).await {
            Ok(result) => {
                let indexes = result.get_document("cursor")
                    .and_then(|cursor| cursor.get_array("firstBatch"))
                    .map(|batch| !batch.is_empty())
                    .unwrap_or(false);
                if indexes {
                    info!("Vector search index already exists for: {}", collection_name);
                }
                indexes
            }
            Err(_) => false
        };

        // Create vector search index if it doesn't exist
        if !index_exists {
            info!("Creating vector search index for: {}", collection_name);
            let command = doc! {
                "createSearchIndexes": collection_name,
                "indexes": [{
                    "name": "vector_index",
                    "definition": {
                        "mappings": {
                            "dynamic": true,
                            "fields": {
                                "embedding": {
                                    "type": "knnVector",
                                    "dimensions": 1536,
                                    "similarity": "cosine"
                                }
                            }
                        }
                    }
                }]
            };
            
            match db.run_command(command).await {
                Ok(_) => info!("Created vector index for: {}", collection_name),
                Err(e) => {
                    info!("Failed to create vector index for {}: {}", collection_name, e);
                    return Err(e.into());
                }
            }
        }

        // Create metadata index
        info!("Creating metadata index for: {}", collection_name);
        let index_model = IndexModel::builder()
            .keys(doc! {
                "metadata.token_address": 1,
                "timestamp": -1
            })
            .build();
        
        let collection = db.collection::<Document>(collection_name);
        match collection.create_index(index_model).await {
            Ok(_) => info!("Created metadata index for: {}", collection_name),
            Err(e) if e.to_string().contains("already exists") => {
                info!("Metadata index for {} already exists", collection_name)
            }
            Err(e) => return Err(e.into()),
        }
    }

    info!("Vector store initialization completed successfully!");
    Ok(())
} 
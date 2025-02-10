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

        let collection = db.collection::<Document>(collection_name);

        // Create metadata index
        info!("Creating metadata index for: {}", collection_name);
        let index_model = IndexModel::builder()
            .keys(doc! {
                "metadata.token_address": 1,
                "timestamp": -1
            })
            .build();
        collection.create_index(index_model).await?;

        // Create embedding index
        info!("Creating embedding index for: {}", collection_name);
        let index_model = IndexModel::builder()
            .keys(doc! {
                "embedding": 1
            })
            .build();
        collection.create_index(index_model).await?;
    }

    info!("Vector store initialization completed successfully!");
    Ok(())
} 
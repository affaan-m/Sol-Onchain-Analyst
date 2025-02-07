use anyhow::Result;
use tracing::info;
use crate::config::mongodb::MongoConfig;

mod mongodb {
    pub mod m01_setup;
    pub mod m02_schema;
    pub mod m03_trade_status;
    pub mod m04_allocations;
    pub mod m05_vector_store;
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    info!("Starting migrations...");
    
    // Initialize MongoDB configuration
    let config = MongoConfig::from_env();
    
    // Run MongoDB migrations in order
    info!("Running MongoDB migrations...");
    
    mongodb::m01_setup::run(&config).await?;
    mongodb::m02_schema::run(&config).await?;
    mongodb::m03_trade_status::run(&config).await?;
    mongodb::m04_allocations::run(&config).await?;
    mongodb::m05_vector_store::run(&config).await?;
    
    info!("All migrations completed successfully!");
    Ok(())
}
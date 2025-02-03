use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    println!("Migrations completed successfully!");

    Ok(())
} 
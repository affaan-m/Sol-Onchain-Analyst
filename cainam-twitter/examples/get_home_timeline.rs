use cainam_twitter::error::Result;
use cainam_twitter::scraper::Scraper;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut scraper = Scraper::new().await?;
    
    scraper.login(
        env::var("TWITTER_USERNAME")?,
        env::var("TWITTER_PASSWORD")?,
        Some(env::var("TWITTER_EMAIL")?),
        Some(env::var("TWITTER_2FA_SECRET")?)
    ).await?;
    let home_timeline = scraper.get_home_timeline(20, vec![]).await?;
    println!("Home timeline: {:?}", home_timeline);
    Ok(())
}

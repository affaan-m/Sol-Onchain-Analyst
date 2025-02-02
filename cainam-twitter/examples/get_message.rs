use cainam_twitter::error::Result;
use cainam_twitter::scraper::Scraper;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut scraper = Scraper::new().await?;
    let cookie_string = std::env::var("TWITTER_COOKIE_STRING")
        .expect("TWITTER_COOKIE_STRING environment variable not set");
    let username =
        std::env::var("TWITTER_USERNAME").expect("TWITTER_USERNAME environment variable not set");
    scraper.set_from_cookie_string(&cookie_string).await?;
    let dm_history = scraper
        .get_direct_message_conversations(&username, None)
        .await?;
    println!("Direct message conversations: {:?}", dm_history);
    Ok(())
}

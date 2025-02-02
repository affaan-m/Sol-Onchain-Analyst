use cainam_twitter::error::Result;
use cainam_twitter::scraper::Scraper;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut scraper = Scraper::new().await?;
    let cookie_string = std::env::var("TWITTER_COOKIE_STRING")
        .expect("TWITTER_COOKIE_STRING environment variable not set");
    scraper.set_from_cookie_string(&cookie_string).await?;
    let username =
        std::env::var("TWITTER_USERNAME").expect("TWITTER_USERNAME environment variable not set");
    let dm_history = scraper
        .get_direct_message_conversations(&username, None)
        .await?;

    let user_id = dm_history.user_id.as_str();

    for conversation in dm_history.conversations {
        if let Some(last_message) = conversation.messages.last() {
            let sender_id = last_message.sender_id.as_str();
            // Only reply if the last message was not from us
            if sender_id != user_id {
                let conversation_id = conversation.conversation_id.as_str();
                let text = last_message.text.as_str();
                println!("conversation_id: {}", conversation_id);
                println!("text: {}", text);
                scraper.send_direct_message(conversation_id, "2").await?;
            }
        }
    }

    Ok(())
}

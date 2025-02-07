use crate::api::client::TwitterClient;
use crate::error::{Result, TwitterError};
use reqwest::header::HeaderMap;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    pub id: String,
    pub text: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub created_at: String,
    pub media_urls: Option<Vec<String>>,
    pub sender_screen_name: Option<String>,
    pub recipient_screen_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessageConversation {
    pub conversation_id: String,
    pub messages: Vec<DirectMessage>,
    pub participants: Vec<Participant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub screen_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessagesResponse {
    pub conversations: Vec<DirectMessageConversation>,
    pub users: Vec<TwitterUser>,
    pub cursor: Option<String>,
    pub last_seen_event_id: Option<String>,
    pub trusted_last_seen_event_id: Option<String>,
    pub untrusted_last_seen_event_id: Option<String>,
    pub inbox_timelines: Option<InboxTimelines>,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxTimelines {
    pub trusted: Option<TimelineStatus>,
    pub untrusted: Option<TimelineStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineStatus {
    pub status: String,
    pub min_entry_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterUser {
    pub id: String,
    pub screen_name: String,
    pub name: String,
    pub profile_image_url: String,
    pub description: Option<String>,
    pub verified: Option<bool>,
    pub protected: Option<bool>,
    pub followers_count: Option<i32>,
    pub friends_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessageEvent {
    pub id: String,
    pub type_: String, // Using type_ since 'type' is a keyword in Rust
    pub message_create: MessageCreate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCreate {
    pub sender_id: String,
    pub target: MessageTarget,
    pub message_data: MessageData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTarget {
    pub recipient_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub text: String,
    pub created_at: String,
    pub entities: Option<MessageEntities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEntities {
    pub urls: Option<Vec<UrlEntity>>,
    pub media: Option<Vec<MediaEntity>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlEntity {
    pub url: String,
    pub expanded_url: String,
    pub display_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaEntity {
    pub url: String,
    #[serde(rename = "type")]
    pub media_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendDirectMessageResponse {
    pub entries: Vec<MessageEntry>,
    pub users: std::collections::HashMap<String, TwitterUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEntry {
    pub message: MessageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    pub id: String,
    pub time: String,
    pub affects_sort: bool,
    pub conversation_id: String,
    pub message_data: MessageData,
}

pub async fn get_direct_message_conversations(
    client: &TwitterClient,
    screen_name: &str,
    cursor: Option<&str>,
) -> Result<DirectMessagesResponse> {
    let mut headers = HeaderMap::new();
    client.auth.install_headers(&mut headers).await?;

    let message_list_url = "https://x.com/i/api/1.1/dm/inbox_initial_state.json";
    let url = if let Some(cursor_val) = cursor {
        format!("{}?cursor={}", message_list_url, cursor_val)
    } else {
        message_list_url.to_string()
    };

    let (data, _) = crate::api::requests::request_api::<Value>(
        &client.client,
        &url,
        headers,
        Method::GET,
        None,
    )
    .await?;
    let user_id = crate::profile::get_user_id_by_screen_name(client, screen_name).await?;
    parse_direct_message_conversations(&data, &user_id)
}

pub async fn send_direct_message(
    client: &TwitterClient,
    conversation_id: &str,
    text: &str,
) -> Result<Value> {
    let mut headers = HeaderMap::new();
    client.auth.install_headers(&mut headers).await?;

    let message_dm_url = "https://x.com/i/api/1.1/dm/new2.json";

    let payload = json!({
        "conversation_id": conversation_id,
        "recipient_ids": false,
        "text": text,
        "cards_platform": "Web-12",
        "include_cards": 1,
        "include_quote_count": true,
        "dm_users": false,
    });

    let (response, _) = crate::api::requests::request_api::<Value>(
        &client.client,
        message_dm_url,
        headers,
        Method::POST,
        Some(payload),
    )
    .await?;

    Ok(response)
}

fn parse_direct_message_conversations(
    data: &Value,
    user_id: &str,
) -> Result<DirectMessagesResponse> {
    let inbox_state = data
        .get("inbox_initial_state")
        .ok_or_else(|| TwitterError::Api("Missing inbox_initial_state".into()))?;

    let empty_map = serde_json::Map::new();
    let conversations = inbox_state
        .get("conversations")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty_map);

    let empty_vec = Vec::new();

    let entries = inbox_state
        .get("entries")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_vec);

    let users = inbox_state
        .get("users")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty_map);

    // Parse users first
    let parsed_users = parse_users(users);

    // Group messages by conversation_id
    let messages_by_conversation = group_messages_by_conversation(entries);

    // Convert to DirectMessageConversation array
    let parsed_conversations = conversations
        .iter()
        .map(|(conv_id, conv)| {
            let messages = messages_by_conversation
                .get(conv_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            parse_conversation(conv_id, conv, messages, users)
        })
        .collect();

    Ok(DirectMessagesResponse {
        conversations: parsed_conversations,
        users: parsed_users,
        cursor: inbox_state
            .get("cursor")
            .and_then(|v| v.as_str())
            .map(String::from),
        last_seen_event_id: inbox_state
            .get("last_seen_event_id")
            .and_then(|v| v.as_str())
            .map(String::from),
        trusted_last_seen_event_id: inbox_state
            .get("trusted_last_seen_event_id")
            .and_then(|v| v.as_str())
            .map(String::from),
        untrusted_last_seen_event_id: inbox_state
            .get("untrusted_last_seen_event_id")
            .and_then(|v| v.as_str())
            .map(String::from),
        inbox_timelines: parse_inbox_timelines(inbox_state),
        user_id: user_id.to_string(),
    })
}

fn parse_users(users: &serde_json::Map<String, Value>) -> Vec<TwitterUser> {
    users
        .values()
        .filter_map(|user| {
            Some(TwitterUser {
                id: user.get("id_str")?.as_str()?.to_string(),
                screen_name: user.get("screen_name")?.as_str()?.to_string(),
                name: user.get("name")?.as_str()?.to_string(),
                profile_image_url: user.get("profile_image_url_https")?.as_str()?.to_string(),
                description: user
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                verified: user.get("verified").and_then(|v| v.as_bool()),
                protected: user.get("protected").and_then(|v| v.as_bool()),
                followers_count: user
                    .get("followers_count")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32),
                friends_count: user
                    .get("friends_count")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32),
            })
        })
        .collect()
}

fn group_messages_by_conversation(
    entries: &[Value],
) -> std::collections::HashMap<String, Vec<&Value>> {
    let mut messages_by_conversation: std::collections::HashMap<String, Vec<&Value>> =
        std::collections::HashMap::new();

    for entry in entries {
        if let Some(message) = entry.get("message") {
            if let Some(conv_id) = message.get("conversation_id").and_then(|v| v.as_str()) {
                messages_by_conversation
                    .entry(conv_id.to_string())
                    .or_default()
                    .push(message);
            }
        }
    }

    messages_by_conversation
}

fn parse_conversation(
    conv_id: &str,
    conv: &Value,
    messages: &[&Value],
    users: &serde_json::Map<String, Value>,
) -> DirectMessageConversation {
    let parsed_messages = parse_direct_messages(messages, users);
    let participants = conv
        .get("participants")
        .and_then(|p| p.as_array())
        .map(|parts| {
            parts
                .iter()
                .filter_map(|p| {
                    Some(Participant {
                        id: p.get("user_id")?.as_str()?.to_string(),
                        screen_name: users
                            .get(p.get("user_id")?.as_str()?)
                            .and_then(|u| u.get("screen_name"))
                            .and_then(|s| s.as_str())
                            .unwrap_or(p.get("user_id")?.as_str()?)
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    DirectMessageConversation {
        conversation_id: conv_id.to_string(),
        messages: parsed_messages,
        participants,
    }
}

fn parse_direct_messages(
    messages: &[&Value],
    users: &serde_json::Map<String, Value>,
) -> Vec<DirectMessage> {
    messages
        .iter()
        .filter_map(|msg| {
            let message_data = msg.get("message_data")?;
            Some(DirectMessage {
                id: message_data.get("id")?.as_str()?.to_string(),
                text: message_data.get("text")?.as_str()?.to_string(),
                sender_id: message_data.get("sender_id")?.as_str()?.to_string(),
                recipient_id: message_data.get("recipient_id")?.as_str()?.to_string(),
                created_at: message_data.get("time")?.as_str()?.to_string(),
                media_urls: extract_media_urls(message_data),
                sender_screen_name: users
                    .get(message_data.get("sender_id")?.as_str()?)
                    .and_then(|u| u.get("screen_name"))
                    .and_then(|s| s.as_str())
                    .map(String::from),
                recipient_screen_name: users
                    .get(message_data.get("recipient_id")?.as_str()?)
                    .and_then(|u| u.get("screen_name"))
                    .and_then(|s| s.as_str())
                    .map(String::from),
            })
        })
        .collect()
}

fn extract_media_urls(message_data: &Value) -> Option<Vec<String>> {
    let mut urls = Vec::new();

    if let Some(entities) = message_data.get("entities") {
        // Extract URLs
        if let Some(url_entities) = entities.get("urls").and_then(|u| u.as_array()) {
            for url in url_entities {
                if let Some(expanded_url) = url.get("expanded_url").and_then(|u| u.as_str()) {
                    urls.push(expanded_url.to_string());
                }
            }
        }

        // Extract media URLs
        if let Some(media_entities) = entities.get("media").and_then(|m| m.as_array()) {
            for media in media_entities {
                if let Some(media_url) = media
                    .get("media_url_https")
                    .or_else(|| media.get("media_url"))
                    .and_then(|u| u.as_str())
                {
                    urls.push(media_url.to_string());
                }
            }
        }
    }

    if urls.is_empty() {
        None
    } else {
        Some(urls)
    }
}

fn parse_inbox_timelines(inbox_state: &Value) -> Option<InboxTimelines> {
    inbox_state
        .get("inbox_timelines")
        .map(|timelines| InboxTimelines {
            trusted: parse_timeline_status(timelines.get("trusted")),
            untrusted: parse_timeline_status(timelines.get("untrusted")),
        })
}

fn parse_timeline_status(timeline: Option<&Value>) -> Option<TimelineStatus> {
    timeline.map(|t| TimelineStatus {
        status: t
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
        min_entry_id: t
            .get("min_entry_id")
            .and_then(|m| m.as_str())
            .map(String::from),
    })
}

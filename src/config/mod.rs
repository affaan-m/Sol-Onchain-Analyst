mod agent_config;
mod market_config;
pub mod mongodb;

pub use self::agent_config::AgentConfig;
pub use self::market_config::MarketConfig;
use rig::providers::openai::{GPT_4O, GPT_4O_MINI, O1_MINI, O1_PREVIEW};

pub const DEFAULT_MODEL: &str = "gpt-4o-mini";

pub fn get_openai_model() -> &'static str {
    match std::env::var("OPENAI_MODEL").as_deref() {
        Ok("gpt-4o") => GPT_4O,
        Ok("gpt-4o-mini") => GPT_4O_MINI,
        Ok("o3-mini") => O1_MINI,
        Ok("o1-preview") => O1_PREVIEW,
        _ => DEFAULT_MODEL,
    }
}

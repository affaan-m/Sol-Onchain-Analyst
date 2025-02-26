pub mod agent_config;
pub mod birdeye_config;
pub mod logging_config;
pub mod market_config;
pub mod mongodb;

pub use self::agent_config::AgentConfig;
use rig::providers::openai::{GPT_4O, GPT_4O_MINI, O3_MINI, O1_PREVIEW};
use rig::providers::anthropic::completion::CLAUDE_3_7_SONNET;

pub const DEFAULT_MODEL: &str = GPT_4O_MINI;

pub fn get_openai_model() -> &'static str {
    match std::env::var("OPENAI_MODEL").as_deref() {
        Ok("gpt-4o") => GPT_4O,
        Ok("gpt-4o-mini") => GPT_4O_MINI,
        Ok("o3-mini") => O3_MINI,
        Ok("o1-preview") => O1_PREVIEW,
        _ => DEFAULT_MODEL,
    }
}

pub fn get_anthropic_model() -> &'static str {
    match std::env::var("ANTHROPIC_MODEL").as_deref() {
        Ok("claude-3-7-sonnet") => CLAUDE_3_7_SONNET,
        _ => CLAUDE_3_7_SONNET, // Default to latest model
    }
}

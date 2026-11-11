use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub username: String,
    pub clients: Vec<String>,
    pub model_provider: String,
    pub image_model_provider: String,
    pub plugins: Vec<String>,
    pub settings: Settings,
    pub system: String,
    pub bio: Vec<String>,
    pub lore: Vec<String>,
    pub knowledge: Vec<String>,
    pub message_examples: Vec<Vec<MessageExample>>,
    pub post_examples: Vec<String>,
    pub topics: Vec<String>,
    pub style: Style,
    pub adjectives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub secrets: HashMap<String, String>,
    pub voice: VoiceSettings,
    pub rag_knowledge: bool,
    pub model_config: ModelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSettings {
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub temperature: f32,
    pub max_tokens: u32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub top_p: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageExample {
    pub user: String,
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    pub text: String,
    pub action: Option<String>,
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    pub tone: String,
    pub writing: String,
    pub personality: String,
    pub quirks: Vec<String>,
    pub all: Vec<String>,
    pub chat: Vec<String>,
    pub post: Vec<String>,
}

impl Character {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let character = serde_json::from_str(&content)?;
        Ok(character)
    }

    pub fn get_system_prompt(&self) -> String {
        let mut prompt = String::new();
        
        // Add system description
        prompt.push_str(&self.system);
        prompt.push_str("\n\n");

        // Add style guidelines
        prompt.push_str("Style Guidelines:\n");
        for guideline in &self.style.all {
            prompt.push_str(&format!("- {}\n", guideline));
        }
        prompt.push_str("\n");

        // Add knowledge base summary
        prompt.push_str("Knowledge Base:\n");
        for knowledge in &self.knowledge {
            prompt.push_str(&format!("- {}\n", knowledge));
        }

        prompt
    }

    pub fn get_post_style(&self) -> Vec<String> {
        self.style.post.clone()
    }

    pub fn get_chat_style(&self) -> Vec<String> {
        self.style.chat.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_deserialization() {
        let json = r#"{
            "name": "Vergen",
            "username": "vergen",
            "clients": ["direct", "discord", "telegram", "twitter"],
            "modelProvider": "anthropic",
            "imageModelProvider": "openai",
            "plugins": [],
            "settings": {
                "secrets": {},
                "voice": {
                    "model": "en_US-hfc_male-medium"
                },
                "ragKnowledge": true,
                "modelConfig": {
                    "temperature": 0.7,
                    "maxTokens": 2048,
                    "frequencyPenalty": 0.0,
                    "presencePenalty": 0.0,
                    "topP": 0.95
                }
            },
            "system": "Test system prompt",
            "bio": ["Test bio"],
            "lore": ["Test lore"],
            "knowledge": ["Test knowledge"],
            "messageExamples": [],
            "postExamples": [],
            "topics": ["Test topic"],
            "style": {
                "tone": "professional",
                "writing": "clear",
                "personality": "confident",
                "quirks": ["test quirk"],
                "all": ["test guideline"],
                "chat": ["test chat style"],
                "post": ["test post style"]
            },
            "adjectives": ["analytical"]
        }"#;

        let character: Character = serde_json::from_str(json).unwrap();
        assert_eq!(character.name, "Vergen");
        assert_eq!(character.username, "vergen");
    }
} 
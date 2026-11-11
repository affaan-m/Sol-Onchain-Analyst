use rig::agent::Agent as RigAgent;
use rig::providers::openai::{Client as OpenAIClient, CompletionModel, GPT_4_TURBO};
use rig::{completion::Prompt, providers};
use anyhow::Result;

pub struct Agent {
    agent: RigAgent<CompletionModel>,
}

impl Agent {
    pub fn new(openai_api_key: &str, prompt: &str) -> Self {
        let openai_client = OpenAIClient::new(openai_api_key);
        let agent = openai_client
            .agent(GPT_4_TURBO)
            .preamble(prompt)
            .temperature(1.0)
            .build();

        Agent { agent }
    }

    pub async fn prompt(&self, input: &str) -> Result<String> {
        let response = self.agent.prompt(input).await?;
        Ok(response)
    }
}

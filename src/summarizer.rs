use crate::llm::{ChatMessage, LLMClient};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;
use tiktoken_rs::cl100k_base;

#[derive(Clone)]
pub struct Summarizer {
    pub llm_client: LLMClient,
    pub output_language: String,
    pub detail_level: String,
}

pub fn split_text_by_tokens(text: &str, max_tokens: usize) -> Vec<String> {
    // Initialize the BPE encoder to count tokens
    let bpe = cl100k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(text);

    let mut sections = Vec::new();
    let mut start = 0;

    // Iterate over the tokens, creating sections that do not exceed the maximum number of tokens
    while start < tokens.len() {
        let end = usize::min(start + max_tokens, tokens.len());
        let section_tokens = &tokens[start..end];
        let section_text = bpe.decode(section_tokens.to_vec()).unwrap();
        sections.push(section_text);
        start = end;
    }

    sections
}

impl Summarizer {
    pub fn new(
        api_key: String,
        model_name: String,
        output_language: String,
        detail_level: String,
    ) -> Self {
        Summarizer {
            llm_client: LLMClient::new(api_key, model_name),
            output_language,
            detail_level,
        }
    }

    pub async fn generate_summary_plan(&self, toc: &[String]) -> Result<String> {
        let prompt_template = fs::read_to_string("prompts/summary_plan.md")?;

        let toc_text = toc.join("\n");

        let prompt = prompt_template
            .replace("{{language}}", &self.output_language)
            .replace("{{toc}}", &toc_text);

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let response = self.llm_client.send_request(messages, 0.7).await?;

        // Log the response to a file
        self.log_llm_response(&response, "summary_plan")?;

        if response.trim().is_empty() {
            return Err(anyhow!("LLM returned an empty response."));
        }

        Ok(response)
    }

    pub async fn summarize_with_plan(&self, text: &str, plan: &str) -> Result<Value> {
        let prompt_template = fs::read_to_string("prompts/detailed_summary.md")?;

        let prompt = prompt_template
            .replace("{{language}}", &self.output_language)
            .replace("{{detail_level}}", &self.detail_level)
            .replace("{{plan}}", plan)
            .replace("{{text}}", text);

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let response = self.llm_client.send_request(messages, 0.7).await?;

        // Log the response to a file
        self.log_llm_response(&response, "detailed_summary")?;

        // Check if the response is empty
        if response.trim().is_empty() {
            return Err(anyhow!("LLM returned an empty response."));
        }

        // Validate if the JSON is valid
        match serde_json::from_str::<Value>(&response) {
            Ok(parsed_response) => Ok(parsed_response),
            Err(_) => {
                self.log_llm_response(&response, "invalid_json")?;
                Err(anyhow!(
                    "Failed to parse JSON response from LLM: {}",
                    response
                ))
            }
        }
    }

    // Helper function to log LLM responses to a file
    fn log_llm_response(&self, response: &str, context: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("llm_responses.log")?;

        writeln!(
            file,
            "[{}] Context: {}\nResponse:\n{}\n",
            Utc::now().to_rfc3339(),
            context,
            response
        )?;

        Ok(())
    }
}

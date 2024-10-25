use crate::llm::{ChatMessage, LLMClient};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tiktoken_rs::cl100k_base;

#[derive(Clone)]
pub struct Summarizer {
    pub llm_client: LLMClient,
    pub output_language: String,
    pub detail_level: String,
    pub log_dir: PathBuf, // Directory for logs
}

impl Summarizer {
    pub fn new(
        api_key: String,
        model_name: String,
        output_language: String,
        detail_level: String,
    ) -> Self {
        let log_dir = PathBuf::from("logs"); // Create log directory
        fs::create_dir_all(&log_dir).expect("Failed to create log directory");

        Summarizer {
            llm_client: LLMClient::new(api_key, model_name),
            output_language,
            detail_level,
            log_dir,
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

        // Log raw response
        self.log_llm_response(&response, "summary_plan", "received")
            .await?;

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

        // Log raw response
        self.log_llm_response(&response, "detailed_summary", "received")
            .await?;

        // Clean up markdown and other unwanted characters from the LLM response
        let cleaned_response = self.clean_response(&response);

        // Stop execution if the response is empty
        if cleaned_response.trim().is_empty() {
            return Err(anyhow!("LLM returned an empty response."));
        }

        // Try to parse the JSON and stop the program if parsing fails
        match serde_json::from_str::<Value>(&cleaned_response) {
            Ok(parsed_response) => {
                // Log successful transformation
                self.log_llm_response(&cleaned_response, "detailed_summary", "parsed")
                    .await?;
                Ok(parsed_response)
            }
            Err(e) => {
                // Log the invalid JSON response
                self.log_llm_response(&cleaned_response, "detailed_summary", "invalid_json")
                    .await?;
                println!(
                    "Critical error parsing LLM response: {}\nResponse: {}",
                    e, cleaned_response
                );
                std::process::exit(1); // Stop the program immediately
            }
        }
    }

    // Log LLM responses in log files under the logs directory
    async fn log_llm_response(&self, response: &str, context: &str, status: &str) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339();
        let log_file_path = self.log_dir.join(format!("llm_{}.log", context));

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)?;

        writeln!(
            file,
            "[{}] Context: {}\nStatus: {}\nResponse:\n{}\n",
            timestamp, context, status, response
        )?;

        Ok(())
    }

    // Clean response from unwanted characters like backticks or JSON markdown
    fn clean_response(&self, response: &str) -> String {
        response
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .to_string()
    }

    // Function to split text into sections based on token count
    pub fn split_text_by_tokens(&self, text: &str, max_tokens: usize) -> Vec<String> {
        let bpe = cl100k_base().unwrap();
        let tokens = bpe.encode_with_special_tokens(text);

        let mut sections = Vec::new();
        let mut start = 0;

        while start < tokens.len() {
            let end = usize::min(start + max_tokens, tokens.len());
            let section_tokens = &tokens[start..end];
            let section_text = bpe.decode(section_tokens.to_vec()).unwrap();
            sections.push(section_text);
            start = end;
        }

        sections
    }
}

use anyhow::Result;
use log::error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;

#[derive(Clone)]
pub struct LLMClient {
    client: Arc<reqwest::Client>,
    pub api_key: String,
    pub model_name: String,
}

impl LLMClient {
    pub fn new(api_key: String, model_name: String) -> Self {
        LLMClient {
            client: Arc::new(reqwest::Client::new()),
            api_key,
            model_name,
        }
    }

    pub async fn send_request(
        &self,
        messages: Vec<ChatMessage>,
        temperature: f32,
    ) -> Result<String> {
        let request_body = OpenRouterRequest {
            model: self.model_name.clone(),
            messages,
            temperature,
        };

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .headers(self.build_headers()?)
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            match serde_json::from_str::<OpenRouterResponse>(&response_text) {
                Ok(response_body) => {
                    if let Some(choice) = response_body.choices.first() {
                        Ok(choice.message.content.clone())
                    } else {
                        Err(anyhow::anyhow!("No response received from LLM"))
                    }
                }
                Err(e) => {
                    error!(
                        "Error deserializing response: {}\nResponse Text: {}",
                        e, response_text
                    );
                    Err(anyhow::anyhow!("Error deserializing response body"))
                }
            }
        } else {
            // Log the response body for debugging
            error!("API returned error status {}: {}", status, response_text);

            Err(anyhow::anyhow!(
                "Request error: {} - {}",
                status,
                response_text
            ))
        }
    }

    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        // Optional headers as per OpenRouter documentation
        headers.insert(
            "X-Title",
            HeaderValue::from_static("AIBook Summarizer"), // Replace with your app name
        );
        headers.insert(
            "HTTP-Referer",
            HeaderValue::from_static("https://github.com/felipepimentel/aibook"), // Replace with your site URL
        );
        Ok(headers)
    }
}

#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

#[derive(Deserialize, Debug)]
struct Message {
    #[allow(dead_code)]
    role: String,
    content: String,
}

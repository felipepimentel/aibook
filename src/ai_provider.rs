use eyre::Result;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Serialize)]
struct SummarizeRequest {
    chapter: String,
    language: String,
}

#[derive(Deserialize)]
struct SummarizeResponse {
    summary: String,
}

fn create_client() -> Result<ClientWithMiddleware> {
    let max_retries = env::var("MAX_RETRIES")
        .unwrap_or_else(|_| "3".to_string())
        .parse()
        .unwrap_or(3);
    let max_elapsed_time_secs = env::var("MAX_ELAPSED_TIME_SECS")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);

    // Criar uma política de retry personalizada
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(max_retries);

    // Criar o cliente com um timeout global
    let client = Client::builder()
        .timeout(Duration::from_secs(max_elapsed_time_secs))
        .build()?;

    // Construir o cliente com middleware
    let client = ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    Ok(client)
}

pub async fn summarize_with_stackspot(api_key: &str, chapter: &str, lang: &str) -> Result<String> {
    let client = create_client()?;
    let request_body = SummarizeRequest {
        chapter: chapter.to_string(),
        language: lang.to_string(),
    };

    let response = client
        .post("https://ai.stackspot.com/api/summarize")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_body)?)
        .send()
        .await?;

    let summary: SummarizeResponse = response.json().await?;
    Ok(summary.summary)
}

pub async fn summarize_with_openrouter(api_key: &str, chapter: &str, lang: &str) -> Result<String> {
    let client = create_client()?;
    let request_body = SummarizeRequest {
        chapter: chapter.to_string(),
        language: lang.to_string(),
    };

    let response = client
        .post("https://openrouter.ai/api/summarize")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_body)?)
        .send()
        .await?;

    let summary: SummarizeResponse = response.json().await?;
    Ok(summary.summary)
}

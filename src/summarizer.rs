use crate::llm::{ChatMessage, LLMClient};
use anyhow::Result;
use tiktoken_rs::cl100k_base;

#[derive(Clone)]
pub struct Summarizer {
    pub llm_client: LLMClient,
    pub output_language: String,
    pub detail_level: String,
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
        let toc_text = toc.join("\n");
        let prompt = format!(
            "You are an expert in summarizing content. Based on the following table of contents, create a detailed summary plan for this e-book. Focus on the main content and key concepts, and exclude any sections related to the author biography, acknowledgments, prefaces, or any meta-information. Use a direct, note-taking style in {}.\n\nTable of Contents:\n{}",
            self.output_language, toc_text
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        self.llm_client.send_request(messages, 0.7).await
    }

    pub async fn summarize_with_plan(
        &self,
        text: &str,
        plan: &str,
    ) -> Result<(String, Vec<String>, Vec<String>, Vec<String>)> {
        let prompt = format!(
            "Using the following summary plan, summarize the text below. Focus on key points, important insights, and technical terms, using a direct, note-taking style. Avoid phrases like 'the text discusses' or 'this chapter explains'. Do not include sections such as 'About the Author' or any meta-information. The summary should be in {}, and the level of detail should be {}.\n\nSummary Plan:\n{}\n\nText:\n{}",
            self.output_language, self.detail_level, plan, text
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let response = self.llm_client.send_request(messages, 0.7).await?;

        // Parse the response to extract summary and keywords
        let (summary, keywords, _, _) = parse_response(&response);
        Ok((summary, keywords, Vec::new(), Vec::new()))
    }
}

// Helper functions
pub fn split_text_by_tokens(text: &str, max_tokens: usize) -> Vec<String> {
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

pub fn parse_response(content: &str) -> (String, Vec<String>, Vec<String>, Vec<String>) {
    // Split sections by known markers
    let sections: Vec<&str> = content.split("\n\n").collect();
    let mut summary = String::new();
    let mut keywords = Vec::new();
    // We will skip glossary and references for now

    for section in sections {
        if section.starts_with("Keywords:") {
            let words = section
                .trim_start_matches("Keywords:")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();
            keywords.extend(words);
        } else if section.starts_with("Glossary:")
            || section.starts_with("References:")
            || section.starts_with("About the Author")
            || section.starts_with("Author Biography")
            || section.starts_with("Preface")
            || section.starts_with("Acknowledgments")
        {
            // Skip unwanted sections
            continue;
        } else {
            // Add to summary
            summary.push_str(section);
            summary.push('\n');
        }
    }

    (
        summary.trim().to_string(),
        keywords,
        Vec::new(), // Empty glossary
        Vec::new(), // Empty references
    )
}

use clap::Parser;
use dotenv::dotenv;
use env_logger::Env;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;

mod ebook;
mod llm;
mod summarizer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path(s) to the EPUB file(s)
    #[arg(short, long)]
    input: Vec<PathBuf>,

    /// Output directory
    #[arg(short, long)]
    output_dir: Option<PathBuf>,

    /// API key for OpenRouter (optional, can use environment variable)
    #[arg(short, long)]
    api_key: Option<String>,

    /// Model to be used (optional, can use environment variable)
    #[arg(long)]
    model: Option<String>,

    /// Output language (optional, can use environment variable)
    #[arg(long)]
    language: Option<String>,

    /// Summary detail level (short, medium, long)
    #[arg(long, default_value = "medium")]
    detail_level: String,

    /// Output format (markdown, html)
    #[arg(long, default_value = "markdown")]
    output_format: String,

    /// Verbosity level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let args = Args::parse();

    // Logging configuration
    let log_level = match args.verbose {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    // Get the API key from the argument or environment variable
    let api_key = args
        .api_key
        .or_else(|| env::var("OPENROUTER_API_KEY").ok())
        .expect("API key not provided");

    // Get the model name from the argument or environment variable
    let model_name = args
        .model
        .or_else(|| env::var("MODEL_NAME").ok())
        .unwrap_or_else(|| "openai/gpt-4o-mini".to_string());

    // Get the output language from the argument or environment variable
    let output_language = args
        .language
        .or_else(|| env::var("OUTPUT_LANGUAGE").ok())
        .unwrap_or_else(|| "en".to_string());

    // Get the output directory from the argument or environment variable
    let default_output_dir = env::var("OUTPUT_DIR").unwrap_or_else(|_| "output".to_string());

    // Process multiple e-books
    for input_path in &args.input {
        // Determine the output directory for each e-book
        let output_dir = match &args.output_dir {
            Some(path) => path.clone(),
            None => PathBuf::from(&default_output_dir),
        };
        // Create a unique directory for each e-book based on the file name
        let ebook_stem = input_path
            .file_stem()
            .unwrap_or_else(|| input_path.as_os_str())
            .to_string_lossy();
        let ebook_output_dir = output_dir.join(ebook_stem.to_string());

        // Create the output directory and the images subdirectory
        fs::create_dir_all(&ebook_output_dir)?;
        let images_dir = ebook_output_dir.join("images");
        fs::create_dir_all(&images_dir)?;

        // Read the e-book and get chapters and images
        let (doc, chapters, _chapters_images) = ebook::read_ebook(&input_path, &images_dir)?;
        info!("E-book '{}' read successfully.", input_path.display());

        // Extract the table of contents
        let toc = ebook::extract_table_of_contents(&doc);

        // Initialize the summarizer client
        let summarizer = summarizer::Summarizer::new(
            api_key.clone(),
            model_name.clone(),
            output_language.clone(),
            args.detail_level.clone(),
        );

        // Generate the summary plan
        println!("Generating summary plan...");
        let plan = summarizer.generate_summary_plan(&toc).await?;

        // Split the plan into sections (if applicable)
        let plan_sections: Vec<String> = plan
            .split("##")
            .skip(1)
            .map(|s| format!("##{}", s.trim()))
            .collect();

        // Progress bar
        let pb = ProgressBar::new(chapters.len() as u64);
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-");
        pb.set_style(style);

        // Vector to store results
        let mut results: Vec<Value> = Vec::new();

        for (index, chapter) in chapters.into_iter().enumerate() {
            let summarizer = summarizer.clone();
            let pb = pb.clone();
            let chapter_plan = plan_sections.get(index).cloned().unwrap_or_default();
            let sections = summarizer::split_text_by_tokens(&chapter, 2000);

            let mut chapter_result = Value::Null;

            for section in sections {
                let section = section.to_string(); // Convert &str to String
                let mut attempt = 0;
                let max_attempts = 3; // Try up to 3 times

                while attempt < max_attempts {
                    match summarizer
                        .summarize_with_plan(&section, &chapter_plan)
                        .await
                    {
                        Ok(result) => {
                            chapter_result = result;
                            break; // Exit the loop if the response is successful
                        }
                        Err(e) => {
                            error!("Error summarizing a section: {}", e);
                            attempt += 1;

                            if attempt == max_attempts {
                                error!("Maximum number of attempts reached. Skipping the section.");
                            }
                        }
                    }
                }
            }

            pb.inc(1);
            results.push(chapter_result);
        }

        pb.finish_with_message("Summarization completed!");

        // Build the final summary
        let mut final_summary = String::new();

        for result in results {
            if let Some(summary) = result.get("summary").and_then(Value::as_str) {
                final_summary.push_str(summary);
                final_summary.push('\n');
            }

            if let Some(keywords) = result.get("keywords").and_then(Value::as_array) {
                final_summary.push_str("\n## Keywords\n");
                for keyword in keywords {
                    final_summary.push_str(&format!("- {}\n", keyword));
                }
            }

            if let Some(glossary) = result.get("glossary").and_then(Value::as_array) {
                final_summary.push_str("\n## Glossary\n");
                for term in glossary {
                    final_summary.push_str(&format!("- {}\n", term));
                }
            }

            if let Some(references) = result.get("references").and_then(Value::as_array) {
                final_summary.push_str("\n## References\n");
                for reference in references {
                    final_summary.push_str(&format!("- {}\n", reference));
                }
            }

            if let Some(resources) = result.get("additional_resources").and_then(Value::as_array) {
                final_summary.push_str("\n## Additional Resources\n");
                for resource in resources {
                    final_summary.push_str(&format!("- {}\n", resource));
                }
            }
        }

        // Path to the summary file
        let summary_path = ebook_output_dir.join(format!("summary.{}", args.output_format));

        // Save the final summary
        fs::write(&summary_path, final_summary)?;
        println!("Summary successfully saved to {}", summary_path.display());
    }

    Ok(())
}

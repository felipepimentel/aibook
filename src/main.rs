use clap::Parser;
use dotenv::dotenv;
use env_logger::Env;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use std::env;
use std::fs;
use std::path::PathBuf;

mod ebook;
mod llm;
mod output;
mod summarizer;

/// Command-line arguments structure
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

    /// Detail level of the summary (short, medium, long)
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

    // Configure logging
    let log_level = match args.verbose {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    // Get the API key from environment variable or argument
    let api_key = args
        .api_key
        .or_else(|| env::var("OPENROUTER_API_KEY").ok())
        .expect("API key not provided");

    // Get the model name from environment variable or argument
    let model_name = args
        .model
        .or_else(|| env::var("MODEL_NAME").ok())
        .unwrap_or_else(|| "openai/gpt-3.5-turbo".to_string());

    // Get the output language from environment variable or argument
    let output_language = args
        .language
        .or_else(|| env::var("OUTPUT_LANGUAGE").ok())
        .unwrap_or_else(|| "en".to_string());

    // Process multiple e-books
    for input_path in &args.input {
        // Determine the output directory for each e-book
        let output_dir = match &args.output_dir {
            Some(path) => path.clone(),
            None => {
                let default_output = PathBuf::from("output");
                default_output
            }
        };
        // Create a unique directory for each e-book based on its filename
        let ebook_stem = input_path
            .file_stem()
            .unwrap_or_else(|| input_path.as_os_str())
            .to_string_lossy();
        let ebook_output_dir = output_dir.join(ebook_stem.to_string());

        // Create the output directory and images subdirectory
        fs::create_dir_all(&ebook_output_dir)?;
        let images_dir = ebook_output_dir.join("images");
        fs::create_dir_all(&images_dir)?;

        // Read the e-book and get chapters
        let (doc, chapters) = ebook::read_ebook(&input_path, &images_dir)?;
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

        // Clone output_format for use inside tasks
        let output_format = args.output_format.clone();

        // Vector to store tasks
        let mut tasks = Vec::new();

        for (index, chapter) in chapters.into_iter().enumerate() {
            let summarizer = summarizer.clone();
            let pb = pb.clone();
            let chapter_number = index + 1;
            let chapter_title = toc
                .get(index)
                .unwrap_or(&format!("Chapter {}", chapter_number))
                .clone();
            let chapter_plan = plan_sections.get(index).cloned().unwrap_or_default();

            // Split the chapter into subsections if necessary
            let sections = summarizer::split_text_by_tokens(&chapter, 2000);

            let output_format_clone = output_format.clone();

            // Process each subsection in parallel
            let task = tokio::spawn(async move {
                let mut chapter_summary = Vec::new();
                let mut chapter_glossary = Vec::new();
                let mut chapter_references = Vec::new();
                for section in sections {
                    match summarizer
                        .summarize_with_plan(&section, &chapter_plan)
                        .await
                    {
                        Ok((summary, keywords, glossary, references)) => {
                            // Highlight keywords
                            let highlighted_summary =
                                output::highlight_keywords(&summary, &keywords);
                            chapter_summary.push(highlighted_summary);
                            chapter_glossary.extend(glossary);
                            chapter_references.extend(references);
                        }
                        Err(e) => error!("Error summarizing a section: {}", e),
                    }
                }
                pb.inc(1);
                let combined_summary = chapter_summary.join("\n\n");

                // Create the section in the specified format
                let section_content =
                    output::format_section(&chapter_title, &combined_summary, &output_format_clone);
                Ok::<_, anyhow::Error>((section_content, chapter_glossary, chapter_references))
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;

        pb.finish_with_message("Summarization completed!");

        // Build the final summary
        let mut final_summary = output::format_title("E-book Summary", &output_format);
        let mut final_glossary = Vec::new();
        let mut final_references = Vec::new();

        for result in results {
            match result {
                Ok(Ok((content, glossary, references))) => {
                    final_summary.push_str(&content);
                    final_summary.push('\n');
                    final_glossary.extend(glossary);
                    final_references.extend(references);
                }
                Ok(Err(e)) => error!("Task error: {}", e),
                Err(e) => error!("Error awaiting task: {}", e),
            }
        }

        // Add glossary and references at the end
        if !final_glossary.is_empty() {
            let glossary_content = output::format_glossary(&final_glossary, &output_format);
            final_summary.push_str(&glossary_content);
        }

        if !final_references.is_empty() {
            let references_content = output::format_references(&final_references, &output_format);
            final_summary.push_str(&references_content);
        }

        // Path to the summary file
        let summary_path = ebook_output_dir.join(format!("summary.{}", output_format));

        // Save the final summary
        fs::write(&summary_path, &final_summary)?;
        println!("Summary saved successfully at {}", summary_path.display());
    }

    Ok(())
}

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

    // Get the API key from argument or environment variable
    let api_key = args
        .api_key
        .or_else(|| env::var("OPENROUTER_API_KEY").ok())
        .expect("API key not provided");

    // Get the model name from argument or environment variable
    let model_name = args
        .model
        .or_else(|| env::var("MODEL_NAME").ok())
        .unwrap_or_else(|| "openai/gpt-4o-mini".to_string());

    // Get the output language from argument or environment variable
    let output_language = args
        .language
        .or_else(|| env::var("OUTPUT_LANGUAGE").ok())
        .unwrap_or_else(|| "en".to_string());

    // Get the output directory from argument or environment variable
    let default_output_dir = env::var("OUTPUT_DIR").unwrap_or_else(|_| "output".to_string());

    // Process multiple e-books
    for input_path in &args.input {
        // Determine the output directory for each e-book
        let output_dir = match &args.output_dir {
            Some(path) => path.clone(),
            None => PathBuf::from(&default_output_dir),
        };
        let ebook_stem = input_path
            .file_stem()
            .unwrap_or_else(|| input_path.as_os_str())
            .to_string_lossy();
        let ebook_output_dir = output_dir.join(ebook_stem.to_string());

        fs::create_dir_all(&ebook_output_dir)?;
        let images_dir = ebook_output_dir.join("images");
        fs::create_dir_all(&images_dir)?;

        // Update the read_ebook function call to match the new return type
        let (doc, chapters, _chapters_images, _metadata) =
            ebook::read_ebook(&input_path, &images_dir)?;

        info!("E-book '{}' successfully read.", input_path.display());

        let toc = ebook::extract_table_of_contents(&doc);

        let summarizer = summarizer::Summarizer::new(
            api_key.clone(),
            model_name.clone(),
            output_language.clone(),
            args.detail_level.clone(),
        );

        println!("Generating summary plan...");
        let plan = summarizer.generate_summary_plan(&toc).await?;

        let plan_sections: Vec<String> = plan
            .split("##")
            .skip(1)
            .map(|s| format!("##{}", s.trim()))
            .collect();

        let pb = ProgressBar::new(chapters.len() as u64); // Use total number of chapters
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-");
        pb.set_style(style);

        // Iterate through chapters
        for (index, chapter) in chapters.iter().enumerate() {
            let chapter_plan = plan_sections.get(index).cloned().unwrap_or_default();

            // Split chapter into sections based on token limit
            let sections = summarizer.split_text_by_tokens(chapter, 2000);

            // Process each section of the chapter
            for section in sections {
                let result = summarizer
                    .summarize_with_plan(&section, &chapter_plan)
                    .await;

                match result {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Error summarizing section: {}", e);
                        pb.finish_with_message("Summarization failed. Check logs for details.");
                        return Err(e.into());
                    }
                }
            }

            // Increment progress bar only after finishing all sections of the chapter
            pb.inc(1);
        }

        pb.finish_with_message("Summarization completed successfully!");
    }

    info!("Summarization completed for {} e-books", args.input.len());
    println!("Summarization completed for {} e-books", args.input.len());

    Ok(())
}

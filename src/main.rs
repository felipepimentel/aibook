mod ai_provider;
mod cli;
mod epub_handler;

use clap::Parser;
use cli::{AIProvider, Cli, Commands, Language};
use dotenv::dotenv;
use eyre::{Result, WrapErr};
use indicatif::{ProgressBar, ProgressStyle};
use log::warn;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct ProcessingState {
    images_extracted: bool,
    text_extracted: bool,
    chapters_processed: Vec<usize>,
    epub_created: bool,
}

impl ProcessingState {
    fn new() -> Self {
        ProcessingState {
            images_extracted: false,
            text_extracted: false,
            chapters_processed: Vec::new(),
            epub_created: false,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let cli = Cli::parse();

    let default_lang = env::var("DEFAULT_LANGUAGE").unwrap_or_else(|_| {
        warn!("DEFAULT_LANGUAGE environment variable is not set. Falling back to default 'pt'.");
        "pt".to_string()
    });
    let default_ai_provider = env::var("AI_PROVIDER").unwrap_or_else(|_| "openrouter".to_string());

    match cli.command {
        Commands::Process { file, lang } => {
            validate_file_path(&file)?;

            let api_key = env::var("API_KEY").wrap_err("Failed to retrieve API_KEY from .env")?;

            let lang_code = get_lang_code(lang, &default_lang);
            let ai_provider = get_ai_provider(None, &default_ai_provider);

            let book_name = Path::new(&file)
                .file_stem()
                .ok_or_else(|| eyre::eyre!("Failed to extract book name from file path"))?
                .to_string_lossy()
                .to_string();
            let sanitized_book_name = sanitize_filename::sanitize(&book_name);

            if sanitized_book_name.is_empty() {
                return Err(eyre::eyre!(
                    "Sanitized book name is empty or malformed. Please provide a valid file name."
                ));
            }

            let output_dir = Path::new(&sanitized_book_name);
            std::fs::create_dir_all(&output_dir)?;

            let state_file = output_dir.join("processing_state.json");
            let mut state = if state_file.exists() {
                let state_json = fs::read_to_string(&state_file)?;
                serde_json::from_str(&state_json)?
            } else {
                ProcessingState::new()
            };

            if !state.images_extracted {
                println!("Extracting images...");
                std::fs::create_dir_all(&output_dir.join("images"))?;
                epub_handler::extract_images_from_epub(
                    &file,
                    output_dir.join("images").to_str().unwrap(),
                )?;
                state.images_extracted = true;
                save_state(&state, &state_file)?;
            }

            let chapters = if !state.text_extracted {
                println!("Extracting text...");
                let chapters = epub_handler::extract_text_from_epub(&file)?;
                if chapters.is_empty() {
                    println!("No chapters found in the EPUB.");
                    return Ok(());
                }
                state.text_extracted = true;
                save_state(&state, &state_file)?;
                chapters
            } else {
                epub_handler::extract_text_from_epub(&file)?
            };

            let total_chapters = chapters.len();
            let pb = ProgressBar::new(total_chapters as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%) {msg}",
                )?
                .progress_chars("##-"),
            );

            let md_path = output_dir.join("summary.md");
            let mut md_file = if state.chapters_processed.is_empty() {
                File::create(&md_path)?
            } else {
                fs::OpenOptions::new().append(true).open(&md_path)?
            };

            let chapter_limit = env::var("CHAPTER_LIMIT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10);

            for (i, chapter) in chapters.iter().enumerate() {
                if state.chapters_processed.contains(&i) {
                    pb.inc(1);
                    continue;
                }
                pb.set_message(format!("Processing chapter {}", i + 1));

                let summary = match ai_provider {
                    "stackspot" => {
                        ai_provider::summarize_with_stackspot(&api_key, chapter, &lang_code).await?
                    }
                    _ => {
                        ai_provider::summarize_with_openrouter(&api_key, chapter, &lang_code)
                            .await?
                    }
                };
                writeln!(
                    md_file,
                    "# Chapter {}

{}",
                    i + 1,
                    summary
                )?;
                state.chapters_processed.push(i);
                save_state(&state, &state_file)?;
                pb.inc(1);

                if i + 1 >= chapter_limit {
                    break;
                }
            }

            pb.finish_with_message("Processing complete");

            if !state.epub_created {
                println!("Creating EPUB...");
                epub_handler::create_epub(&output_dir, &md_path)?;
                state.epub_created = true;
                save_state(&state, &state_file)?;
            }

            println!("Pocket book created at {:?}", output_dir);
        }
    }

    Ok(())
}

fn save_state(state: &ProcessingState, state_file: &Path) -> Result<()> {
    let state_json = serde_json::to_string(state)?;
    fs::write(state_file, state_json)?;
    Ok(())
}

fn validate_file_path(file: &str) -> Result<()> {
    let path = Path::new(file);
    if !path.exists() || !path.is_file() {
        return Err(eyre::eyre!(
            "The provided file path does not exist or is invalid: {}",
            file
        ));
    }
    Ok(())
}

fn get_lang_code(lang: Option<Language>, default: &str) -> &str {
    match lang {
        Some(Language::PtBr) => "pt",
        Some(Language::En) => "en",
        None => default,
    }
}

fn get_ai_provider(provider: Option<AIProvider>, default: &str) -> &str {
    match provider {
        Some(AIProvider::StackSpot) => "stackspot",
        Some(AIProvider::OpenRouter) => "openrouter",
        None => default,
    }
}

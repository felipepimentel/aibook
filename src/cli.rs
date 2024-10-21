use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "aibook")]
#[command(about = "A CLI tool for generating pocket books from EPUB files", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Process {
        #[arg(short, long)]
        file: String,
        #[arg(short, long, value_enum)]
        lang: Option<Language>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Language {
    PtBr,
    En,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum AIProvider {
    StackSpot,
    OpenRouter,
}

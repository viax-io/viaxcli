use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    pub env: Option<String>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Deploy {
        // #[arg(short, long)]
        path: std::path::PathBuf,
    },
}

use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    pub env: String,
    pub path: std::path::PathBuf,
}

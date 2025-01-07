use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    pub pattern: String,
    pub path: std::path::PathBuf,
}

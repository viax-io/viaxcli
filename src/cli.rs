use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    pub path: std::path::PathBuf,
}

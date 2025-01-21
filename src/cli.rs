use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, arg_required_else_help = true)]
pub struct Cli {
    pub env: Option<String>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Deploy an integration
    DeployInt {
        /// path to a zipped integration
        // #[arg(short, long)]
        path: std::path::PathBuf,
    },
    /// Deploy a function
    DeployFn {
        /// path to a zipped function
        // #[arg(short, long)]
        path: std::path::PathBuf,
    },
}

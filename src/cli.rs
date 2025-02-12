use core::fmt;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(version, arg_required_else_help = true)]
pub struct Cli {
    pub env: Option<String>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Functions management commands
    Fn {
        #[command(subcommand)]
        command: FnCommands,
    },
    /// Inegrations management commands
    Int {
        #[command(subcommand)]
        command: IntCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum IntCommands {
    /// Deploy a integration
    Deploy {
        /// path to a zipped integration
        path: PathBuf,
    },
    /// Get a integration
    Get {
        name: String,
    },
    Delete {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum FnCommands {
    /// Deploy a function
    Deploy {
        /// path to a zipped function
        path: PathBuf,
    },
    /// Get a function
    Get {
        name: String,
    },
    Create {
        #[arg(value_enum)]
        lang: Lang,
        name: String,
    },
    Delete {
        name: String,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Lang {
    Node,
    Typescript,
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

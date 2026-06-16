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
        /// path to a integration directory
        path: PathBuf,
    },
    /// Get a integration
    Get {
        name: String,
    },
    List,
    Delete {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum FnCommands {
    /// Deploy a function
    Deploy {
        /// path to a function directory
        path: PathBuf,
    },
    /// Get a function
    Get {
        name: String,
    },
    List,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(std::iter::once("viax").chain(args.iter().copied()))
            .expect("parse should succeed")
    }

    #[test]
    fn subcommand_wins_over_optional_env_positional() {
        // `viax fn list` — first arg looks positional but clap routes it to the subcommand.
        let cli = parse(&["fn", "list"]);
        assert!(cli.env.is_none());
        assert!(matches!(cli.command, Commands::Fn { command: FnCommands::List }));
    }

    #[test]
    fn env_positional_is_picked_up_before_subcommand() {
        let cli = parse(&["dev", "fn", "list"]);
        assert_eq!(cli.env.as_deref(), Some("dev"));
        assert!(matches!(cli.command, Commands::Fn { command: FnCommands::List }));
    }

    #[test]
    fn fn_get_captures_name() {
        let Commands::Fn { command: FnCommands::Get { name } } = parse(&["fn", "get", "my-fn"]).command
        else { panic!("expected Fn::Get") };
        assert_eq!(name, "my-fn");
    }

    #[test]
    fn fn_delete_captures_name() {
        let Commands::Fn { command: FnCommands::Delete { name } } = parse(&["fn", "delete", "my-fn"]).command
        else { panic!("expected Fn::Delete") };
        assert_eq!(name, "my-fn");
    }

    #[test]
    fn fn_deploy_captures_path() {
        let Commands::Fn { command: FnCommands::Deploy { path } } = parse(&["fn", "deploy", "./p"]).command
        else { panic!("expected Fn::Deploy") };
        assert_eq!(path, PathBuf::from("./p"));
    }

    #[test]
    fn fn_create_parses_both_languages() {
        for (arg, want) in [("node", Lang::Node), ("typescript", Lang::Typescript)] {
            let Commands::Fn { command: FnCommands::Create { lang, name } } =
                parse(&["fn", "create", arg, "my-fn"]).command
            else { panic!("expected Fn::Create for {arg}") };
            assert_eq!(lang, want);
            assert_eq!(name, "my-fn");
        }
    }

    #[test]
    fn fn_create_rejects_unknown_lang() {
        assert!(Cli::try_parse_from(["viax", "fn", "create", "ruby", "f"]).is_err());
    }

    #[test]
    fn int_subcommand_parses() {
        // Spot-check Int routing; argument parsing inside Int mirrors Fn so it doesn't need full coverage.
        let cli = parse(&["prod", "int", "list"]);
        assert_eq!(cli.env.as_deref(), Some("prod"));
        assert!(matches!(cli.command, Commands::Int { command: IntCommands::List }));
    }

    #[test]
    fn no_args_returns_help_error() {
        // `arg_required_else_help` means bare `viax` is a parse error, not a successful empty parse.
        assert!(Cli::try_parse_from(["viax"]).is_err());
    }

    #[test]
    fn lang_display_matches_value_enum_input() {
        // main.rs passes lang.to_string() to FunctionLanguage::from_str — these strings must round-trip.
        assert_eq!(Lang::Node.to_string(), "Node");
        assert_eq!(Lang::Typescript.to_string(), "Typescript");
    }
}

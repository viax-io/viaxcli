pub mod cli;

use crate::cli::Cli;
use api::fun::{command_create_fn, command_deploy_fn, delete_fn, get_fn};
use api::int::{command_deploy_int, delete_int, get_int};
use cli::{Commands, FnCommands, IntCommands};
use std::{error::Error, path::PathBuf};
use viax_config::config::ViaxConfig;

use clap::Parser;

fn main() -> Result<(), Box<dyn Error>> {
    let user_dir = directories::UserDirs::new().unwrap();
    let mut config_path = PathBuf::from(user_dir.home_dir());
    config_path.push(".viax/config");
    let cfg: ViaxConfig = confy::load_path(config_path.as_path())?;

    let args = Cli::parse();
    // println!("path: {:#?}", args);

    let env = args
        .env
        .or_else(|| -> Option<String> { Some("default".to_string()) })
        .unwrap();
    let env_cfg = cfg.config(&env);

    match &args.command {
        Commands::Int { command } => match command {
            IntCommands::Get { name } => {
                get_int(&cfg, env_cfg, &env, name)?;
            }
            IntCommands::Deploy { path } => command_deploy_int(&cfg, env_cfg, &env, path)?,
            IntCommands::Delete { name } => delete_int(&cfg, env_cfg, &env, name)?,
        },

        Commands::Fn { command } => match command {
            FnCommands::Get { name } => {
                get_fn(&cfg, env_cfg, &env, name)?;
            }
            FnCommands::Deploy { path } => command_deploy_fn(&cfg, env_cfg, &env, path)?,
            FnCommands::Delete { name } => delete_fn(&cfg, env_cfg, &env, name)?,
            FnCommands::Create { lang } => command_create_fn(&cfg, env_cfg, &env, lang)?,
        },
    };

    Ok(())
}

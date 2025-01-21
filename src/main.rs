pub mod api;
#[warn(dead_code)]
pub mod cli;
pub mod config;

use crate::cli::Cli;
use api::{command_deploy_fn, command_deploy_int};
use cli::Commands;
use std::{error::Error, path::PathBuf};

use clap::Parser;
use config::ViaxConfig;

fn main() -> Result<(), Box<dyn Error>> {
    let user_dir = directories::UserDirs::new().unwrap();
    let mut config_path = PathBuf::from(user_dir.home_dir());
    config_path.push(".viax/config");
    let cfg: ViaxConfig = confy::load_path(config_path.as_path())?;
    // println!("viax config '{:?}' created, path: {:#?}", cfg, config_path);

    let args = Cli::parse();
    // println!("path: {:#?}", args);

    let env = args
        .env
        .or_else(|| -> Option<String> { Some("default".to_string()) })
        .unwrap();
    let env_cfg = cfg.config(&env);

    match &args.command {
        Some(Commands::DeployInt { path }) => command_deploy_int(&cfg, env_cfg, &env, path)?,
        Some(Commands::DeployFn { path }) => command_deploy_fn(&cfg, env_cfg, &env, path)?,
        None => {}
    }

    Ok(())
}

pub mod cli;

use crate::cli::Cli;
use api::fun::{command_create_fn, command_deploy_fn, delete_fn, get_fn, list_fns};
use api::int::{command_deploy_int, delete_int, get_int, list_ints};
use cli::{Commands, FnCommands, IntCommands};
use rpassword::read_password;
use std::io::{self, Write};
use std::{error::Error, path::PathBuf};
use viax_config::config::{from_api_token, ViaxConfig, API_TOKEN_ENV};

use clap::Parser;

fn main() -> Result<(), Box<dyn Error>> {
    let api_token = std::env::var(API_TOKEN_ENV).ok();

    let cfg: ViaxConfig = if let Some(ref token) = api_token {
        from_api_token(token)?
    } else {
        let user_dir = directories::UserDirs::new().unwrap();
        let mut config_path = PathBuf::from(user_dir.home_dir());
        config_path.push(".viax/config");
        confy::load_path(config_path.as_path())?
    };

    let args = Cli::parse();
    // println!("path: {:#?}", args);

    let env = if api_token.is_some() {
        // env is derived from the JWT's iss claim; ignore CLI env arg.
        cfg.envs
            .keys()
            .next()
            .cloned()
            .expect("from_api_token must populate at least one env")
    } else {
        args.env.unwrap_or_else(|| "default".to_string())
    };
    let env_cfg = cfg.config(&env);

    let mut password = "".to_string();
    if api_token.is_none() && env_cfg.client_secret.is_none() {
        print!("Enter password: ");
        io::stdout().flush().unwrap();
        password = read_password().expect("Password must be entered!");
        // password = password.expect("Password must be entered!");
    }

    match &args.command {
        Commands::Int { command } => match command {
            IntCommands::Get { name } => {
                get_int(&cfg, env_cfg, &env, name, &password)?;
            }
            IntCommands::Deploy { path } => {
                command_deploy_int(&cfg, env_cfg, &env, &password, path)?
            }
            IntCommands::Delete { name } => delete_int(&cfg, env_cfg, &env, name, &password)?,
            IntCommands::List => list_ints(&cfg, env_cfg, &env, &password)?,
        },

        Commands::Fn { command } => match command {
            FnCommands::Get { name } => {
                get_fn(&cfg, env_cfg, &env, name, &password)?;
            }
            FnCommands::List => list_fns(&cfg, env_cfg, &env, &password)?,
            FnCommands::Deploy { path } => command_deploy_fn(&cfg, env_cfg, &env, &password, path)?,
            FnCommands::Delete { name } => delete_fn(&cfg, env_cfg, &env, name, &password)?,
            FnCommands::Create { lang, name } => {
                command_create_fn(&cfg, env_cfg, &env, &password, &lang.to_string(), name)?
            }
        },
    };

    Ok(())
}

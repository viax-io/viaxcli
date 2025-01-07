pub mod cli;
pub mod config;

use crate::cli::Cli;
use std::{error::Error, path::PathBuf};

use clap::Parser;
use config::ViaxConfig;

fn main() -> Result<(), Box<dyn Error>> {
    let user_dir = directories::UserDirs::new().unwrap();

    let mut config_path = PathBuf::from(user_dir.home_dir());
    config_path.push(".viax/config");
    let cfg: ViaxConfig = confy::load_path(config_path.as_path())?;

    println!("viax config '{:?}' created, path: {:#?}", cfg, config_path);

    let args = Cli::parse();

    println!("pattern: {:?}, path: {:?}", args.pattern, args.path);
    Ok(())
}

pub mod cli;
pub mod config;

use crate::cli::Cli;
use query::*;
use std::{error::Error, path::PathBuf};

use clap::Parser;
use config::ViaxConfig;

use cynic::QueryBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let q = FnMgmnt::build(FnMgmntVariables {
        name: Some("my-fun"),
    });
    // let response = surf::post("https://api.viax.lab.viax.tech/graphql").run_graphql(q);

    let viax_api_token = std::env::var("VIAX_API_TOKEN").expect("Missing VIAX_API_TOKEN env var");

    let response = reqwest::blocking::Client::new()
        .post("https://api.viax.lab.viax.tech/graphql")
        .bearer_auth(viax_api_token)
        .json(&q)
        .send()
        .unwrap();
    println!("response >>> {:#?}", response.text()?);

    let user_dir = directories::UserDirs::new().unwrap();

    let mut config_path = PathBuf::from(user_dir.home_dir());
    config_path.push(".viax/config");
    let cfg: ViaxConfig = confy::load_path(config_path.as_path())?;

    println!("viax config '{:?}' created, path: {:#?}", cfg, config_path);

    let args = Cli::parse();

    println!("pattern: {:?}, path: {:?}", args.pattern, args.path);
    Ok(())
}

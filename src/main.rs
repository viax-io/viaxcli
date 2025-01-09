#[warn(dead_code)]
pub mod cli;
pub mod config;

use crate::cli::Cli;
use cli::Commands;
// use query::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, path::PathBuf};

use clap::Parser;
use config::{ConfVal, ViaxConfig};

fn main() -> Result<(), Box<dyn Error>> {
    let user_dir = directories::UserDirs::new().unwrap();
    let mut config_path = PathBuf::from(user_dir.home_dir());
    config_path.push(".viax/config");
    let cfg: ViaxConfig = confy::load_path(config_path.as_path())?;
    // println!("viax config '{:?}' created, path: {:#?}", cfg, config_path);

    let args = Cli::parse();
    println!("path: {:#?}", args);

    let env = args
        .env
        .or_else(|| -> Option<String> { Some("default".to_string()) })
        .unwrap();
    let env_cfg = cfg.config(&env);

    match &args.command {
        Some(Commands::Deploy { path }) => command_deploy(&cfg, env_cfg, path)?,
        None => {}
    }

    // let q = FnMgmnt::build(FnMgmntVariables {
    //     name: Some("my-fun"),
    // });
    // let response = req_client
    //     .post("https://api.viax.lab.viax.tech/graphql")
    //     .bearer_auth(format!("Bearer {}", viax_api_token.access_token))
    //     .json(&q)
    //     .send()
    //     .unwrap();
    // println!("response >>> {:#?}", response.text()?);

    Ok(())
}

fn command_deploy(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let req_client = reqwest::blocking::Client::new();
    let viax_api_token = acquire_token(
        &env_cfg.auth_url,
        &env_cfg.client_id,
        &env_cfg.client_secret,
        &cfg.realm,
        &req_client,
    );
    // println!("api token: {:?}", viax_api_token);
    // let viax_api_token = std::env::var("VIAX_API_TOKEN").expect("Missing VIAX_API_TOKEN env var");

    let form = reqwest::blocking::multipart::Form::new()
        .text("operations", r#"{ "operationName": "upsertFunction",  "query": "mutation upsertFunction($file: Upload!) { upsertFunction(input: { fun: $file }) { uid } }",  "variables": {    "file": null } }"#)
        .text("map", r#"{ "File":["variables.file"] }"#)
        .file("File", path)?;
    //"/Users/inc/Work/viax/dev/viaxcli/fn.zip"
    // println!("{:?}", form);

    let response = req_client
        .post("https://api.viax.lab.viax.tech/graphql")
        .bearer_auth(format!("Bearer {}", viax_api_token.access_token))
        .multipart(form)
        .send()
        .unwrap();
    println!("response: {:#?}", response.text()?);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiToken {
    access_token: String,
}

fn acquire_token(
    url: &str,
    client_id: &str,
    client_secret: &str,
    realm: &str,
    client: &reqwest::blocking::Client,
) -> ApiToken {
    let response = client
        .post(format!(
            "{url}/realms/{realm}/protocol/openid-connect/token",
        ))
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "client_credentials"),
        ])
        .send();
    response.unwrap().json().unwrap()
}

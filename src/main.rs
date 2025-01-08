pub mod cli;
pub mod config;

use crate::cli::Cli;
use query::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, path::PathBuf};

use clap::Parser;
use config::ViaxConfig;

use cynic::QueryBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let user_dir = directories::UserDirs::new().unwrap();

    let mut config_path = PathBuf::from(user_dir.home_dir());
    config_path.push(".viax/config");
    let cfg: ViaxConfig = confy::load_path(config_path.as_path())?;

    println!("viax config '{:?}' created, path: {:#?}", cfg, config_path);

    let args = Cli::parse();

    println!("pattern: {:?}, path: {:?}", args.pattern, args.path);

    let req_client = reqwest::blocking::Client::new();
    let url = "https://auth.viax.lab.viax.tech";
    let def_cfg = cfg.env.get("default").unwrap();
    let client_id = &def_cfg.client_id;
    let client_secret = &def_cfg.secret;

    let viax_api_token = acquire_token(url, client_id, client_secret, "viax", &req_client);
    println!("api token: {:?}", viax_api_token);

    let q = FnMgmnt::build(FnMgmntVariables {
        name: Some("my-fun"),
    });
    // let viax_api_token = std::env::var("VIAX_API_TOKEN").expect("Missing VIAX_API_TOKEN env var");

    let response = req_client
        .post("https://api.viax.lab.viax.tech/graphql")
        .bearer_auth(format!("Bearer {}", viax_api_token.access_token))
        .json(&q)
        .send()
        .unwrap();
    println!("response >>> {:#?}", response.text()?);

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
    let access_toekn: ApiToken = response.unwrap().json().unwrap();
    access_toekn
}

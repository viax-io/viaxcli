use std::{error::Error, path::PathBuf};

use crate::config::{ConfVal, ViaxConfig};
use serde::{Deserialize, Serialize};

pub fn command_deploy(
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

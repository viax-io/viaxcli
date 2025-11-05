use crate::auth::acquire_token;
use std::path::PathBuf;

use reqwest::blocking::Response;
use viax_config::config::ConfVal;
use viax_config::config::ViaxConfig;

pub fn deploy(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    password: &String,
    path: &PathBuf,
    operation: String,
) -> Result<Response, reqwest::Error> {
    let req_client = reqwest::blocking::Client::new();
    let viax_api_token = acquire_token(env_cfg, &cfg.realm, env, password, &req_client);
    // let viax_api_token = std::env::var("VIAX_API_TOKEN").expect("Missing VIAX_API_TOKEN env var");

    let form = reqwest::blocking::multipart::Form::new()
        .text("operations", operation)
        .text("map", r#"{ "File":["variables.file"] }"#)
        .file("File", path)
        .unwrap();

    req_client
        .post(env_cfg.api_url(&cfg.realm, env))
        .bearer_auth(viax_api_token)
        .multipart(form)
        .send()
}

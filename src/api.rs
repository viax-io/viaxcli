use std::{error::Error, path::PathBuf};

use crate::config::{ConfVal, ViaxConfig};
use query::{FnDeploy, IntDeploy};
use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};

pub fn deploy(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    path: &PathBuf,
    operation: String,
) -> Response {
    let req_client = reqwest::blocking::Client::new();
    let viax_api_token = acquire_token(
        &env_cfg.auth_url(&cfg.realm, env),
        &env_cfg.client_id,
        &env_cfg.client_secret,
        &cfg.realm,
        &req_client,
    );
    // println!("api token: {:?}", viax_api_token);
    // let viax_api_token = std::env::var("VIAX_API_TOKEN").expect("Missing VIAX_API_TOKEN env var");

    let form = reqwest::blocking::multipart::Form::new()
        .text("operations", operation)
        .text("map", r#"{ "File":["variables.file"] }"#)
        .file("File", path)
        .unwrap();

    req_client
        .post(env_cfg.api_url(&cfg.realm, env))
        .bearer_auth(format!("Bearer {}", viax_api_token.access_token))
        .multipart(form)
        .send()
        .unwrap()
}

pub fn command_deploy_int(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let response = deploy(
        cfg,
        env_cfg,
        env,
        path,
        String::from(
            r#"{ "operationName": "upsertIntegrationDeployment", "query": "mutation upsertIntegrationDeployment($file: Upload!) { upsertIntegrationDeployment(input: { package: $file }) { uid name deployStatus latestDeploymentStartedAt enqueuedAt } }", "variables": { "file": null } }"#,
        ),
    );

    let data: cynic::GraphQlResponse<IntDeploy> = response.json()?;
    let intgr = data.data.unwrap().upsert_integration_deployment.unwrap();

    println!("Enqueued deployment:");
    println!(
        "uid: {}, deploy status: {:?}",
        intgr.uid.0,
        intgr.deploy_status.unwrap()
    );

    Ok(())
}

pub fn command_deploy_fn(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let response = deploy(
        cfg,
        env_cfg,
        env,
        path,
        String::from(
            r#"{ "operationName": "upsertFunction", "query": "mutation upsertFunction($file: Upload!) { upsertFunction(input: { fun: $file }) { uid deployStatus version readyRevision ready latestDeploymentStartedAt latestCreatedRevision enqueuedAt } }", "variables": { "file": null } }"#,
        ),
    );

    let data: cynic::GraphQlResponse<FnDeploy> = response.json()?;
    let fun = data.data.unwrap().upsert_function.unwrap();

    println!("Enqueued deployment:");
    println!(
        "uid: {}, deploy status: {:?}",
        fun.uid.0,
        fun.deploy_status.unwrap()
    );

    println!("Note: last deployed function will be working until new function is deployed. Previously deployed:");
    println!(
        "ready: {}, revision: {}",
        fun.ready.unwrap(),
        fun.ready_revision.unwrap()
    );

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

use std::{error::Error, path::PathBuf};

use crate::config::{ConfVal, ViaxConfig};
use cynic::QueryBuilder;
use query::{FnDeploy, FnMgmnt, FnMgmntVariables, Function, IntDeploy};
use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};

pub fn get_fn(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    name: &str,
) -> Result<Function, Box<dyn Error>> {
    use cynic::http::ReqwestBlockingExt;

    let req_client = reqwest::blocking::Client::new();
    let viax_api_token = acquire_token(env_cfg, &cfg.realm, env, &req_client);

    let q = FnMgmnt::build(FnMgmntVariables { name: Some(name) });

    let response = reqwest::blocking::Client::new()
        .post(env_cfg.api_url(&cfg.realm, env))
        .bearer_auth(viax_api_token)
        .run_graphql(q)
        .unwrap();

    if response.errors.is_some() {
        Err(format!(
            "Failed to get fn {name}, errors: {:?}",
            response.errors.unwrap()
        ))?
    } else {
        let fnmgmt = response.data.unwrap();
        if fnmgmt.get_function.is_none() {
            Err(format!("Function '{name}' not found"))?
        }
        let fun = fnmgmt.get_function.unwrap();
        println!("{:<30} {:<10}", "NAME", "READY");
        let ready = &fun.ready;
        println!("{:<30} {:<10}", fun.name, ready.as_ref().unwrap());
        Ok(fun)
    }
}

pub fn deploy(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    path: &PathBuf,
    operation: String,
) -> Response {
    let req_client = reqwest::blocking::Client::new();
    let viax_api_token = acquire_token(env_cfg, &cfg.realm, env, &req_client);
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
            r#"{ "operationName": "upsertFunction", "query": "mutation upsertFunction($file: Upload!) { upsertFunction(input: { fun: $file }) { uid name deployStatus version readyRevision ready latestDeploymentStartedAt latestCreatedRevision enqueuedAt } }", "variables": { "file": null } }"#,
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
    env_cfg: &ConfVal,
    realm: &str,
    env: &str,
    client: &reqwest::blocking::Client,
) -> String {
    let url = env_cfg.auth_url(realm, env);
    let client_id = &env_cfg.client_id;
    let client_secret = &env_cfg.client_secret;
    let grant_type = "client_credentials".to_string();

    let response = client
        .post(format!(
            "{url}/realms/{realm}/protocol/openid-connect/token",
        ))
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", &grant_type),
        ])
        .send();

    if response.is_err() {
        println!("Failed to get access_token, {:#?}", response);
        panic!()
    }

    let viax_api_token: ApiToken = response.unwrap().json().unwrap();
    format!("Bearer {}", viax_api_token.access_token)
}

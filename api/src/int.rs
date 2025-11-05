use crate::api::deploy;
use crate::auth::acquire_token;
use std::fs::remove_file;
use std::time::SystemTime;
use std::{error::Error, path::PathBuf};

use cynic::MutationBuilder;
use cynic::QueryBuilder;
use query::IntDelete;
use query::IntDeleteVariables;
use query::IntDeploy;
use query::IntDeployGet;
use query::IntGetVariables;
use query::IntList;
use query::Integration;
use query::IntegrationDeployment;
use query::Uuid;
use reqwest::blocking::Client;
use viax_config::config::ConfVal;
use viax_config::config::ViaxConfig;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip_extensions::zip_create_from_directory_with_options;

pub fn delete_int(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    name: &str,
    password: &String,
) -> Result<(), Box<dyn Error>> {
    use cynic::http::ReqwestBlockingExt;

    let req_client = reqwest::blocking::Client::new();
    let viax_api_token = acquire_token(env_cfg, &cfg.realm, env, password, &req_client);

    let int = get_int_with_token(cfg, env_cfg, env, &req_client, name, &viax_api_token).unwrap();

    let uid = int.uid;
    let q = IntDelete::build(IntDeleteVariables {
        uid: Uuid(String::from(&uid.0)),
    });

    let response = req_client
        .post(env_cfg.api_url(&cfg.realm, env))
        .bearer_auth(viax_api_token)
        .run_graphql(q)
        .unwrap();

    if response.errors.is_some() {
        Err(format!(
            "Failed to delete fn {name}, uid='{:?}', errors: {:?}",
            &uid.0,
            response.errors.unwrap()
        ))?
    } else {
        let int_delete = response.data.unwrap();
        let int = int_delete.delete_integration_deployment.unwrap();
        display_int(&int);
        Ok(())
    }
}

pub fn get_int_with_token(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    req_client: &Client,
    name: &str,
    api_token: &str,
) -> Result<IntegrationDeployment, Box<dyn Error>> {
    use cynic::http::ReqwestBlockingExt;

    let q = IntDeployGet::build(IntGetVariables { name: Some(name) });

    let response = req_client
        .post(env_cfg.api_url(&cfg.realm, env))
        .bearer_auth(api_token)
        .run_graphql(q)
        .unwrap();

    if response.errors.is_some() {
        Err(format!(
            "Failed to get fn {name}, errors: {:?}",
            response.errors.unwrap()
        ))?
    } else {
        let int_deploy = response.data.unwrap();
        if int_deploy.get_integration_deployment.is_none() {
            Err(format!("Function '{name}' not found"))?
        }
        let int = int_deploy.get_integration_deployment.unwrap();
        Ok(int)
    }
}

pub fn get_int(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    name: &str,
    password: &String,
) -> Result<IntegrationDeployment, Box<dyn Error>> {
    let req_client = reqwest::blocking::Client::new();
    let viax_api_token = acquire_token(env_cfg, &cfg.realm, env, password, &req_client);

    let int_result = get_int_with_token(cfg, env_cfg, env, &req_client, name, &viax_api_token);
    if let Ok(ref int) = int_result {
        display_int(int);
    }
    int_result
}

pub fn command_deploy_int(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    password: &String,
    path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let tmp_dir = std::env::temp_dir();

    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let int_bundle = tmp_dir.join(format!("i_{}.zip", now));

    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip_create_from_directory_with_options(&int_bundle, path, |_| options)?;
    println!("zip created {:?}", &int_bundle);

    let response = deploy(
        cfg,
        env_cfg,
        env,
        password,
        &int_bundle,
        String::from(
            r#"{ "operationName": "upsertIntegrationDeployment", "query": "mutation upsertIntegrationDeployment($file: Upload!) { upsertIntegrationDeployment(input: { package: $file }) { uid name deployStatus latestDeploymentStartedAt enqueuedAt } }", "variables": { "file": null } }"#,
        ),
    );

    println!("Cleaning up...");
    remove_file(&int_bundle)?;

    let data: cynic::GraphQlResponse<IntDeploy> = response?.json()?;
    let intgr = data.data.unwrap().upsert_integration_deployment.unwrap();

    println!("Enqueued deployment:");
    println!(
        "uid: {}, deploy status: {:?}",
        intgr.uid.0,
        intgr.deploy_status.unwrap()
    );

    Ok(())
}

pub fn list_ints(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    password: &String,
) -> Result<(), Box<dyn Error>> {
    use cynic::http::ReqwestBlockingExt;
    let req_client = reqwest::blocking::Client::new();
    let api_token = acquire_token(env_cfg, &cfg.realm, env, password, &req_client);

    let q = IntList::build(());

    let response = req_client
        .post(env_cfg.api_url(&cfg.realm, env))
        .bearer_auth(api_token)
        .run_graphql(q)
        .unwrap();

    if response.errors.is_some() {
        Err(format!(
            "Failed to get integrations list, errors: {:?}",
            response.errors.unwrap()
        ))?
    } else {
        let int_list = response.data.unwrap();
        if int_list.get_integrations.is_none() {
            println!("Integrations not found");
        } else {
            display_int_header();
            int_list
                .get_integrations
                .expect("Failed to deserealize integrations")
                .iter()
                .for_each(|integration| {
                    display_int_crd(integration.as_ref().unwrap());
                });
        }
        Ok(())
    }
}

fn display_int(int: &IntegrationDeployment) {
    println!(
        "{:<30} {:<20} {:<26} {:<26}",
        "NAME", "DEPLOY_STATUS", "ENQUEUED_AT", "LAST_DEPLOY_STARTED"
    );
    println!(
        "{:<30} {:<20} {:<26} {:<26}",
        int.name,
        format!("{:?}", &int.deploy_status.as_ref().unwrap()),
        int.enqueued_at.as_ref().unwrap().0,
        match int.latest_deployment_started_at {
            Some(ref latest_deployment_started_at) => &latest_deployment_started_at.0,
            None => "",
        }
    );
}

fn display_int_header() {
    println!("{:<30} {:<20}", "NAME", "PHASE");
}

fn display_int_crd(int: &Integration) {
    println!(
        "{:<30} {:<20}",
        int.name.as_ref().unwrap(),
        int.phase.as_ref().unwrap(),
    );
}

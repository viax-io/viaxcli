use crate::api::deploy;
use crate::auth::acquire_token;
use std::fs::{create_dir_all, remove_file, OpenOptions};
use std::{error::Error, path::PathBuf};
use std::{io, str::FromStr};

use cynic::{MutationBuilder, QueryBuilder};
use query::{
    FnDelete, FnDeleteVariables, FnDeploy, FnMgmnt, FnMgmntVariables, Function, FunctionLanguage,
    FunctionRuntimeResponse, Uuid,
};
use query::{FnTemplate, FnTemplateVariables};
use reqwest::blocking::Client;
use viax_config::config::ConfVal;
use viax_config::config::ViaxConfig;

pub fn delete_fn(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    name: &str,
    password: &String,
) -> Result<(), Box<dyn Error>> {
    use cynic::http::ReqwestBlockingExt;

    let req_client = reqwest::blocking::Client::new();
    let viax_api_token = acquire_token(env_cfg, &cfg.realm, env, password, &req_client);

    let fun = get_fn_with_token(cfg, env_cfg, env, &req_client, name, &viax_api_token).unwrap();

    let uid = fun.uid;
    let q = FnDelete::build(FnDeleteVariables {
        uid: Uuid(String::from(&uid.0)),
    });

    let response = req_client
        .post(env_cfg.api_url(&cfg.realm, env))
        .bearer_auth(viax_api_token)
        .run_graphql(q)
        .expect("Failed to retrive auth token");

    if response.errors.is_some() {
        Err(format!(
            "Failed to delete fn {name}, uid='{:?}', errors: {:?}",
            &uid.0,
            response.errors.unwrap()
        ))?
    } else {
        let fnmgmt = response.data.unwrap();
        let fun = fnmgmt.delete_function.unwrap();
        display_fn(&fun);
        Ok(())
    }
}

pub fn get_fn_with_token(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    req_client: &Client,
    name: &str,
    api_token: &str,
) -> Result<Function, Box<dyn Error>> {
    use cynic::http::ReqwestBlockingExt;

    let q = FnMgmnt::build(FnMgmntVariables { name: Some(name) });

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
        let fnmgmt = response.data.unwrap();
        if fnmgmt.get_function.is_none() {
            Err(format!("Function '{name}' not found"))?
        }
        let fun = fnmgmt.get_function.unwrap();
        Ok(fun)
    }
}

pub fn get_fn(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    name: &str,
    password: &String,
) -> Result<Function, Box<dyn Error>> {
    let req_client = reqwest::blocking::Client::new();
    let viax_api_token = acquire_token(env_cfg, &cfg.realm, env, password, &req_client);

    let fun_result = get_fn_with_token(cfg, env_cfg, env, &req_client, name, &viax_api_token);
    if let Ok(ref fun) = fun_result {
        display_fn(fun);
    }
    fun_result
}

pub fn command_deploy_fn(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    password: &String,
    path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let response = deploy(
        cfg,
        env_cfg,
        env,
        password,
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

fn display_fn(fun: &Function) {
    println!(
        "{:<30} {:<5} {:<20} {:<8} {:<10}",
        "NAME", "READY", "DEPLOY_STATUS", "VERSION", "REVISION"
    );
    let ready = &fun.ready;
    println!(
        "{:<30} {:<5} {:<20} {:<8} {:<10}",
        fun.name,
        ready.as_ref().unwrap(),
        format!("{:?}", &fun.deploy_status.as_ref().unwrap()),
        &fun.version.as_ref().unwrap(),
        &fun.ready_revision.as_ref().unwrap()
    );
}

pub fn get_fn_template(
    req_client: &reqwest::blocking::Client,
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    password: &String,
    lang: FunctionLanguage,
) -> Result<FunctionRuntimeResponse, Box<dyn Error>> {
    use cynic::http::ReqwestBlockingExt;

    let api_token = acquire_token(env_cfg, &cfg.realm, env, password, req_client);

    let q = FnTemplate::build(FnTemplateVariables { lang });

    let response = req_client
        .post(env_cfg.api_url(&cfg.realm, env))
        .bearer_auth(api_token)
        .run_graphql(q)
        .unwrap();

    if response.errors.is_some() {
        Err(format!(
            "Failed to get fn template, errors: {:?}",
            response.errors.unwrap()
        ))?
    } else {
        let fntmplt = response.data.unwrap();
        Ok(fntmplt.runtime_template.unwrap())
    }
}

pub fn command_create_fn(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    password: &String,
    lang: &str,
    name: &str,
) -> Result<(), Box<dyn Error>> {
    let fn_lang = FunctionLanguage::from_str(lang).expect("No such function lang available");

    let req_client = reqwest::blocking::Client::new();

    let src_dir = String::from(name);
    create_dir_all(&src_dir)?;

    let fnrt = get_fn_template(&req_client, cfg, env_cfg, env, password, fn_lang)?;
    let mut resp = req_client.get(fnrt.url.0).send().unwrap();

    let dst_zip = String::from(&src_dir) + "/tmplt.zip";
    let mut out_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&dst_zip)?;

    io::copy(&mut resp, &mut out_file)?;
    out_file.sync_all()?;

    let zip_file = OpenOptions::new().write(true).read(true).open(&dst_zip)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;
    let target_path = PathBuf::from_str(&src_dir)?;
    archive.extract(&target_path)?;

    remove_file(&dst_zip)?;

    println!("Successfully create {name} function! Check dir '{name}'.");
    Ok(())
}

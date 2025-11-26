use crate::api::deploy;
use crate::auth::acquire_token;
use std::fs::{self, create_dir_all, remove_file, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Component;
use std::time::SystemTime;
use std::{error::Error, path::PathBuf};
use std::{io, str::FromStr};

use cynic::{MutationBuilder, QueryBuilder};
use query::{
    FnDelete, FnDeleteVariables, FnDeploy, FnList, FnMgmnt, FnMgmntVariables, Function,
    FunctionLanguage, FunctionRuntimeResponse, Uuid,
};
use query::{FnTemplate, FnTemplateVariables};
use reqwest::blocking::Client;
use viax_config::config::ConfVal;
use viax_config::config::ViaxConfig;
use zip::result::ZipResult;
use zip::write::{FileOptionExtension, FileOptions, SimpleFileOptions};
use zip::{CompressionMethod, ZipWriter};

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

pub fn list_fns(
    cfg: &ViaxConfig,
    env_cfg: &viax_config::config::ConfVal,
    env: &str,
    password: &String,
) -> Result<(), Box<dyn Error>> {
    use cynic::http::ReqwestBlockingExt;

    let req_client = reqwest::blocking::Client::new();
    let api_token = acquire_token(env_cfg, &cfg.realm, env, password, &req_client);

    let q = FnList::build(());

    let response = req_client
        .post(env_cfg.api_url(&cfg.realm, env))
        .bearer_auth(api_token)
        .run_graphql(q)
        .unwrap();

    if response.errors.is_some() {
        Err(format!(
            "Failed to get list of funs, errors: {:?}",
            response.errors.unwrap()
        ))?
    } else {
        let fnlist = response.data.unwrap();
        if fnlist.filter_function.is_none() {
            println!("Functions not found");
        } else {
            display_header();
            fnlist
                .filter_function
                .expect("Failed to deserealize functions")
                .edges
                .expect("Failed to deserealize edges")
                .iter()
                .for_each(|edge| {
                    let fun = edge.node.as_ref().unwrap();
                    display_fn_data(fun);
                });
        }
        Ok(())
    }
}

pub fn command_deploy_fn(
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

    let fun_bundle = tmp_dir.join(format!("f_{}.zip", now));

    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let file = File::create(&fun_bundle)?;
    let zip_writer = ZipWriter::new(file);
    zip_writer.create_from_directory_with_options2(path, |_| options)?;

    // zip_create_from_directory_with_options(&fun_bundle, path, |_| options)?;
    println!("zip created {:?}", &fun_bundle);

    let response = deploy(
        cfg,
        env_cfg,
        env,
        password,
        &fun_bundle,
        String::from(
            r#"{ "operationName": "upsertFunction", "query": "mutation upsertFunction($file: Upload!) { upsertFunction(input: { fun: $file }) { uid name deployStatus version readyRevision ready latestDeploymentStartedAt latestCreatedRevision enqueuedAt } }", "variables": { "file": null } }"#,
        ),
    );

    println!("Cleaning up...");
    remove_file(&fun_bundle)?;

    if response.is_err() {
        println!("Failed to deploy function. {:?}", response);
        return Ok(());
    }
    let data: cynic::GraphQlResponse<FnDeploy> = response?.json()?;
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

fn display_header() {
    println!(
        "{:<35} {:<5} {:<20} {:<8} {:<10}",
        "NAME", "READY", "DEPLOY_STATUS", "VERSION", "REVISION"
    );
}

fn display_fn_data(fun: &Function) {
    let ready = &fun.ready;
    println!(
        "{:<35} {:<5} {:<20} {:<8} {:<10}",
        fun.name,
        ready.as_ref().unwrap(),
        format!("{:?}", &fun.deploy_status.as_ref().unwrap()),
        &fun.version.as_ref().unwrap(),
        &fun.ready_revision.as_ref().unwrap()
    );
}

fn display_fn(fun: &Function) {
    display_header();
    display_fn_data(fun);
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

pub trait ZipWriterExtensions {
    /// Creates a zip archive that contains the files and directories from the specified directory.
    fn create_from_directory(self, directory: &PathBuf) -> ZipResult<()>;

    /// Creates a zip archive that contains the files and directories from the specified directory, uses the specified compression level.
    fn create_from_directory_with_options2<F, T>(
        self,
        directory: &PathBuf,
        cb_file_options: F,
    ) -> ZipResult<()>
    where
        T: FileOptionExtension,
        F: Fn(&PathBuf) -> FileOptions<T>;
}

impl<W: Write + io::Seek> ZipWriterExtensions for ZipWriter<W> {
    fn create_from_directory(self, directory: &PathBuf) -> ZipResult<()> {
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        self.create_from_directory_with_options2(directory, |_| options)
    }

    fn create_from_directory_with_options2<F, T>(
        mut self,
        directory: &PathBuf,
        cb_file_options: F,
    ) -> ZipResult<()>
    where
        T: FileOptionExtension,
        F: Fn(&PathBuf) -> FileOptions<T>,
    {
        let mut paths_queue: Vec<PathBuf> = vec![];
        paths_queue.push(directory.clone());

        let mut buffer = Vec::new();

        while let Some(next) = paths_queue.pop() {
            let directory_entry_iterator = std::fs::read_dir(next)?;

            for entry in directory_entry_iterator {
                let entry_path = entry?.path();
                let file_options = cb_file_options(&entry_path);
                let entry_metadata = std::fs::metadata(entry_path.clone())?;
                let symlink_metadata = std::fs::symlink_metadata(entry_path.clone())?;
                if symlink_metadata.is_symlink() {
                    let target = fs::read_link(&entry_path)?;
                    let relative_path = make_relative_path(&directory, &entry_path);

                    self.add_symlink(
                        relative_path.to_str().unwrap(),
                        target.to_str().unwrap(),
                        SimpleFileOptions::default(),
                    )?;
                } else if entry_metadata.is_file() {
                    let mut f = File::open(&entry_path)?;
                    f.read_to_end(&mut buffer)?;
                    let relative_path = make_relative_path(&directory, &entry_path);
                    self.start_file(path_as_string(&relative_path), file_options)?;
                    self.write_all(buffer.as_ref())?;
                    buffer.clear();
                } else if entry_metadata.is_dir() {
                    let relative_path = make_relative_path(&directory, &entry_path);
                    self.add_directory(path_as_string(&relative_path), file_options)?;
                    paths_queue.push(entry_path.clone());
                }
            }
        }

        self.finish()?;
        Ok(())
    }
}

/// Returns a relative path from one path to another.
pub(crate) fn make_relative_path(root: &PathBuf, current: &PathBuf) -> PathBuf {
    let mut result = PathBuf::new();
    let root_components = root.components().collect::<Vec<Component>>();
    let current_components = current.components().collect::<Vec<_>>();
    for i in 0..current_components.len() {
        let current_path_component: Component = current_components[i];
        if i < root_components.len() {
            let other: Component = root_components[i];
            if other != current_path_component {
                break;
            }
        } else {
            result.push(current_path_component)
        }
    }
    result
}

// Returns a String representing the given Path.
pub(crate) fn path_as_string(path: &std::path::Path) -> String {
    let mut path_str = String::new();
    for component in path.components() {
        if let Component::Normal(os_str) = component {
            if !path_str.is_empty() {
                path_str.push('/');
            }
            path_str.push_str(&*os_str.to_string_lossy());
        }
    }
    path_str
}

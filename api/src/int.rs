use crate::api::deploy;
use std::{error::Error, path::PathBuf};

use query::IntDeploy;
use viax_config::config::ConfVal;
use viax_config::config::ViaxConfig;

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

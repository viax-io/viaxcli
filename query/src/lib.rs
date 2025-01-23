// use cynic;
use viax_schema::viax as schema;

#[derive(cynic::QueryVariables, Debug)]
pub struct FnDeployVariables {
    pub file: Upload,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "FnDeployVariables")]
pub struct FnDeploy {
    #[arguments(input: { fun: $file })]
    pub upsert_function: Option<Function>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Function {
    pub uid: Uuid,
    pub name: String,
    pub deploy_status: Option<DeployStatus>,
    pub version: Option<String>,
    pub ready_revision: Option<String>,
    pub ready: Option<String>,
    pub latest_deployment_started_at: Option<DateTime>,
    pub latest_created_revision: Option<String>,
    pub enqueued_at: Option<DateTime>,
    // pub fun: Option<File>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct IntDeployVariables {
    pub file: Upload,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "IntDeployVariables")]
pub struct IntDeploy {
    #[arguments(input: { package: $file })]
    pub upsert_integration_deployment: Option<IntegrationDeployment>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct IntegrationDeployment {
    pub uid: Uuid,
    pub name: String,
    pub deploy_status: Option<DeployStatus>,
    pub latest_deployment_started_at: Option<DateTime>,
    pub enqueued_at: Option<DateTime>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct File {
    pub name: String,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum DeployStatus {
    Cancelled,
    Deleting,
    Deploying,
    EnqueuedDeleting,
    EnqueuedDeploying,
    Failed,
    Ready,
    TimeOut,
    Unknown,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct DateTime(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "UUID")]
pub struct Uuid(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Upload(pub String);

#[derive(cynic::QueryVariables, Debug)]
pub struct FnMgmntVariables<'a> {
    pub name: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "FnMgmntVariables")]
pub struct FnMgmnt {
    #[arguments(name: $name)]
    pub get_function: Option<Function>,
}

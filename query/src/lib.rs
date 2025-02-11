use viax_schema::viax::{self as schema};

#[derive(cynic::QueryVariables, Debug)]
pub struct FnTemplateVariables {
    pub lang: FunctionLanguage,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "FnTemplateVariables")]
pub struct FnTemplate {
    #[arguments(input: { language: $lang })]
    pub runtime_template: Option<FunctionRuntimeResponse>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct FunctionRuntimeResponse {
    pub url: Url,
}

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

#[derive(cynic::QueryVariables, Debug)]
pub struct FnDeleteVariables {
    pub uid: Uuid,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "FnDeleteVariables")]
pub struct FnDelete {
    #[arguments(uid: $uid)]
    pub delete_function: Option<Function>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema = "viax")]
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
pub struct IntGetVariables<'a> {
    pub name: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "IntGetVariables")]
pub struct IntDeployGet {
    #[arguments(name: $name)]
    pub get_integration_deployment: Option<IntegrationDeployment>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct IntGet {
    pub get_integrations: Option<Vec<Option<Integration>>>,
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

#[derive(cynic::QueryVariables, Debug)]
pub struct IntDeleteVariables {
    pub uid: Uuid,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "IntDeleteVariables")]
pub struct IntDelete {
    #[arguments(uid: $uid)]
    pub delete_integration_deployment: Option<IntegrationDeployment>,
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
pub struct Integration {
    pub name: Option<String>,
    pub phase: Option<String>,
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

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum FunctionLanguage {
    #[cynic(rename = "Node")]
    Node,
    #[cynic(rename = "Typescript")]
    Typescript,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct DateTime(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "UUID")]
pub struct Uuid(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Upload(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Url(pub String);

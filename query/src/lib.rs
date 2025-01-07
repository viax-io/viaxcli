// use cynic;
use viax_schema::viax as schema;

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

#[derive(cynic::QueryFragment, Debug)]
pub struct Function {
    pub uid: Uuid,
    pub name: String,
    pub deploy_status: Option<DeployStatus>,
    pub ready: Option<String>,
    pub ready_revision: Option<String>,
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
#[cynic(graphql_type = "UUID")]
pub struct Uuid(pub String);

//
// #[derive(cynic::QueryFragment, Debug)]
// #[cynic(graphql_type = "Query")]
// pub struct FnMgmnt {
//     #[arguments(name: "my-fun")]
//     pub get_function: Option<Function>,
// }
//
// #[derive(cynic::QueryFragment, Debug)]
// pub struct Function {
//     pub uid: Uuid,
//     pub name: String,
//     pub deploy_status: Option<DeployStatus>,
//     pub ready: Option<String>,
//     pub ready_revision: Option<String>,
// }
//
// #[derive(cynic::Enum, Clone, Copy, Debug)]
// pub enum DeployStatus {
//     Cancelled,
//     Deleting,
//     Deploying,
//     EnqueuedDeleting,
//     EnqueuedDeploying,
//     Failed,
//     Ready,
//     TimeOut,
//     Unknown,
// }
//
// #[derive(cynic::Scalar, Debug, Clone)]
// #[cynic(graphql_type = "UUID")]
// pub struct Uuid(pub String);

// #[derive(cynic::QueryFragment, Debug)]
// #[cynic(schema = "viax", graphql_type = "Query")]
// pub struct Getfn {
//     #[arguments(name: "my-fun")]
//     pub get_function: Option<Function>,
// }
//
// #[derive(cynic::QueryFragment, Debug)]
// pub struct Function {
//     pub uid: Uuid,
//     pub name: String,
//     pub deploy_status: Option<DeployStatus>,
//     pub ready: Option<String>,
//     pub ready_revision: Option<String>,
// }
//
// #[derive(cynic::Enum, Clone, Copy, Debug)]
// pub enum DeployStatus {
//     Cancelled,
//     Deleting,
//     Deploying,
//     EnqueuedDeleting,
//     EnqueuedDeploying,
//     Failed,
//     Ready,
//     TimeOut,
//     Unknown,
// }
//
// #[derive(cynic::Scalar, Debug, Clone)]
// #[cynic(graphql_type = "UUID")]
// pub struct Uuid(pub String);

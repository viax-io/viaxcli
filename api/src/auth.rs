use serde::{Deserialize, Serialize};
use viax_config::config::ConfVal;

#[derive(Debug, Serialize, Deserialize)]
struct ApiToken {
    access_token: String,
}

pub fn acquire_token(
    env_cfg: &ConfVal,
    realm: &str,
    env: &str,
    password: &String,
    client: &reqwest::blocking::Client,
) -> String {
    let url = env_cfg.auth_url(realm, env);
    let client_id = &env_cfg.client_id;
    let client_secret = &env_cfg.client_secret;
    let user = &env_cfg.user;
    let grant_type_client_creds = "client_credentials".to_string();
    let grant_type_password = "password".to_string();
    let form_params = if client_secret.is_none() {
        vec![
            ("client_id", client_id),
            ("username", user.as_ref().unwrap()),
            ("password", password),
            ("grant_type", &grant_type_password),
        ]
    } else {
        vec![
            ("client_id", client_id),
            ("client_secret", client_secret.as_ref().unwrap()),
            ("grant_type", &grant_type_client_creds),
        ]
    };

    let response = client
        .post(format!(
            "{url}/realms/{realm}/protocol/openid-connect/token",
        ))
        .form(&form_params)
        .send();

    if response.is_err() || !response.as_ref().unwrap().status().is_success() {
        println!("Failed to get access_token, {:#?}", response);
        panic!()
    }

    let viax_api_token: ApiToken = response.unwrap().json().unwrap();
    format!("Bearer {}", viax_api_token.access_token)
}

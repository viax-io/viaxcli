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

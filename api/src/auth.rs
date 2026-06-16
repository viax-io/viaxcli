use serde::{Deserialize, Serialize};
use viax_config::config::{ConfVal, API_TOKEN_ENV};

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
    if let Ok(token) = std::env::var(API_TOKEN_ENV) {
        return format!("Bearer {}", token);
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn cv(secret: Option<&str>, user: Option<&str>, auth: &str) -> ConfVal {
        ConfVal::new(
            "cid".into(),
            secret.map(Into::into),
            user.map(Into::into),
            None,
            Some(auth.into()),
            None,
        )
    }

    #[test]
    #[serial(viax_env)]
    fn env_var_short_circuits_oidc() {
        std::env::set_var(API_TOKEN_ENV, "abc.def.ghi");

        let client = reqwest::blocking::Client::new();
        let conf = cv(Some("s"), None, "http://unreachable.invalid");
        let token = acquire_token(&conf, "viax", "prod", &"pw".to_string(), &client);

        std::env::remove_var(API_TOKEN_ENV);
        assert_eq!(token, "Bearer abc.def.ghi");
    }

    #[test]
    #[serial(viax_env)]
    fn client_credentials_grant_used_when_secret_present() {
        std::env::remove_var(API_TOKEN_ENV);

        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/realms/viax/protocol/openid-connect/token")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("client_id".into(), "cid".into()),
                mockito::Matcher::UrlEncoded("client_secret".into(), "shh".into()),
                mockito::Matcher::UrlEncoded("grant_type".into(), "client_credentials".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"access_token":"cc-token"}"#)
            .create();

        let client = reqwest::blocking::Client::new();
        let conf = cv(Some("shh"), None, &server.url());
        let token = acquire_token(&conf, "viax", "prod", &String::new(), &client);

        mock.assert();
        assert_eq!(token, "Bearer cc-token");
    }

    #[test]
    #[serial(viax_env)]
    fn password_grant_used_when_secret_absent() {
        std::env::remove_var(API_TOKEN_ENV);

        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/realms/myrealm/protocol/openid-connect/token")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("client_id".into(), "cid".into()),
                mockito::Matcher::UrlEncoded("username".into(), "alice".into()),
                mockito::Matcher::UrlEncoded("password".into(), "secret".into()),
                mockito::Matcher::UrlEncoded("grant_type".into(), "password".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"access_token":"pw-token"}"#)
            .create();

        let client = reqwest::blocking::Client::new();
        let conf = cv(None, Some("alice"), &server.url());
        let token = acquire_token(&conf, "myrealm", "dev", &"secret".to_string(), &client);

        mock.assert();
        assert_eq!(token, "Bearer pw-token");
    }

    #[test]
    #[should_panic]
    #[serial(viax_env)]
    fn panics_on_non_2xx_oidc_response() {
        std::env::remove_var(API_TOKEN_ENV);

        let mut server = mockito::Server::new();
        let _m = server
            .mock("POST", "/realms/viax/protocol/openid-connect/token")
            .with_status(401)
            .with_body("nope")
            .create();

        let client = reqwest::blocking::Client::new();
        let conf = cv(Some("shh"), None, &server.url());
        let _ = acquire_token(&conf, "viax", "prod", &String::new(), &client);
    }
}

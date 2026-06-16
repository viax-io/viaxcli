use crate::auth::acquire_token;
use std::path::PathBuf;

use reqwest::blocking::Response;
use viax_config::config::ConfVal;
use viax_config::config::ViaxConfig;

pub fn deploy(
    cfg: &ViaxConfig,
    env_cfg: &ConfVal,
    env: &str,
    password: &String,
    path: &PathBuf,
    operation: String,
) -> Result<Response, reqwest::Error> {
    let req_client = reqwest::blocking::Client::new();
    let viax_api_token = acquire_token(env_cfg, &cfg.realm, env, password, &req_client);
    // let viax_api_token = std::env::var("VIAX_API_TOKEN").expect("Missing VIAX_API_TOKEN env var");

    let form = reqwest::blocking::multipart::Form::new()
        .text("operations", operation)
        .text("map", r#"{ "File":["variables.file"] }"#)
        .file("File", path)
        .unwrap();

    req_client
        .post(env_cfg.api_url(&cfg.realm, env))
        .bearer_auth(viax_api_token)
        .multipart(form)
        .send()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::io::Write;
    use viax_config::config::API_TOKEN_ENV;

    #[test]
    #[serial(viax_env)]
    fn deploy_posts_multipart_with_bearer_token() {
        std::env::set_var(API_TOKEN_ENV, "abc.def.ghi");

        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/graphql")
            // Loosely match the bearer token regardless of how reqwest renders the prefix.
            .match_header(
                "authorization",
                mockito::Matcher::Regex("abc.def.ghi".into()),
            )
            // operations + map + File together prove the multipart envelope is well-formed.
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex(r#"name="operations""#.into()),
                mockito::Matcher::Regex(r#"name="map""#.into()),
                mockito::Matcher::Regex(r#"name="File""#.into()),
                mockito::Matcher::Regex("upsertFunction".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"data":{"upsertFunction":null}}"#)
            .create();

        let tmp = tempfile::tempdir().unwrap();
        let zip_path = tmp.path().join("bundle.zip");
        std::fs::File::create(&zip_path)
            .unwrap()
            .write_all(b"fake zip content")
            .unwrap();

        let env_cfg = ConfVal::new(
            "cid".into(),
            None,
            None,
            None,
            None,
            Some(format!("{}/graphql", server.url())),
        );
        let mut envs = HashMap::new();
        envs.insert("test".into(), env_cfg);
        let cfg = ViaxConfig { realm: "viax".into(), envs };

        let resp = deploy(
            &cfg,
            cfg.config("test"),
            "test",
            &String::new(),
            &zip_path,
            r#"{"operationName":"upsertFunction","query":"mutation upsertFunction($file: Upload!) { upsertFunction(input: { fun: $file }) { uid } }","variables":{"file":null}}"#.into(),
        )
        .expect("deploy should send the request");

        std::env::remove_var(API_TOKEN_ENV);
        mock.assert();
        assert_eq!(resp.status(), 200);
    }
}

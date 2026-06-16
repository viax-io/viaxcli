use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use std::error::Error;

pub const API_TOKEN_ENV: &str = "VIAX_API_TOKEN";

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfVal {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
    auth_url: Option<String>,
    api_url: Option<String>,
}

impl ConfVal {
    pub fn new(
        client_id: String,
        client_secret: Option<String>,
        user: Option<String>,
        password: Option<String>,
        auth_url: Option<String>,
        api_url: Option<String>,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            user,
            password,
            auth_url,
            api_url,
        }
    }

    pub fn auth_url(&self, realm: &str, env: &str) -> String {
        self.auth_url
            .clone()
            .unwrap_or(format!("https://auth.{realm}.{env}.viax.io"))
    }

    pub fn api_url(&self, realm: &str, env: &str) -> String {
        self.api_url
            .clone()
            .unwrap_or(format!("https://api.{realm}.{env}.viax.io/graphql"))
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct ViaxConfig {
    pub realm: String,
    #[serde(flatten)]
    pub envs: HashMap<String, ConfVal>,
}

impl ViaxConfig {
    pub fn config(&self, env: &str) -> &ConfVal {
        let def_cfg = self
            .envs
            .get(env)
            .or_else(|| -> Option<&ConfVal> { self.envs.get("default") })
            .expect(
                "Env is not present in config, define 'default' config or pass it as 1st argument",
            );
        def_cfg
    }
}

pub fn from_api_token(token: &str) -> Result<ViaxConfig, Box<dyn Error>> {
    let payload_segment = token
        .split('.')
        .nth(1)
        .ok_or("VIAX_API_TOKEN is not a valid JWT (missing payload)")?;
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_segment.trim_end_matches('='))
        .map_err(|e| format!("Failed to base64-decode JWT payload: {e}"))?;
    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
        .map_err(|e| format!("Failed to parse JWT payload as JSON: {e}"))?;
    let iss = payload
        .get("iss")
        .and_then(|v| v.as_str())
        .ok_or("JWT payload is missing 'iss' claim")?;

    let parsed = parse_iss(iss)?;

    let conf_val = ConfVal {
        client_id: String::new(),
        client_secret: Some(String::new()),
        user: None,
        password: None,
        auth_url: Some(parsed.auth_url),
        api_url: Some(parsed.api_url),
    };
    let mut envs = HashMap::new();
    envs.insert(parsed.env, conf_val);
    Ok(ViaxConfig {
        realm: parsed.realm,
        envs,
    })
}

struct ParsedIss {
    realm: String,
    env: String,
    auth_url: String,
    api_url: String,
}

fn parse_iss(iss: &str) -> Result<ParsedIss, Box<dyn Error>> {
    let trimmed = iss.trim_end_matches('/');

    let (auth_url, realms_part) = trimmed
        .split_once("/realms/")
        .ok_or_else(|| format!("iss '{iss}' is missing '/realms/' segment"))?;
    let realm = realms_part
        .split('/')
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("Cannot extract realm from iss '{iss}'"))?
        .to_string();

    let scheme_split = auth_url
        .split_once("://")
        .ok_or_else(|| format!("iss '{iss}' is missing scheme"))?;
    let (scheme, host) = scheme_split;
    if !host.starts_with("auth.") {
        return Err(format!("iss host '{host}' does not start with 'auth.'").into());
    }
    let api_host = host.replacen("auth.", "api.", 1);
    let api_url = format!("{scheme}://{api_host}/graphql");

    // Host convention: auth.{realm}.{env}.{tld...}; the env segment is index 2.
    let env = host
        .split('.')
        .nth(2)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("Cannot extract env from iss host '{host}'"))?
        .to_string();

    Ok(ParsedIss {
        realm,
        env,
        auth_url: auth_url.to_string(),
        api_url,
    })
}

impl ::std::default::Default for ViaxConfig {
    fn default() -> Self {
        let conf_val = ConfVal {
            client_id: "".into(),
            client_secret: Some("".to_string()),
            user: Some("".to_string()),
            password: Some("".to_string()),
            auth_url: Some("".to_string()),
            api_url: Some("".to_string()),
        };
        let mut vals = HashMap::new();
        vals.insert("default".to_string(), conf_val);
        Self {
            envs: vals,
            realm: "viax".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    fn jwt_with_iss(iss: &str) -> String {
        let payload = URL_SAFE_NO_PAD.encode(format!(r#"{{"iss":"{iss}"}}"#).as_bytes());
        format!("h.{payload}.s")
    }

    fn cv(auth: Option<&str>, api: Option<&str>) -> ConfVal {
        ConfVal::new("cid".into(), None, None, None, auth.map(Into::into), api.map(Into::into))
    }

    #[test]
    fn auth_url_uses_default_pattern_or_override() {
        assert_eq!(cv(None, None).auth_url("viax", "prod"), "https://auth.viax.prod.viax.io");
        assert_eq!(cv(Some("https://x"), None).auth_url("viax", "prod"), "https://x");
    }

    #[test]
    fn api_url_uses_default_pattern_or_override() {
        assert_eq!(cv(None, None).api_url("acme", "dev"), "https://api.acme.dev.viax.io/graphql");
        assert_eq!(cv(None, Some("https://y/q")).api_url("acme", "dev"), "https://y/q");
    }

    #[test]
    fn config_returns_named_env_then_falls_back_to_default() {
        let mut envs = HashMap::new();
        envs.insert("default".into(), cv(Some("d"), None));
        envs.insert("prod".into(), cv(Some("p"), None));
        let cfg = ViaxConfig { realm: "viax".into(), envs };

        assert_eq!(cfg.config("prod").auth_url("viax", "prod"), "p");
        assert_eq!(cfg.config("missing").auth_url("viax", "x"), "d");
    }

    #[test]
    #[should_panic]
    fn config_panics_when_no_match_and_no_default() {
        let cfg = ViaxConfig { realm: "viax".into(), envs: HashMap::new() };
        let _ = cfg.config("missing");
    }

    #[test]
    fn default_seeds_a_default_env() {
        // ViaxConfig::default is what confy uses when no config file exists.
        let cfg = ViaxConfig::default();
        assert_eq!(cfg.realm, "viax");
        assert!(cfg.envs.contains_key("default"));
    }

    #[test]
    fn from_api_token_extracts_realm_env_and_urls() {
        let cfg = from_api_token(&jwt_with_iss(
            "https://auth.acme.prod.viax.io/realms/myrealm",
        ))
        .unwrap();
        assert_eq!(cfg.realm, "myrealm");
        let env_cfg = cfg.envs.get("prod").expect("prod env populated");
        assert_eq!(env_cfg.auth_url("acme", "prod"), "https://auth.acme.prod.viax.io");
        assert_eq!(env_cfg.api_url("acme", "prod"), "https://api.acme.prod.viax.io/graphql");
    }

    #[test]
    fn from_api_token_tolerates_trailing_slash_in_iss() {
        let cfg = from_api_token(&jwt_with_iss(
            "https://auth.acme.dev.viax.io/realms/r/",
        ))
        .unwrap();
        assert!(cfg.envs.contains_key("dev"));
    }

    #[test]
    fn from_api_token_rejects_malformed_inputs() {
        // Each case exercises a distinct error branch; the substring tells us which one fired.
        let payload_no_iss = URL_SAFE_NO_PAD.encode(br#"{"sub":"u"}"#);
        let cases: Vec<(String, &str)> = vec![
            ("not-a-jwt".into(), "not a valid JWT"),
            (format!("h.{payload_no_iss}.s"), "'iss'"),
            (jwt_with_iss("https://auth.x.y.z/oops/r"), "/realms/"),
            (jwt_with_iss("https://idp.x.y.z/realms/r"), "'auth.'"),
            (jwt_with_iss("auth.x.y.z/realms/r"), "scheme"),
        ];
        for (token, want) in cases {
            let err = from_api_token(&token).unwrap_err().to_string();
            assert!(err.contains(want), "expected {want:?} in error, got {err:?}");
        }
    }
}

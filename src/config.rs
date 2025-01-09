use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfVal {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub api_url: String,
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

impl ::std::default::Default for ViaxConfig {
    fn default() -> Self {
        let conf_val = ConfVal {
            client_id: "".into(),
            client_secret: "".into(),
            auth_url: "".to_string(),
            api_url: "".to_string(),
        };
        let mut vals = HashMap::new();
        vals.insert("default".to_string(), conf_val);
        Self {
            envs: vals,
            realm: "viax".to_string(),
        }
    }
}

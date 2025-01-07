use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfVal {
    pub client_id: String,
    pub secret: String,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct ViaxConfig {
    #[serde(flatten)]
    pub env: HashMap<String, ConfVal>,
}
impl ::std::default::Default for ViaxConfig {
    fn default() -> Self {
        let conf_val = ConfVal {
            client_id: "42".into(),
            secret: "hello".into(),
        };
        let mut vals = HashMap::new();
        vals.insert("default".to_string(), conf_val);
        Self { env: vals }
    }
}

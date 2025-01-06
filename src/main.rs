use std::{collections::HashMap, error::Error, path::PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Cli {
    pattern: String,
    path: std::path::PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfVal {
    client_id: String,
    secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ViaxConfig {
    env: HashMap<String, ConfVal>,
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

fn main() -> Result<(), Box<dyn Error>> {
    let user_dir = directories::UserDirs::new().unwrap();

    let mut config_path = PathBuf::from(user_dir.home_dir());
    config_path.push(".viax/config");
    let cfg: ViaxConfig = confy::load_path(config_path.as_path())?;

    println!("viax config '{:?}' created, path: {:#?}", cfg, config_path);

    let args = Cli::parse();

    println!("pattern: {:?}, path: {:?}", args.pattern, args.path);
    Ok(())
}

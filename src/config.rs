use std::{collections::HashMap, path::Path};
use anyhow::Result;

use serde::{Deserialize, Serialize};
use figment::{Figment, providers::{Format, Serialized, Yaml}};

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub image: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub profiles: HashMap<String, Profile>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            profiles: HashMap::from([
                ("aider".to_string(), Profile {
                    image: "nixery.dev/shell/nix/aider-chat".to_string()
                })
            ])
        }
    }
}

pub fn parse_config<P>(config_file: Option<P>) -> Result<Config>
where P: AsRef<Path> {
    let mut config_manager = Figment::new()
        .merge(Serialized::defaults(Config::default()));
    if let Some(path) = config_file {
        config_manager = config_manager.merge(Yaml::file(path));
    }
    Ok(config_manager.extract()?)
}
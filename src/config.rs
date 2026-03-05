use anyhow::{Context, Result};
use delegate::delegate;
use std::{collections::HashMap, path::Path};

use figment::{
    Figment,
    providers::{Format, Serialized, Yaml},
};
use serde::{Deserialize, Serialize};

use crate::args::CmdLineEngineConfig;

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub image: String,
    #[serde(flatten)]
    pub cmd_line_config_defaults: CmdLineEngineConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    profiles: HashMap<String, Profile>,
}

impl Config {
    delegate! {
        to self.profiles {
            #[call(get)]
            pub fn get_profile(&self, name: &String) -> Option<&Profile>;
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            profiles: HashMap::from([(
                "aider".into(),
                Profile {
                    image: "nixery.dev/shell/nix/aider-chat".into(),
                    cmd_line_config_defaults: CmdLineEngineConfig::default(),
                },
            )]),
        }
    }
}

pub fn parse_config<P>(config_file: Option<P>) -> Result<Config>
where
    P: AsRef<Path>,
{
    let mut config_manager = Figment::new().merge(Serialized::defaults(Config::default()));
    if let Some(path) = config_file {
        config_manager = config_manager.merge(Yaml::file(path));
    }
    Ok(config_manager
        .extract()
        .context("Parse and merge profile config")?)
}

use anyhow::{Context, Result};
use delegate::delegate;
use std::{collections::HashMap, path::Path};

use figment::{
    Figment,
    providers::{Format, Yaml},
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

static DEFAULT_CONFIG: &'static str = r#"
profiles:
    aider:
        image: nixery.dev/shell/nix/busybox/aider-chat
"#;

pub fn parse_config<P>(config_file: Option<P>) -> Result<Config>
where
    P: AsRef<Path>,
{
    let mut config_manager = Figment::new().merge(Yaml::string(DEFAULT_CONFIG));
    if let Some(path) = config_file {
        config_manager = config_manager.merge(Yaml::file(path));
    }
    Ok(config_manager
        .extract()
        .context("Parse and merge profile config")?)
}

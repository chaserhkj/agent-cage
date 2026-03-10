use anyhow::{Context, Result};
use std::{collections::HashMap, path::Path};

use figment::{
    Figment,
    providers::{Format, Yaml, Serialized},
};
use serde::{Deserialize, Serialize};

use crate::{args::CmdLineEngineConfig, rel_provider::YamlWithRel};

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub image: String,
    #[serde(flatten)]
    pub cmd_line_config_defaults: CmdLineEngineConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    profiles: HashMap<String, Profile>,
    global: Option<CmdLineEngineConfig>,
}

impl Config {
    pub fn get_merged_profile(&self, name: &String) -> Option<Profile> {
        let Some(profile) = self.profiles.get(name) else {
            return None;
        };
        let mut profile_config_manager =
            Figment::new().merge(Serialized::defaults(profile));
        if let Some(ref global_config) = self.global {
            profile_config_manager = profile_config_manager.adjoin(Serialized::defaults(global_config));
        }
        Some(Profile {
            image: profile.image.clone(),
            cmd_line_config_defaults: profile_config_manager.extract().ok()?
        })

    }
}

static DEFAULT_CONFIG: &'static str = include_str!("./defaults.yaml");

fn contextual_config() -> Figment {
    let mut contextual_config_files = Vec::new();
    let cwd = std::env::current_dir().ok();
    let mut current_dir = cwd.as_deref();
    while let Some(dir) = current_dir {
        let config_file = dir.join("agent-cage.yaml");
        if config_file.is_file() {
            contextual_config_files.push(config_file)
        }
        current_dir = dir.parent();
    }
    contextual_config_files
        .iter()
        .rev()
        .fold(Figment::new(), |f, config| {
            f.admerge(YamlWithRel::new(config))
        })
}

pub fn parse_config<P>(
    config_file: Option<P>,
    parse_default: bool,
    parse_contextual: bool,
) -> Result<Config>
where
    P: AsRef<Path>,
{
    let mut config_manager = Figment::new();
    if parse_default {
        config_manager = config_manager.merge(Yaml::string(DEFAULT_CONFIG))
    }
    if parse_contextual {
        config_manager = config_manager.admerge(contextual_config())
    }
    if let Some(file_name) = config_file.as_ref() {
        config_manager = config_manager.admerge(YamlWithRel::new(file_name.as_ref()))
    }
    Ok(config_manager
        .extract()
        .context("Parse and merge profile config")?)
}

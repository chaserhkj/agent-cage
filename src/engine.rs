use anyhow::{Context, Result};
use figment::{Figment, providers::Serialized};

use crate::{args::{CmdLineEngineConfig, ResolvedCmdLineEngineConfig}, config::Profile};

/// Construct command line arguments passed to the container engine
/// Current only support podman as container engine

#[derive(Debug)]
struct EngineArgs {
    image: String,
    volumes: Vec<VolumeSpec>,
    work_dir: Option<String>,
}

#[derive(Debug)]
struct VolumeSpec {
    src: String,
    dst: String,
    flag: Option<String>,
}

#[derive(Debug)]
pub struct EngineConfig {
    image: String,
    cmd_line_config: ResolvedCmdLineEngineConfig,
}

// impl Into<EngineArgs> for EngineConfig {
//     fn into(self) -> EngineArgs {
        
//     }
// }

impl Profile {
    pub fn instantiate(&self, parsed_config: &CmdLineEngineConfig) -> Result<EngineConfig> {
        Ok(EngineConfig {
            image: self.image.to_owned(),
            cmd_line_config: parsed_config
                .resolve(&self.cmd_line_config_defaults)
                .context("Instantiate profile")?,
        })
    }
}

impl CmdLineEngineConfig {
    /// Baseline defaults
    fn base() -> Self {
        Self { 
            cwd: Some(true),
            runtime: Some("krun".into()),
        }
    }
    /// Resolves command line engine config from, in priority ascending order:
    ///   Base config defined above
    ///   Passed in defaults (as parsed from profile config)
    ///   Values stored in current config struct (as parsed from command line)
    pub fn resolve(&self, defaults: &Self) -> Result<ResolvedCmdLineEngineConfig> {
        let result  = Figment::new()
            .merge(Serialized::defaults(Self::base()))
            .merge(Serialized::defaults(defaults))
            .merge(Serialized::defaults(self))
            .extract()
            .context("Resolve command line engine config")?;
        Ok(result)
    }
}

use std::iter::once;

use anyhow::{Context, Result};
use figment::{Figment, providers::Serialized};

use crate::{
    args::{CmdLineEngineConfig, ResolvedCmdLineEngineConfig},
    config::Profile, engine,
};

/// Construct command line arguments passed to the container engine
/// Current only support podman as container engine

#[derive(Debug)]
struct EngineArgs {
    image: String,
    runtime: String,
    volumes: Vec<VolumeSpec>,
    work_dir: Option<String>,
}

impl From<EngineArgs> for Vec<String> {
    fn from(value: EngineArgs) -> Self {
        // --runtime <runtime>
        once("--runtime".into())
        .chain(once(value.runtime))
        // Volume flags, chained together
        .chain(
            value.volumes.into_iter()
            .flat_map(|v| Vec::from(v)))
        // --workdir <workdir>
        .chain(
            value.work_dir.into_iter()
            .flat_map(|dir|
                once("--workdir".into()).chain(once(dir))
                ))
        // Image ref
        .chain(once(value.image))
        .collect()
    }
}

#[derive(Debug)]
struct VolumeSpec {
    src: String,
    dst: String,
    flag: Option<String>,
}

impl From<VolumeSpec> for Vec<String> {
    fn from(value: VolumeSpec) -> Self {
        vec![
            "--volume".into(),
            format!(
                "{}:{}{}",
                value.src,
                value.dst,
                value.flag.map(|f| format!(":{}", f)).unwrap_or("".into())
            ),
        ]
    }
}

#[derive(Debug)]
pub struct EngineConfig {
    image: String,
    cmd_line_config: ResolvedCmdLineEngineConfig,
}

impl EngineConfig {
    pub fn into_cmd_args(self) -> Vec<String> {
        let engine_args: EngineArgs = self.into();
        Vec::from(engine_args)
    }
}

impl From<EngineConfig> for EngineArgs {
    fn from(config: EngineConfig) -> Self {
        let mut vols = Vec::new();
        let work_dir = if config.cmd_line_config.cwd {
            vols.push(VolumeSpec {
                src: ".".into(),
                dst: "/work".into(),
                flag: None,
            });
            Some("/work".into())
        } else {
            None
        };

        Self {
            image: config.image,
            runtime: config.cmd_line_config.runtime,
            volumes: vols,
            work_dir: work_dir,
        }
    }
}

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
        let result = Figment::new()
            .merge(Serialized::defaults(Self::base()))
            .merge(Serialized::defaults(defaults))
            .merge(Serialized::defaults(self))
            .extract()
            .context("Resolve command line engine config")?;
        Ok(result)
    }
}

use std::iter::once;

use anyhow::{Context, Result, anyhow};
use figment::{Figment, providers::Serialized};

use crate::{
    args::{CmdLineEngineConfig, ResolvedCmdLineEngineConfig, TermConnectionType, OpMode},
    config::Profile,
};

/// Construct command line arguments passed to the container engine
/// Current only support podman as container engine

#[derive(Debug)]
struct EngineArgs {
    image: String,
    name: Option<String>,
    runtime: String,
    volumes: Vec<String>,
    envs: Vec<String>,
    env_file: Option<String>,
    ports: Vec<String>,
    work_dir: Option<String>,
    remove: bool,
    interactive: bool,
    tty: bool,
    detach: bool,
    command: Vec<String>,
}

impl From<EngineArgs> for Vec<String> {
    fn from(value: EngineArgs) -> Self {
        // --runtime <runtime>
        value
            .name
            .into_iter()
            .flat_map(|n| once("--name".into()).chain(once(n)))
            .chain(once("--runtime".into()))
            .chain(once(value.runtime))
            // Volume flags, chained together
            .chain(
                value
                    .volumes
                    .into_iter()
                    .flat_map(|v| once("--volume".into()).chain(once(v))),
            )
            // Environment flags, chained together
            .chain(
                value
                    .envs
                    .into_iter()
                    .flat_map(|e| once("--env".into()).chain(once(e))),
            )
            // Environment file
            .chain(
                value
                    .env_file
                    .into_iter()
                    .flat_map(|fp| once("--env-file".into()).chain(once(fp))),
            )
            // Port flags, chained together
            .chain(
                value
                    .ports
                    .into_iter()
                    .flat_map(|p| once("--publish".into()).chain(once(p))),
            )
            // --workdir <workdir>
            .chain(
                value
                    .work_dir
                    .into_iter()
                    .flat_map(|dir| once("--workdir".into()).chain(once(dir))),
            )
            .chain(value.remove.then_some("--remove".into()))
            .chain(value.interactive.then_some("--interactive".into()))
            .chain(value.tty.then_some("--tty".into()))
            .chain(value.detach.then_some("--detach".into()))
            // Image ref
            .chain(once(value.image))
            .chain(value.command.into_iter())
            .collect()
    }
}

#[derive(Debug)]
pub struct EngineConfig {
    image: String,
    name: Option<String>,
    cmd_line_config: ResolvedCmdLineEngineConfig,
    ephemeral: bool,
}

/// Substitute variable references from environment variables
fn sub_env<S>(string: S) -> String
where
    S: AsRef<str>,
{
    subst::substitute(string.as_ref(), &subst::Env).unwrap_or(string.as_ref().into())
}

/// Run child process in foreground and waiting for it to return
fn run_in_foreground<'a, I>(command: &str, args: I, ignore_rtn_code: bool) -> Result<()> 
where I: IntoIterator<Item = &'a str>
{
    let mut proc = std::process::Command::new(command)
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn().context("Spawn child process to run in foreground")?;
    let status = proc.wait().context("Wait for child process to return from foreground")?;
    if status.success() || ignore_rtn_code {
        Ok(())
    } else {
        Err(anyhow!("Foreground child process failed with status: {}", status))
    }
}

static ISOLATED_GIT_REPO_PREPARE_SCRIPT: &'static str = include_str!("./prepare-isolated-git-repo.sh");

impl EngineConfig {
    /// Runs prepare procedures according to config
    pub fn run_prepare(&self) -> Result<()> {
        match self.cmd_line_config.mode {
            OpMode::IsolatedGitRepo => {
                run_in_foreground("/bin/sh", ["-c", ISOLATED_GIT_REPO_PREPARE_SCRIPT], false)
            }
            _ => {Ok(())}
        }
    }
    pub fn with_ephemeral(mut self) -> Self {
        self.ephemeral = true;
        self
    }
    pub fn with_name<S>(mut self, name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.name = Some(name.as_ref().to_string());
        self
    }
}

impl EngineConfig {
    pub fn into_cmd_args(self) -> Vec<String> {
        let engine_args: EngineArgs = self.into();
        Vec::from(engine_args)
    }
}

impl From<EngineConfig> for EngineArgs {
    fn from(config: EngineConfig) -> Self {
        let mut volumes: Vec<_> = config
            .cmd_line_config
            .volumes
            .into_iter()
            .map(|s| sub_env(s))
            .collect();
        volumes.extend(config.cmd_line_config.mode.to_volume_mounts());
        let work_dir = config.cmd_line_config.mode.to_work_dir();

        let envs: Vec<_> = config
            .cmd_line_config
            .envs
            .into_iter()
            .map(|e| sub_env(e))
            .collect();

        let mut ports = Vec::new();
        if config.cmd_line_config.terminal_connection_type == TermConnectionType::Telnet {
            ports.push(format!("{}:23", config.cmd_line_config.telnet_bind));
        }

        let use_terminal =
            config.cmd_line_config.terminal_connection_type == TermConnectionType::Direct;
        let override_commands =
            shell_words::split(&config.cmd_line_config.command).unwrap_or(vec![]);
        let raw_command = if override_commands.is_empty() {
            match config.cmd_line_config.terminal_connection_type {
                TermConnectionType::Direct => vec!["bash".into()],
                TermConnectionType::Telnet => vec!["busybox", "telnetd", "-F", "-l", "bash"]
                    .into_iter()
                    .map(|s| s.into())
                    .collect(),
            }
        } else {
            override_commands
        };
        let command = raw_command.into_iter().map(|s| sub_env(s)).collect();
        Self {
            image: config.image,
            name: config.name,
            runtime: config.cmd_line_config.runtime,
            volumes,
            envs,
            env_file: if config.cmd_line_config.env_file.is_empty() {
                None
            } else {
                Some(config.cmd_line_config.env_file)
            },
            ports,
            work_dir,
            remove: config.ephemeral,
            interactive: use_terminal,
            tty: use_terminal,
            detach: !use_terminal,
            command,
        }
    }
}

impl Profile {
    pub fn instantiate(&self, parsed_config: &CmdLineEngineConfig) -> Result<EngineConfig> {
        Ok(EngineConfig {
            image: self.image.to_owned(),
            name: None,
            cmd_line_config: parsed_config
                .resolve(&self.cmd_line_config_defaults)
                .context("Instantiate profile")?,
            ephemeral: false,
        })
    }
}

impl CmdLineEngineConfig {
    /// Baseline defaults
    fn base() -> Self {
        Self {
            mode: Some(OpMode::TmpOverlayGit),
            runtime: Some("krun".into()),
            terminal_connection_type: Some(TermConnectionType::Telnet),
            telnet_bind: Some("127.0.0.1:2323".into()),
            command: Some(String::new()),
            volumes: Some(Vec::new()),
            envs: Some(Vec::new()),
            env_file: Some(String::new()),
        }
    }
    /// Resolves command line engine config from, in priority ascending order:
    ///   Base config defined above
    ///   Passed in defaults (as parsed from profile config)
    ///   Values stored in current config struct (as parsed from command line)
    pub fn resolve(&self, defaults: &Self) -> Result<ResolvedCmdLineEngineConfig> {
        let result = Figment::new()
            .merge(Serialized::defaults(Self::base()))
            .admerge(Serialized::defaults(defaults))
            .admerge(Serialized::defaults(self))
            .extract()
            .context("Resolve command line engine config")?;
        Ok(result)
    }
}

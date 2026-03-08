use std::path::PathBuf;

use crate::config::parse_config;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Path to configuration yaml file. This file will be overlaid on default and contextual
    /// configurations. See also --no-contextual-config, --no-default-config
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Disable read of contextual configurations (agent-cage.yaml in parent folder tree)
    #[arg(long, global = true)]
    no_contextual_config: bool,

    /// Disable read of default configurations
    #[arg(long, global = true)]
    no_default_config: bool,

    #[command(subcommand)]
    sub_command: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Run an ephemeral sandbox that is removed at exit
    Run {
        #[command(flatten)]
        container_args: CreationArgs,
    },
}

/// Common args for creating a sandbox
#[derive(Parser, Debug)]
struct CreationArgs {
    #[command(flatten)]
    engine_config: CmdLineEngineConfig,
    /// Profile to run
    profile: String,
}

/// Defines a pair of structs:
/// - A "resolvable" struct with `Option` fields for command-line overrides
/// - A "stripped" struct with concrete types for the final resolved configuration
macro_rules! define_resolvable_struct {
    {
        $(#[$struct_meta:meta])*
        $(($(#[$stripped_struct_meta:meta])*))?
        $name:ident, $stripped_name:ident, {
            $(
                $(#[$field_meta:meta])* $field:ident: $type:ty
            ),* $(,)?
        }
    } => {
        $(#[$struct_meta])*
        #[skip_serializing_none]
        #[derive(clap::Parser, Debug, Serialize, Deserialize, Default)]
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field: Option<$type>,
            )*
        }

        $(#[$stripped_struct_meta])*
        #[derive(Debug, Serialize, Deserialize)]
        pub struct $stripped_name {
            $(
                pub $field: $type,
            )*
        }
    };
}

define_resolvable_struct! {
    /// Parts of engine config that could be overridden by command line args
    CmdLineEngineConfig, ResolvedCmdLineEngineConfig, {
        /// Mode of operation to handle working dir, default: tmp-overlay-git
        #[arg(short, long, value_enum)]
        mode: OpMode,
        /// Runtime to use with container engine, default: krun
        #[arg(short, long)]
        runtime: String,
        /// Terminal connection type, default: telnet
        #[arg(short, long, value_enum)]
        terminal_connection_type: TermConnectionType,
        /// Bind address for telnet terminal connection, default: 127.0.0.1:2323
        #[arg(short = 'T', long)]
        telnet_bind: String,
        /// Command to execute in container, overrides default as set by terminal connection type.
        /// Accepts single string, parsed as shell line
        #[arg(short = 'C', long)]
        command: String,
        /// Extra volumes to mount
        #[arg(short, long)]
        volumes: Vec<String>,
        /// Extra environment variables to set
        #[arg[short, long]]
        envs: Vec<String>,
        /// Read in a file of environment variables
        #[arg[short = 'E', long]]
        env_file: String,
    }
}

#[derive(Debug, ValueEnum, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TermConnectionType {
    /// Use direct pty allocation, be aware that pty sharing across krun boundary
    /// is unstable and may break your terminal
    Direct,
    /// Use telnet to connect terminal, requires busybox for the underlying image.
    /// This is recommended for krun. Also see --telnet-bind
    Telnet,
}

#[derive(Debug, ValueEnum, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OpMode {
    /// Do not set volume mount and working dir at all
    Disable,
    /// Mount and set current working dir to /work, with full read write access
    ReadWrite,
    /// Mount and set current working dir to /work, with read only access
    ReadOnly,
    /// Mount an ephemeral overlay on top of current working dir and use as /work.
    /// Note that any changes not manually saved elsewhere will be discarded once
    /// container is deleted.
    TmpOverlay,
    /// Mount an ephemeral overlay on .git of current working dir, and expose everything
    /// else to the sandbox. Requires to be run in the root of a git repo. Note that any
    /// local git operations will be discarded once container is deleted.
    TmpOverlayGit,
    /// Creates an isolated git repo for the agent to work on. This mode creates a nested
    /// git repo agent-cage-repo in current directory, then tracks it via remote "agent-cage-repo"
    /// and tracks its main branch via branch "agent-cage" in the current git repo. This 
    /// gives agents a completely isolated git repo without external references to work on
    /// and makes current repo able to push-to/pull-from agent repo on "agent-cage" branch
    IsolatedGitRepo
}

impl OpMode {
    pub fn to_volume_mounts(self) -> Vec<String> {
        match self {
            Self::Disable => Vec::new(),
            Self::ReadWrite => vec![
                ".:/work".into()
            ],
            Self::ReadOnly => vec![
                ".:/work:ro".into()
            ],
            Self::TmpOverlay => vec![
                ".:/work:O".into()
            ],
            Self::TmpOverlayGit => vec![
                ".:/work".into(),
                "./.git:/work/.git:O".into()
            ],
            Self::IsolatedGitRepo => vec![
                "./agent-cage-repo:/work".into()
            ]
        }
    }
    pub fn to_work_dir(self) -> Option<String> {
        if let Self::Disable = self {
            None
        } else {
            Some("/work".into())
        }
    }
}

impl Args {
    pub fn exec(self) -> Result<()> {
        let global_config = parse_config(
            self.config.as_ref(),
            !self.no_default_config,
            !self.no_contextual_config,
        )?;
        match &self.sub_command {
            SubCommand::Run {
                container_args:
                    CreationArgs {
                        engine_config,
                        profile,
                    },
            } => {
                let profile_obj = global_config
                    .get_profile(profile)
                    .context(format!("Look up profile {} in global config", profile))?;
                let final_engine_config = profile_obj
                    .instantiate(engine_config)
                    .context("Instantiate profile to get final engine config")?
                    .with_ephemeral()
                    .with_name(format!("agent-cage-{}", profile));
                final_engine_config.run_prepare().context("Run prepare scripts")?;
                let cmd_args = final_engine_config.into_cmd_args();
                println!("{:#?}", cmd_args)
            }
        }
        Ok(())
    }
}

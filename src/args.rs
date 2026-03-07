use std::path::PathBuf;

use crate::config::parse_config;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Path to config.yaml, omit to use embedded defaults
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
        /// Volume mount and set current working directory onto /work, default: true
        #[arg(short = 'w', long)]
        cwd: bool,
        /// Runtime to use with container engine, default: krun
        #[arg(short, long)]
        runtime: String,
        /// Terminal connection types, direct or telnet.
        /// Use telnet to connect to krun isolated terminal, since krun pty handling
        /// is still very inconsistent. Default: telnet
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

#[derive(Debug, ValueEnum, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TermConnectionType {
    Direct,
    Telnet,
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
                let cmd_args = final_engine_config.into_cmd_args();
                println!("{:#?}", cmd_args)
            }
        }
        Ok(())
    }
}

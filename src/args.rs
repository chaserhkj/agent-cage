use std::path::PathBuf;

use clap::{Parser,Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub sub_command: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// Run an ephemeral sandbox that is removed at exit
    Run {
        #[command(flatten)]
        container_args: ContainerArgs
    }
}


#[derive(Parser, Debug)]
pub struct ContainerArgs {
    /// Volume mount and set current working directory onto /work
    #[arg(short, long)]
    set_cwd: bool,

    profile: String,
}


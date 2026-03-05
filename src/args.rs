use clap::{Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub enum Args {
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


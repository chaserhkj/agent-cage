mod args;
mod config;

use args::Args;
use clap::Parser;
use anyhow::Result;

fn main() -> Result<()> {
    let args = Args::parse();
    let config = config::parse_config(args.config.as_ref())?;
    println!("{:?}", args);
    println!("{:?}", config);
    Ok(())
}

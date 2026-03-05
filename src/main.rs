mod args;
mod config;
mod engine;

use anyhow::Result;
use args::Args;
use clap::Parser;

fn main() -> Result<()> {
    let args = Args::parse();
    args.exec()?;
    Ok(())
}

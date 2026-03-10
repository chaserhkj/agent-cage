use anyhow::{anyhow, Context, Result};

/// Run child process in foreground and waiting for it to return
pub fn run_in_foreground<'a, I>(command: &str, args: I, ignore_rtn_code: bool) -> Result<()> 
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
mod cli;
mod config;
mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()?;

    Ok(())
}

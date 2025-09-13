#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod cli;
mod config;
mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    if user::get_user_name()? != "root" {
        eprintln!("Error: Need to run as root user");
        panic!();
    }
    cli::run()?;

    Ok(())
}

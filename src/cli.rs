use std::{fs, path::Path};

use anyhow::Result;
use clap::Parser;

use crate::utils::option_to_str;

#[derive(Parser)]
#[command(author,  about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Install the linux
    Install {
        /// Path of rootfs.
        rootfs: String,
        /// Target path
        target: String,
    },
}

pub fn run() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Install { rootfs, target } => {
            let rootfs = Path::new(rootfs.as_str());
            let target = Path::new(target.as_str());

            if !rootfs.exists() {
                eprintln!("Error: rootfs is not exists");
                std::process::exit(1);
            }
            if !target.exists() {
                if let Err(e) = fs::create_dir_all(target) {
                    eprintln!("Error: create target is falied");
                    std::process::exit(1);
                }
            }
            if target.is_file() {
                eprintln!("Error: target is file");
                std::process::exit(2);
            }
            let target_dir = target.read_dir()?;
            if target_dir.count() > 0 as usize {
                eprintln!("Error: target is not empty");
                std::process::exit(3);
            }
        }
    }

    Ok(())
}

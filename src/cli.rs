use std::{
    fs::{self, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
};

use anyhow::Result;
use clap::Parser;

use crate::utils::{compress::zip, option_to_str};

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
    /// Remove the linux
    Remove {
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
                if let Err(_) = fs::create_dir_all(target) {
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

            let extra = option_to_str(option_to_str(rootfs.extension()).to_str());

            println!("extracting rootfs to target");
            match extra {
                "zip" => {
                    println!("rootfs type is zip");
                    zip::extract(rootfs, target)?;
                }
                /*"xz" => {
                    println!("rootfs type is xz");
                }*/
                _ => {
                    eprintln!("Error");
                    std::process::exit(4);
                }
            }

            fs::remove_file(target.join("/etc/resolv.conf"))?;
            let mut resolv = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(target.join("/etc/resolv.conf"))?;
            resolv.write(
                r"nameserver 8.8.8.8
                nameserver 114.114.114.114"
                    .as_bytes(),
            )?;
            println!("install is done");
        }
        Commands::Remove { target } => {
            let target = Path::new(target.as_str());

            fs::set_permissions(target, PermissionsExt::from_mode(0777));
            let output = Command::new("chattr")
                .args(["-R", "-i", option_to_str(target.to_str())])
                .output()?;
            if !output.status.success() {
                eprintln!("Error: can't chattr to target");
            }

            fs::remove_dir_all(target)?;
        }
    }

    Ok(())
}

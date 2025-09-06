use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
};

use anyhow::Result;
use clap::Parser;

use crate::{
    config::Config,
    utils::{
        chroot::{self, unmount},
        compress::{tar as tar_tools, zip},
        option_to_str,
    },
};

const ASH: &str = "/bin/ash";
const BASH: &str = "/bin/bash";

#[derive(Parser)]
#[command(
    name = "alt",
    about = "Android chroot manager",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Login is unshare mode
    unshare: bool,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Install a rootfs into the specified target directory
    Install {
        /// Rootfs archive to install (tar, tar.gz, tar.xz, etc.)
        rootfs: String,

        /// Directory where the rootfs will be unpacked
        target: String,
    },

    /// Remove the chroot directory
    Remove {
        /// Directory to delete
        target: String,
    },

    /// Open an interactive shell inside the running chroot
    Login {
        /// Path to the chroot directory
        target: String,
    },

    /// Unmount all bind-mounts under the chroot directory
    Unmount {
        /// Path to the chroot directory
        target: String,
    },

    /// Bakup the choot directory
    Bakup {
        /// Path to the chroot directory
        target: String,
        /// Path to the output file
        output: String,
    },
}

pub fn run() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Install { rootfs, target } => {
            let rootfs = Path::new(rootfs.as_str());
            let target = Path::new(target.as_str());

            if !rootfs.exists() {
                eprintln!("Error: {} does not exist.", rootfs.display());
                std::process::exit(1);
            }
            if !target.exists() {
                if let Err(_) = fs::create_dir_all(target) {
                    eprintln!("Error: failed to create {}.", target.display());
                    std::process::exit(1);
                }
            }
            if target.is_file() {
                eprintln!("Error: {} is a file.", target.display());
                std::process::exit(2);
            }
            let target_dir = target.read_dir()?;
            if target_dir.count() > 0 as usize {
                eprintln!("Error: {} is not empty", target.display());
                std::process::exit(3);
            }

            let extra = option_to_str(option_to_str(rootfs.extension()).to_str());

            println!("extracting {} to {}", rootfs.display(), target.display());
            match extra {
                "zip" => {
                    println!("rootfs type is zip");
                    zip::extract(rootfs, target)?;
                }
                "xz" => {
                    println!("rootfs type is xz");
                    tar_tools::extract_tar(rootfs, target, tar_tools::Type::Xz)?;
                }
                "gz" => {
                    println!("rootfs type is gz");
                    tar_tools::extract_tar(rootfs, target, tar_tools::Type::Gz)?;
                }
                _ => {
                    eprintln!("Error");
                    std::process::exit(4);
                }
            }

            fs::remove_file(target.join("/etc/resolv.conf"))?;
            Config::init(target)?;
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

            let usergroup = include_str!("./useradd.sh");
            let mut usergroup_file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(target.join("tmp/usergroup.sh"))?;
            usergroup_file.write_all(usergroup.as_bytes())?;
            let envs = Config::read_config(target)?;
            let shell = if fs::exists(ASH)? { ASH } else { BASH };

            chroot::start(
                target,
                "/root",
                envs.envs,
                shell,
                "/tmp/usergroup.sh",
                args.unshare,
            )?;
            println!("install is done");
        }
        Commands::Remove { target } => {
            let target = Path::new(target.as_str());

            unmount(target.join("proc"))?;
            unmount(target.join("sys"))?;
            fs::set_permissions(target, PermissionsExt::from_mode(0777))?;
            let output = Command::new("chattr")
                .args(["-R", "-i", option_to_str(target.to_str())])
                .output()?;
            if !output.status.success() {
                eprintln!("Error: can't chattr to {}", target.display());
            }

            fs::remove_dir_all(target)?;
        }
        Commands::Unmount { target } => {
            let target = Path::new(target.as_str());
            unmount(target.join("proc"))?;
            unmount(target.join("sys"))?;
        }
        Commands::Login { target } => {
            let target = Path::new(target.as_str());
            let home = Path::new("/root");
            let ksu_susfs = Path::new("/data/adb/ksu/bin/ksu_susfs");

            let config = Config::read_config(target)?;
            let mut envs = vec![
                ("USER".to_string(), config.user),
                ("HOME".to_string(), config.home),
            ];
            envs.extend(config.envs);

            if ksu_susfs.exists() {
                Command::new(ksu_susfs)
                    .arg("hide_sus_mnts_for_all_procs")
                    .arg("0")
                    .output()?;
            }

            chroot::start(
                target,
                home,
                envs,
                &config.shell.main,
                &config.shell.args,
                args.unshare,
            )?;

            return Err(std::io::Error::last_os_error().into());
        }
        Commands::Bakup { target, output } => {
            let target = Path::new(target.as_str());
            let output = Path::new(output.as_str());

            println!("bakuping");

            zip::zip(target, output)?;

            println!("bakup is completed");
        }
    }

    Ok(())
}

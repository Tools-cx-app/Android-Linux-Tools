use std::{
    ffi::CString,
    fs,
    io::Write,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
    process::Command,
    ptr,
};

use anyhow::Result;
use clap::Parser;

use crate::utils::{compress::zip, mount, option_to_str};

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
    /// Login the linux
    Login {
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
                eprintln!("Error: can't chattr to {}", target.display());
            }

            fs::remove_dir_all(target)?;
        }
        Commands::Login { target } => {
            let target = Path::new(target.as_str());
            let proc = target.join("/proc");
            let dev = target.join("/dev");

            mount("sysfs", "sys", target.join("/sys"), 0)?;
            mount("proc", "proc", target.join("/proc"), 0)?;
            mount(
                "tmpfs",
                "tmpfs",
                target.join("/tmp"),
                libc::MS_NOSUID | libc::MS_NODEV,
            )?;
            mount("none", "/dev", target.join("dev"), libc::MS_BIND)?;

            let dev_dirs = ["/dev/shm", "/dev/pts", "/dev/net"];
            for dir in &dev_dirs {
                if !Path::new(dir).exists() {
                    fs::create_dir_all(dir)?;
                }
            }

            mount(
                "tmpfs",
                "tmpfs",
                Path::new("/dev/shm"),
                libc::MS_NOSUID | libc::MS_NODEV,
            )?;
            mount("devpts", "devpts", dev.join("/pts"), libc::MS_NOEXEC)?;
            mount("none", "/dev/shm", dev.join("/shm"), libc::MS_BIND)?;
            mount("none", "/dev/pts", dev.join("/pts"), libc::MS_BIND)?;

            let links = [
                ("/proc/self/fd", "/dev/fd"),
                ("/proc/self/fd/0", "/dev/stdin"),
                ("/proc/self/fd/1", "/dev/stdout"),
                ("/proc/self/fd/2", "/dev/stderr"),
            ];

            for (src, dst) in &links {
                if !Path::new(dst).exists() {
                    symlink(src, dst)?;
                }
            }

            if !Path::new("/dev/tty0").exists() {
                symlink("/dev/null", "/dev/tty0")?;
            }

            let tun_path = dev.join("/net/tun");
            if tun_path.exists() {
                return Ok(());
            }

            fs::create_dir_all(tun_path.parent().unwrap())?;

            unsafe {
                if libc::mknod(
                    CString::new("/dev/net/tun")?.as_ptr(),
                    libc::S_IFCHR | 0o666,
                    libc::makedev(10, 200),
                ) != 0
                {
                    return Err(std::io::Error::last_os_error().into());
                }

                if libc::chroot(CString::new(option_to_str(target.to_str()))?.as_ptr()) != 0 {
                    return Err(std::io::Error::last_os_error().into());
                }

                if libc::execvp(
                    CString::new("/bin/bash")?.as_ptr(),
                    vec![
                        CString::new("/bin/bash")?.as_ptr(),
                        CString::new("-l")?.as_ptr(),
                        ptr::null(),
                    ]
                    .as_ptr(),
                ) != 0
                {
                    return Err(std::io::Error::last_os_error().into());
                }
            }
        }
    }

    Ok(())
}

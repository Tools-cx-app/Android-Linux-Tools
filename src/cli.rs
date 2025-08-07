use std::{
    ffi::CString,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    ptr,
};

use anyhow::Result;
use clap::Parser;

use crate::utils::{
    chroot::{mount, set_envs, unmount},
    compress::{tar as tar_tools, zip},
    option_to_str,
};

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
    /// Unmount the linux
    Unmount {
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
            let dev_target = target.join("dev");
            let mut usergroup_file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(target.join("tmp/usergroup.sh"))?;
            usergroup_file.write_all(usergroup.as_bytes())?;
            fs::create_dir_all(&dev_target)?;

            unsafe {
                let null_path = dev_target.join("null");
                if !null_path.exists() {
                    libc::mknod(
                        CString::new(null_path.to_str().unwrap())?.as_ptr(),
                        libc::S_IFCHR | 0o666,
                        libc::makedev(1, 3),
                    );
                }

                let tty_path = dev_target.join("tty");
                if !tty_path.exists() {
                    libc::mknod(
                        CString::new(tty_path.to_str().unwrap())?.as_ptr(),
                        libc::S_IFCHR | 0o666,
                        libc::makedev(5, 0),
                    );
                }

                let tun_dir = dev_target.join("net");
                fs::create_dir_all(&tun_dir)?;
                let tun_path = tun_dir.join("tun");
                if !tun_path.exists() {
                    libc::mknod(
                        CString::new(tun_path.to_str().unwrap())?.as_ptr(),
                        libc::S_IFCHR | 0o666,
                        libc::makedev(10, 200),
                    );
                }
            }

            mount("sysfs", "sys", target.join("sys"), 0)?;
            mount("proc", "proc", target.join("proc"), 0)?;

            unsafe {
                if libc::chroot(CString::new(target.to_str().unwrap())?.as_ptr()) != 0 {
                    return Err(std::io::Error::last_os_error().into());
                }

                libc::chdir(CString::new("/")?.as_ptr());

                let envs = [
                    ("PATH", "/usr/local/bin:/usr/bin:/bin"),
                    ("TERM", "xterm-256color"),
                    ("HOME", "/root"),
                    ("USER", "root"),
                    ("SHELL", "/bin/bash"),
                    ("LANG", "C.UTF-8"),
                ];
                set_envs(&envs)?;

                let bash = CString::new("/tmp/usergroup.sh")?;
                let argv = [bash.as_ptr(), ptr::null()];
                libc::execvp(bash.as_ptr(), argv.as_ptr());
            }

            println!("install is done");
        }
        Commands::Remove { target } => {
            let target = Path::new(target.as_str());

            unmount(target);
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
            unmount(target)?;
        }
        Commands::Login { target } => {
            let target = Path::new(target.as_str());

            let dev_target = target.join("dev");
            fs::create_dir_all(&dev_target)?;

            unsafe {
                let null_path = dev_target.join("null");
                if !null_path.exists() {
                    libc::mknod(
                        CString::new(null_path.to_str().unwrap())?.as_ptr(),
                        libc::S_IFCHR | 0o666,
                        libc::makedev(1, 3),
                    );
                }

                let tty_path = dev_target.join("tty");
                if !tty_path.exists() {
                    libc::mknod(
                        CString::new(tty_path.to_str().unwrap())?.as_ptr(),
                        libc::S_IFCHR | 0o666,
                        libc::makedev(5, 0),
                    );
                }

                let tun_dir = dev_target.join("net");
                fs::create_dir_all(&tun_dir)?;
                let tun_path = tun_dir.join("tun");
                if !tun_path.exists() {
                    libc::mknod(
                        CString::new(tun_path.to_str().unwrap())?.as_ptr(),
                        libc::S_IFCHR | 0o666,
                        libc::makedev(10, 200),
                    );
                }
            }

            mount("sysfs", "sys", target.join("sys"), 0)?;
            mount("proc", "proc", target.join("proc"), 0)?;

            unsafe {
                if libc::chroot(CString::new(target.to_str().unwrap())?.as_ptr()) != 0 {
                    return Err(std::io::Error::last_os_error().into());
                }

                libc::chdir(CString::new("/")?.as_ptr());

                let envs = [
                    ("PATH", "/usr/local/bin:/usr/bin:/bin"),
                    ("TERM", "xterm-256color"),
                    ("HOME", "/root"),
                    ("USER", "root"),
                    ("SHELL", "/bin/bash"),
                    ("LANG", "C.UTF-8"),
                ];
                set_envs(&envs)?;

                let bash = CString::new("/bin/bash")?;
                let login_flag = CString::new("-l")?;
                let argv = [bash.as_ptr(), login_flag.as_ptr(), ptr::null()];
                libc::execvp(bash.as_ptr(), argv.as_ptr());
            }

            return Err(std::io::Error::last_os_error().into());
        }
    }

    Ok(())
}

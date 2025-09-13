use std::{ffi::CString, fs, path::Path, ptr};

use anyhow::Result;

use crate::utils::option_to_str;

pub fn mount(fs_type: &str, source: &str, target: impl AsRef<Path>, flags: u64) -> Result<()> {
    let target = target.as_ref();
    fs::create_dir_all(target)?;

    let fs_type_cstr = CString::new(fs_type)?;
    let source_cstr = CString::new(source)?;
    let target_cstr = CString::new(option_to_str(target.to_str()))?;

    unsafe {
        #[cfg(target_arch = "arm")]
        {
            if libc::mount(
                source_cstr.as_ptr(),
                target_cstr.as_ptr(),
                fs_type_cstr.as_ptr(),
                flags as u32,
                std::ptr::null(),
            ) != 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if libc::mount(
                source_cstr.as_ptr(),
                target_cstr.as_ptr(),
                fs_type_cstr.as_ptr(),
                flags,
                std::ptr::null(),
            ) != 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
        }
    }
    Ok(())
}

pub fn mount_bind(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<()> {
    let target = target.as_ref();
    let source = source.as_ref();
    let source_cstr = CString::new(source.to_str().unwrap_or_default())?;
    let target_cstr = CString::new(target.to_str().unwrap_or_default())?;

    unsafe {
        if libc::mount(
            source_cstr.as_ptr(),
            target_cstr.as_ptr(),
            std::ptr::null(),
            libc::MS_BIND,
            std::ptr::null(),
        ) != 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    Ok(())
}

pub fn unmount(target: impl AsRef<Path>) -> Result<()> {
    let target = target.as_ref();
    fs::create_dir_all(target)?;

    let target_cstr = CString::new(option_to_str(target.to_str()))?;

    unsafe {
        if libc::umount(target_cstr.as_ptr()) != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    Ok(())
}

pub unsafe fn set_envs(vars: Vec<(String, String)>) -> Result<()> {
    for (k, v) in vars {
        let key = CString::new(k)?;
        let val = CString::new(v)?;
        if unsafe { libc::setenv(key.as_ptr(), val.as_ptr(), 1) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    Ok(())
}

pub fn start(
    target: impl AsRef<Path>,
    home: impl AsRef<Path>,
    envs: Vec<(String, String)>,
    bash: &str,
    args: &str,
    unshare: bool,
) -> Result<()> {
    let target = target.as_ref();
    let home = home.as_ref().to_string_lossy();
    let dev_target = target.join("dev");

    fs::create_dir_all(&dev_target)?;

    if unshare {
        unsafe {
            if libc::unshare(libc::CLONE_NEWNS) != 0 {
                return Err(std::io::Error::last_os_error().into());
            }

            let root = CString::new("/")?;
            if libc::mount(
                ptr::null(),
                root.as_ptr(),
                ptr::null(),
                libc::MS_SLAVE | libc::MS_REC,
                ptr::null(),
            ) != 0
            {
                return Err(std::io::Error::last_os_error().into());
            }

            let flags = libc::CLONE_NEWPID | libc::CLONE_NEWUTS | libc::CLONE_NEWIPC;
            if libc::unshare(flags) != 0 {
                return Err(std::io::Error::last_os_error().into());
            }
        }
    }
    let _ = mount("sysfs", "sys", target.join("sys"), 0);
    let _ = mount("proc", "proc", target.join("proc"), 0);
    let _ = mount_bind("/dev/", target.join("dev"));

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

    unsafe {
        if libc::chroot(CString::new(target.to_str().unwrap())?.as_ptr()) != 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        libc::chdir(CString::new(&*home)?.as_ptr());

        set_envs(envs)?;

        let bash = CString::new(bash)?;
        let argv_ptr = [args.as_ptr(), std::ptr::null()];
        libc::execvp(bash.as_ptr(), argv_ptr.as_ptr());
    }

    Ok(())
}

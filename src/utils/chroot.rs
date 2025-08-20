use std::{ffi::CString, fs, path::Path};

use anyhow::Result;

use crate::utils::option_to_str;

pub fn mount(fs_type: &str, source: &str, target: impl AsRef<Path>, flags: u64) -> Result<()> {
    let target = target.as_ref();
    fs::create_dir_all(target)?;

    let fs_type_cstr = CString::new(fs_type)?;
    let source_cstr = CString::new(source)?;
    let target_cstr = CString::new(option_to_str(target.to_str()))?;

    unsafe {
        if libc::mount(
            source_cstr.as_ptr(),
            target_cstr.as_ptr(),
            fs_type_cstr.as_ptr(),
            flags as u64,
            std::ptr::null(),
        ) != 0
        {
            return Err(std::io::Error::last_os_error().into());
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
) -> Result<()> {
    let target = target.as_ref();
    let home = home.as_ref().to_string_lossy();
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

    let _ = mount("sysfs", "sys", target.join("sys"), 0);
    let _ = mount("proc", "proc", target.join("proc"), 0);
    let _ = mount_bind("/dev/", target.join("dev"));

    unsafe {
        if libc::chroot(CString::new(target.to_str().unwrap())?.as_ptr()) != 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        libc::chdir(CString::new(&*home)?.as_ptr());

        set_envs(envs)?;

        let bash = CString::new(bash)?;
        let argv = [args.as_ptr(), std::ptr::null()];
        libc::execvp(bash.as_ptr(), argv.as_ptr());
    }

    Ok(())
}

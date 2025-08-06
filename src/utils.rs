use std::{ffi::CString, fs, path::Path};

use anyhow::Result;

pub fn option_to_str<T: Default>(option: Option<T>) -> T {
    option.unwrap_or_default()
}

pub mod compress {
    pub mod zip {
        use std::{
            fs::{self, File},
            io,
            path::Path,
        };

        use anyhow::Result;
        use zip::ZipArchive;

        pub fn extract<T: AsRef<Path>>(path: T, output: T) -> Result<()> {
            let path = path.as_ref();
            let zipfile = File::open(path)?;
            let mut zip = ZipArchive::new(zipfile)?;

            for i in 0..zip.len() {
                let mut file = zip.by_index(i)?;
                let outpath = output.as_ref().join(file.mangled_name());

                if file.is_dir() {
                    fs::create_dir_all(outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            fs::create_dir_all(p)?;
                        }

                        let mut out = File::create(outpath)?;
                        io::copy(&mut file, &mut out)?;
                    }
                }
            }

            Ok(())
        }
    }

    pub mod tar {
        use std::{fs::File, path::Path};

        use anyhow::Result;

        pub enum Type {
            Xz,
            Gz,
        }

        pub fn extract_tar<T: AsRef<Path>>(path: T, target: T, tar_type: Type) -> Result<()> {
            let path = path.as_ref();
            let target = target.as_ref();
            let file = File::open(path)?;
            let boxed: Box<dyn std::io::Read> = match tar_type {
                Type::Xz => Box::new(xz2::read::XzDecoder::new(file)),
                Type::Gz => Box::new(flate2::read::GzDecoder::new(file)),
            };

            let mut archive = tar::Archive::new(boxed);
            archive.unpack(target)?;

            Ok(())
        }
    }
}

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

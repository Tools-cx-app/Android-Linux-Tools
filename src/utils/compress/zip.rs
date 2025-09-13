use std::{
    fs::{self, File},
    io,
    path::Path,
};

use anyhow::Result;
use zip::ZipArchive;
use zip_extensions::zip_create_from_directory;

pub fn extract<T: AsRef<Path>>(path: T, output: T) -> Result<()> {
    let path = path.as_ref();
    let zipfile = File::open(path)?;
    let mut zip = ZipArchive::new(zipfile)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let outpath = output.as_ref().join(file.mangled_name());

        if file.is_dir() {
            fs::create_dir_all(outpath)?;
        } else if let Some(p) = outpath.parent() {
            if !p.exists() {
                fs::create_dir_all(p)?;
            }

            let mut out = File::create(outpath)?;
            io::copy(&mut file, &mut out)?;
        }
    }

    Ok(())
}

pub fn zip<T: AsRef<Path>>(target: T, output: T) -> Result<()> {
    let target = target.as_ref();
    let output = output.as_ref();
    zip_create_from_directory(&output.to_path_buf(), &target.to_path_buf())?;
    Ok(())
}

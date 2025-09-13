use std::{fs::File, path::Path};

use anyhow::Result;

pub enum Type {
    Xz,
    Gz,
}

pub fn extract_tar<T: AsRef<Path>>(path: T, target: T, tar_type: &Type) -> Result<()> {
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

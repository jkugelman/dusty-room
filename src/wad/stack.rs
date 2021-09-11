use std::{io, path::Path};

use super::file::WadFile;
use super::file::WadType;

/// A base IWAD plus zero or more PWAD patches layered on top.
pub struct WadStack {
    files: Vec<WadFile>,
}

impl WadStack {
    pub fn new(file: impl AsRef<Path>) -> io::Result<Self> {
        let file = file.as_ref();
        let wad = WadFile::open(file)?;

        match wad.wad_type() {
            WadType::Initial => Ok(Self { files: vec![wad] }),
            WadType::Patch => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} not an IWAD", file.display()),
            )),
        }
    }

    pub fn add(&mut self, file: impl AsRef<Path>) -> io::Result<()> {
        let file = file.as_ref();
        let wad = WadFile::open(file)?;

        match wad.wad_type() {
            WadType::Patch => {
                self.files.push(wad);
                Ok(())
            }
            WadType::Initial => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} not a PWAD", file.display()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn iwad_then_pwads() -> io::Result<()> {
        let mut wad = WadStack::new(test_path("doom.wad"))?;
        wad.add(test_path("killer.wad"))?;

        // Can't add an IWAD as a patch.
        let mut wad = WadStack::new(test_path("doom.wad"))?;
        assert!(wad.add(test_path("doom.wad")).is_err());

        // Can't start with a PWAD.
        assert!(WadStack::new(test_path("killer.wad")).is_err());

        Ok(())
    }

    fn test_path(path: impl AsRef<Path>) -> PathBuf {
        Path::new("test").join(path)
    }
}

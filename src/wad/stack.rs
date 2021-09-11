use std::sync::Arc;
use std::{io, path::Path};

use super::LumpBlock;
use super::WadFile;
use super::WadType;

/// An IWAD plus zero or more PWADs layered on top.
pub struct WadStack {
    iwad: WadFile,
    pwads: Vec<WadFile>,
}

impl WadStack {
    /// Creates a stack starting with a IWAD such as `doom.wad`.
    pub fn iwad(file: impl AsRef<Path>) -> io::Result<Self> {
        let file = file.as_ref();
        let wad = WadFile::open(file)?;

        match wad.wad_type() {
            WadType::Iwad => Ok(Self {
                iwad: wad,
                pwads: vec![],
            }),
            WadType::Pwad => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} not an IWAD", file.display()),
            )),
        }
    }

    /// Adds a PWAD that overrides files earlier in the stack.
    pub fn pwad(mut self, file: impl AsRef<Path>) -> io::Result<Self> {
        self.add_pwad(file)?;
        Ok(self)
    }

    /// Adds a PWAD that overrides files earlier in the stack.
    pub fn add_pwad(&mut self, file: impl AsRef<Path>) -> io::Result<()> {
        let file = file.as_ref();
        let wad = WadFile::open(file)?;

        match wad.wad_type() {
            WadType::Pwad => {
                self.pwads.push(wad);
                Ok(())
            }
            WadType::Iwad => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} not a PWAD", file.display()),
            )),
        }
    }

    /// Retrieves a named lump. The name must be unique.
    ///
    /// Lumps in later files override lumps from earlier ones.
    pub fn lump(&self, name: &str) -> Option<Arc<[u8]>> {
        for pwad in self.pwads.iter().rev() {
            if let Some(lump) = pwad.lump(name) {
                return Some(lump);
            }
        }

        self.iwad.lump(name)
    }

    /// Retrieves a block of `size` lumps following a named marker. The marker lump
    /// is not included in the result.
    ///
    /// Blocks in later files override entire blocks from earlier files.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kdoom::WadStack;
    ///
    /// let wad = WadStack::iwad("doom.wad")?.pwad("killer.wad")?;
    /// let map = wad.lumps_after("E1M5", 10);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn lumps_after(&self, start: &str, size: usize) -> Option<LumpBlock> {
        for pwad in self.pwads.iter().rev() {
            if let Some(lumps) = pwad.lumps_after(start, size) {
                return Some(lumps);
            }
        }

        self.iwad.lumps_after(start, size)
    }

    /// Retrieves a block of lumps between start and end markers. The marker lumps
    /// are not included in the result.
    ///
    /// Blocks in later wads override entire blocks from earlier files.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kdoom::WadStack;
    ///
    /// let wad = WadStack::iwad("doom2.wad")?.pwad("biotech.wad")?;
    /// let sprites = wad.lumps_between("SS_START", "SS_END");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn lumps_between(&self, start: &str, end: &str) -> Option<LumpBlock> {
        for pwad in self.pwads.iter().rev() {
            if let Some(lumps) = pwad.lumps_between(start, end) {
                return Some(lumps);
            }
        }

        self.iwad.lumps_between(start, end)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn iwad_then_pwads() {
        // IWAD + PWAD = success.
        WadStack::iwad(test_path("doom.wad"))
            .unwrap()
            .pwad(test_path("killer.wad"))
            .unwrap();

        // IWAD + IWAD = error.
        let mut wad = WadStack::iwad(test_path("doom.wad")).unwrap();
        assert!(wad.add_pwad(test_path("doom2.wad")).is_err());

        // Can't start with a PWAD.
        assert!(WadStack::iwad(test_path("killer.wad")).is_err());
    }

    #[test]
    fn layering() {
        let mut wad = WadStack::iwad(test_path("doom2.wad")).unwrap();
        assert_eq!(wad.lump("DEMO3").unwrap().len(), 17898);
        assert_eq!(
            wad.lumps_after("MAP01", 10)
                .unwrap()
                .iter()
                .map(|(name, lump)| -> (&str, usize) { (name, lump.len()) })
                .collect::<Vec<_>>(),
            [
                ("THINGS", 690),
                ("LINEDEFS", 5180),
                ("SIDEDEFS", 15870),
                ("VERTEXES", 1532),
                ("SEGS", 7212),
                ("SSECTORS", 776),
                ("NODES", 5404),
                ("SECTORS", 1534),
                ("REJECT", 436),
                ("BLOCKMAP", 6418),
            ],
        );
        assert_eq!(wad.lumps_between("S_START", "S_END").unwrap().len(), 1381);

        wad.add_pwad(test_path("biotech.wad")).unwrap();
        assert_eq!(wad.lump("DEMO3").unwrap().len(), 9490);
        assert_eq!(
            wad.lumps_after("MAP01", 10)
                .unwrap()
                .iter()
                .map(|(name, lump)| -> (&str, usize) { (name, lump.len()) })
                .collect::<Vec<_>>(),
            [
                ("THINGS", 1050),
                ("LINEDEFS", 5040),
                ("SIDEDEFS", 17400),
                ("VERTEXES", 1372),
                ("SEGS", 7536),
                ("SSECTORS", 984),
                ("NODES", 6860),
                ("SECTORS", 2184),
                ("REJECT", 882),
                ("BLOCKMAP", 4362),
            ],
        );
        assert_eq!(wad.lumps_between("S_START", "S_END").unwrap().len(), 1381);
        assert_eq!(wad.lumps_between("SS_START", "SS_END").unwrap().len(), 263);
    }

    fn test_path(path: impl AsRef<Path>) -> PathBuf {
        Path::new("test").join(path)
    }
}

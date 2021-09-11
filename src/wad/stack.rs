use std::sync::Arc;
use std::{io, path::Path};

use super::LumpBlock;
use super::WadFile;
use super::WadType;

/// A base IWAD plus zero or more PWAD patches layered on top.
pub struct WadStack {
    base: WadFile,
    patches: Vec<WadFile>,
}

impl WadStack {
    /// Creates a stack starting with a base IWAD such as `doom.wad`.
    pub fn base(file: impl AsRef<Path>) -> io::Result<Self> {
        let file = file.as_ref();
        let wad = WadFile::open(file)?;

        match wad.wad_type() {
            WadType::Initial => Ok(Self {
                base: wad,
                patches: vec![],
            }),
            WadType::Patch => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} not an IWAD", file.display()),
            )),
        }
    }

    /// Adds a PWAD patch that overrides wads earlier in the stack.
    pub fn patch(mut self, file: impl AsRef<Path>) -> io::Result<Self> {
        self.add_patch(file)?;
        Ok(self)
    }

    /// Adds a PWAD patch that overrides wads earlier in the stack.
    pub fn add_patch(&mut self, file: impl AsRef<Path>) -> io::Result<()> {
        let file = file.as_ref();
        let wad = WadFile::open(file)?;

        match wad.wad_type() {
            WadType::Patch => {
                self.patches.push(wad);
                Ok(())
            }
            WadType::Initial => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} not a PWAD", file.display()),
            )),
        }
    }

    /// Retrieves a named lump. The name must be unique.
    ///
    /// Lumps in patch wads override the base wad.
    pub fn lump(&self, name: &str) -> io::Result<Option<Arc<[u8]>>> {
        for patch in self.patches.iter().rev() {
            if let Ok(Some(lump)) = patch.lump(name) {
                return Ok(Some(lump));
            }
        }

        self.base.lump(name)
    }

    /// Retrieves a block of `size` lumps following a named marker. The marker lump
    /// is not included in the result.
    ///
    /// Blocks in patch wads override the entire block from the base wad.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kdoom::WadStack;
    ///
    /// let wad = WadStack::base("doom.wad")?.patch("killer.wad")?;
    /// let map = wad.lumps_after("E1M5", 10)?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn lumps_after(&self, start: &str, size: usize) -> io::Result<Option<LumpBlock>> {
        for patch in self.patches.iter().rev() {
            if let Ok(Some(lumps)) = patch.lumps_after(start, size) {
                return Ok(Some(lumps));
            }
        }

        self.base.lumps_after(start, size)
    }

    /// Retrieves a block of lumps between start and end markers. The marker lumps
    /// are not included in the result.
    ///
    /// Blocks in patch wads override the entire block from the base wad.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kdoom::WadStack;
    ///
    /// let wad = WadStack::base("doom2.wad")?.patch("biotech.wad")?;
    /// let sprites = wad.lumps_between("SS_START", "SS_END")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn lumps_between(&self, start: &str, end: &str) -> io::Result<Option<LumpBlock>> {
        for patch in self.patches.iter().rev() {
            if let Ok(Some(lumps)) = patch.lumps_between(start, end) {
                return Ok(Some(lumps));
            }
        }

        self.base.lumps_between(start, end)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn iwad_then_pwads() -> io::Result<()> {
        // IWAD + PWAD = success.
        WadStack::base(test_path("doom.wad"))?.patch(test_path("killer.wad"))?;

        // Can't add an IWAD as a patch.
        let mut wad = WadStack::base(test_path("doom.wad"))?;
        assert!(wad.add_patch(test_path("doom.wad")).is_err());

        // Can't start with a PWAD.
        assert!(WadStack::base(test_path("killer.wad")).is_err());

        Ok(())
    }

    #[test]
    fn layering() -> io::Result<()> {
        let mut wad = WadStack::base(test_path("doom2.wad"))?;
        assert_eq!(wad.lump("DEMO3")?.unwrap().len(), 17898);
        assert_eq!(
            wad.lumps_after("MAP01", 10)?.unwrap()
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
        assert_eq!(wad.lumps_between("S_START", "S_END")?.unwrap().len(), 1381);

        wad.add_patch(test_path("biotech.wad"))?;
        assert_eq!(wad.lump("DEMO3")?.unwrap().len(), 9490);
        assert_eq!(
            wad.lumps_after("MAP01", 10)?.unwrap()
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
        assert_eq!(wad.lumps_between("S_START", "S_END")?.unwrap().len(), 1381);
        assert_eq!(wad.lumps_between("SS_START", "SS_END")?.unwrap().len(), 263);

        Ok(())
    }

    fn test_path(path: impl AsRef<Path>) -> PathBuf {
        Path::new("test").join(path)
    }
}

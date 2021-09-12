use std::{io, path::Path, sync::Arc};

use crate::{Lump, WadFile, WadType};

/// A stack of WAD files layered on top of each other, with later files
/// overlaying earlier ones.
///
/// A `Wad` usually consists of an IWAD overlaid with zero or more PWADs, a
/// convention which is enforced by the [`iwad`] and [`pwad`] builder methods.
/// There are a set of unchecked methods if you want to ignore this convention.
///
/// [`iwad`]: Wad::iwad
/// [`pwad`]: Wad::pwad
#[derive(Clone, Debug)]
#[must_use]
pub struct Wad {
    files: Vec<Arc<WadFile>>,
}

impl Wad {
    /// Opens an IWAD such as `doom.wad`.
    pub fn iwad(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::initial(Arc::new(WadFile::open(path.as_ref())?))
    }

    /// Overlays a PWAD.
    pub fn pwad(&self, path: impl AsRef<Path>) -> io::Result<Self> {
        self.patch(Arc::new(WadFile::open(path.as_ref())?))
    }

    /// Creates a `Wad` without a starting IWAD. Use this if you want to bypass
    /// IWAD/PWAD type checking.
    pub fn empty_unchecked() -> Self {
        Self { files: Vec::new() }
    }

    /// Creates a `Wad` starting with an already opened [`WadFile`], which must be
    /// an IWAD.
    pub fn initial(file: Arc<WadFile>) -> io::Result<Self> {
        match file.wad_type() {
            WadType::Iwad => Ok(Self::initial_unchecked(file)),
            WadType::Pwad => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} not an IWAD", file),
            )),
        }
    }

    /// Creates a `Wad` starting with an already opened [`WadFile`], which need not
    /// be an IWAD. Use this if you want to bypass IWAD/PWAD type checking.
    pub fn initial_unchecked(file: Arc<WadFile>) -> Self {
        Self::empty_unchecked().patch_unchecked(file)
    }

    /// Overlays an already opened [`WadFile`], which must be a PWAD.
    pub fn patch(&self, file: Arc<WadFile>) -> io::Result<Self> {
        match file.wad_type() {
            WadType::Pwad => Ok(self.patch_unchecked(file)),
            WadType::Iwad => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} not a PWAD", file),
            )),
        }
    }

    /// Overlays an already opened [`WadFile`], which need not be a PWAD. Use this
    /// if you want to bypass IWAD/PWAD type checking.
    pub fn patch_unchecked(&self, file: Arc<WadFile>) -> Self {
        let mut clone = self.clone();
        clone.files.push(file);
        clone
    }

    /// Retrieves a unique lump by name.
    ///
    /// Lumps in later files override lumps from earlier ones.
    pub fn lump(&self, name: &str) -> Option<&Lump> {
        self.files.iter().rev().find_map(|file| file.lump(name))
    }

    /// Retrieves a block of `size` lumps starting with a unique named marker. The
    /// marker lump is included in the result.
    ///
    /// Blocks in later files override entire blocks from earlier files.
    pub fn lumps_after(&self, start: &str, size: usize) -> Option<&[Lump]> {
        self.files
            .iter()
            .rev()
            .find_map(|file| file.lumps_after(start, size))
    }

    /// Retrieves a block of lumps between start and end markers. The marker lumps
    /// are included in the result.
    ///
    /// Blocks in later wads override entire blocks from earlier files.
    pub fn lumps_between(&self, start: &str, end: &str) -> Option<&[Lump]> {
        self.files
            .iter()
            .rev()
            .find_map(|file| file.lumps_between(start, end))
    }
}

impl PartialEq for Wad {
    fn eq(&self, other: &Self) -> bool {
        let self_ptrs = self.files.iter().map(Arc::as_ptr);
        let other_ptrs = other.files.iter().map(Arc::as_ptr);
        self_ptrs.eq(other_ptrs)
    }
}

impl Eq for Wad {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

    #[test]
    fn iwad_then_pwads() {
        // IWAD + PWAD = success.
        let _ = Wad::iwad(DOOM_WAD_PATH)
            .unwrap()
            .pwad(KILLER_WAD_PATH)
            .unwrap();

        // IWAD + IWAD = error.
        let wad = Wad::iwad(DOOM_WAD_PATH).unwrap();
        assert_matches!(wad.pwad(DOOM2_WAD_PATH), Err(_));

        // Can't start with a PWAD.
        assert_matches!(Wad::iwad(KILLER_WAD_PATH), Err(_));
    }

    #[test]
    fn layering() {
        assert_eq!(DOOM2_WAD.lump("DEMO3").unwrap().size(), 17898);
        assert_eq!(
            DOOM2_WAD
                .lumps_after("MAP01", 11)
                .unwrap()
                .iter()
                .map(|lump| (lump.name.as_str(), lump.size()))
                .collect::<Vec<_>>(),
            [
                ("MAP01", 0),
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
        assert_eq!(
            DOOM2_WAD.lumps_between("S_START", "S_END").unwrap().len(),
            1383
        );

        let wad = DOOM2_WAD.patch_unchecked(BIOTECH_WAD_FILE.clone());
        assert_eq!(wad.lump("DEMO3").unwrap().size(), 9490);
        assert_eq!(
            wad.lumps_after("MAP01", 11)
                .unwrap()
                .iter()
                .map(|lump| (lump.name.as_str(), lump.size()))
                .collect::<Vec<_>>(),
            [
                ("MAP01", 0),
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
        assert_eq!(wad.lumps_between("S_START", "S_END").unwrap().len(), 1383);
        assert_eq!(wad.lumps_between("SS_START", "SS_END").unwrap().len(), 265);
    }

    #[test]
    fn no_type_checking() {
        // Nonsensical ordering.
        let silly_wad = Wad::empty_unchecked()
            .patch_unchecked(KILLER_WAD_FILE.clone())
            .patch_unchecked(DOOM2_WAD_FILE.clone())
            .patch_unchecked(DOOM_WAD_FILE.clone())
            .patch_unchecked(BIOTECH_WAD_FILE.clone());

        assert_matches!(silly_wad.lump("E1M1"), Some(_));
        assert_matches!(silly_wad.lump("MAP01"), Some(_));
    }

    // Make sure Wad is Send and Sync.
    trait IsSendAndSync: Send + Sync {}
    impl IsSendAndSync for Wad {}
}

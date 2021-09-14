use std::{path::Path, sync::Arc};

use crate::wad::{self, Lump, WadFile, WadType};

/// A stack of WAD files layered on top of each other, with later files overlaying earlier ones.
///
/// A `Wad` usually consists of an IWAD overlaid with zero or more PWADs, a convention which is
/// enforced by the [`iwad`] and [`pwad`] builder methods. There are a set of unchecked methods if
/// you want to ignore this convention.
///
/// [`iwad`]: Wad::iwad
/// [`pwad`]: Wad::pwad
#[derive(Clone, Debug)]
#[must_use]
pub struct Wad {
    initial: Arc<WadFile>,
    patches: Vec<Arc<WadFile>>,
}

impl Wad {
    /// Opens an IWAD such as `doom.wad`.
    pub fn iwad(path: impl AsRef<Path>) -> wad::Result<Self> {
        Self::initial(Arc::new(WadFile::open(path.as_ref())?))
    }

    /// Overlays a PWAD.
    pub fn pwad(&self, path: impl AsRef<Path>) -> wad::Result<Self> {
        self.patch(Arc::new(WadFile::open(path.as_ref())?))
    }

    /// Creates a `Wad` starting with an already opened [`WadFile`], which must be an IWAD.
    pub fn initial(file: Arc<WadFile>) -> wad::Result<Self> {
        let file = file.expect(WadType::Iwad)?;
        Ok(Self::initial_unchecked(file))
    }

    /// Creates a `Wad` starting with an already opened [`WadFile`], which need not be an IWAD. Use
    /// this if you want to bypass IWAD/PWAD type checking.
    pub fn initial_unchecked(file: Arc<WadFile>) -> Self {
        Self {
            initial: file,
            patches: Vec::new(),
        }
    }

    /// Overlays an already opened [`WadFile`], which must be a PWAD.
    pub fn patch(&self, file: Arc<WadFile>) -> wad::Result<Self> {
        let file = file.expect(WadType::Pwad)?;
        Ok(self.patch_unchecked(file))
    }

    /// Overlays an already opened [`WadFile`], which need not be a PWAD. Use this if you want to
    /// bypass IWAD/PWAD type checking.
    pub fn patch_unchecked(&self, file: Arc<WadFile>) -> Self {
        let mut clone = self.clone();
        clone.patches.push(file);
        clone
    }

    /// Retrieves a unique lump by name.
    ///
    /// Lumps in later files override lumps from earlier ones.
    ///
    /// It is an error if the lump is missing.
    pub fn lump(&self, name: &str) -> wad::Result<&Lump> {
        self.lookup(|patch| patch.try_lump(name), |initial| initial.lump(name))
    }

    /// Retrieves a unique lump by name.
    ///
    /// Lumps in later files override lumps from earlier ones.
    ///
    /// Returns `Ok(None)` if the lump is missing.
    pub fn try_lump(&self, name: &str) -> wad::Result<Option<&Lump>> {
        self.try_lookup(|file| file.try_lump(name))
    }

    /// Retrieves a block of `size` lumps following a unique named marker. The marker lump is
    /// included in the result.
    ///
    /// Blocks in later files override entire blocks from earlier files.
    ///
    /// It is an error if the block is missing.
    pub fn lumps_following(&self, start: &str, size: usize) -> wad::Result<&[Lump]> {
        self.lookup(
            |patch| patch.try_lumps_following(start, size),
            |initial| initial.lumps_following(start, size),
        )
    }

    /// Retrieves a block of `size` lumps following a unique named marker. The marker lump is
    /// included in the result.
    ///
    /// Blocks in later files override entire blocks from earlier files.
    ///
    /// Returns `Ok(None)` if the block is missing.
    pub fn try_lumps_following(&self, start: &str, size: usize) -> wad::Result<Option<&[Lump]>> {
        self.try_lookup(|file| file.try_lumps_following(start, size))
    }

    /// Retrieves a block of lumps between start and end markers. The marker lumps are included in
    /// the result.
    ///
    /// Blocks in later wads override entire blocks from earlier files.
    ///
    /// It is an error if the block is missing.
    pub fn lumps_between(&self, start: &str, end: &str) -> wad::Result<&[Lump]> {
        self.lookup(
            |patch| patch.try_lumps_between(start, end),
            |initial| initial.lumps_between(start, end),
        )
    }

    /// Retrieves a block of lumps between start and end markers. The marker lumps are included in
    /// the result.
    ///
    /// Blocks in later wads override entire blocks from earlier files.
    ///
    /// Returns `Ok(None)` if the block is missing.
    pub fn try_lumps_between(&self, start: &str, end: &str) -> wad::Result<Option<&[Lump]>> {
        self.try_lookup(|file| file.try_lumps_between(start, end))
    }

    fn lookup<'wad, T>(
        &'wad self,
        try_lookup: impl Fn(&'wad WadFile) -> wad::Result<Option<T>>,
        lookup: impl FnOnce(&'wad WadFile) -> wad::Result<T>,
    ) -> wad::Result<T> {
        for patch in self.patches.iter().rev() {
            if let Some(value) = try_lookup(patch)? {
                return Ok(value);
            }
        }

        lookup(&self.initial)
    }

    fn try_lookup<'wad, T>(
        &'wad self,
        try_lookup: impl Fn(&'wad WadFile) -> wad::Result<Option<T>>,
    ) -> wad::Result<Option<T>> {
        for patch in self.patches.iter().rev() {
            if let Some(value) = try_lookup(patch)? {
                return Ok(Some(value));
            }
        }

        try_lookup(&self.initial)
    }
}

/// Adds an extension method to check that a [`WadFile`] is the correct type.
trait ExpectWadType
where
    Self: Sized,
{
    fn expect(self, expected: WadType) -> wad::Result<Self>;
}

impl ExpectWadType for Arc<WadFile> {
    /// Checks that a [`WadFile`] is the correct type.
    fn expect(self, expected: WadType) -> wad::Result<Self> {
        if self.wad_type() == expected {
            Ok(self)
        } else {
            Err(wad::Error::WrongType {
                path: self.path().into(),
                expected,
            })
        }
    }
}

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
                .lumps_following("MAP01", 11)
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
            wad.lumps_following("MAP01", 11)
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
        let silly_wad = Wad::initial_unchecked(KILLER_WAD_FILE.clone())
            .patch_unchecked(DOOM2_WAD_FILE.clone())
            .patch_unchecked(DOOM_WAD_FILE.clone())
            .patch_unchecked(BIOTECH_WAD_FILE.clone());

        assert_matches!(silly_wad.lump("E1M1"), Ok(_));
        assert_matches!(silly_wad.lump("MAP01"), Ok(_));
    }

    // Make sure `Wad` is `Send` and `Sync`.
    trait IsSendAndSync: Send + Sync {}
    impl IsSendAndSync for Wad {}
}

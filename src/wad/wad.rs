use std::{path::Path, sync::Arc};

use crate::wad::{self, FromFile, LumpRef, LumpRefs, WadFile, WadType};

/// A stack of WAD files layered on top of each other, with later files overlaying earlier ones.
///
/// A `Wad` usually consists of an IWAD overlaid with zero or more PWADs, an ordering which is
/// enforced by the [`open`] and [`patch`] builder methods. There are a set of unchecked methods if
/// you want to bypass this constraint.
///
/// [`open`]: Wad::open
/// [`patch`]: Wad::patch
#[derive(Clone, Debug)]
#[must_use]
pub struct Wad {
    initial: Arc<WadFile>,
    patches: Vec<Arc<WadFile>>,
}

impl Wad {
    /// Opens the initial [IWAD].
    ///
    /// [IWAD]: WadType::Iwad
    pub fn open(path: impl AsRef<Path>) -> wad::Result<Self> {
        let file = WadFile::open(path.as_ref())?.expect(WadType::Iwad)?;
        Ok(Self::new(file))
    }

    /// Opens the initial WAD without checking if it's an [IWAD].
    ///
    /// [IWAD]: WadType::Iwad
    pub fn open_unchecked(path: impl AsRef<Path>) -> wad::Result<Self> {
        let file = WadFile::open(path.as_ref())?;
        Ok(Self::new(file))
    }

    fn new(file: WadFile) -> Self {
        Self {
            initial: Arc::new(file),
            patches: Vec::new(),
        }
    }

    /// Overlays a [PWAD].
    ///
    /// [PWAD]: WadType::Pwad
    pub fn patch(&self, path: impl AsRef<Path>) -> wad::Result<Self> {
        let file = WadFile::open(path.as_ref())?.expect(WadType::Pwad)?;
        Ok(self.add(file))
    }

    /// Overlays a WAD without checking if it's a [PWAD].
    ///
    /// [PWAD]: WadType::Pwad
    pub fn patch_unchecked(&self, path: impl AsRef<Path>) -> wad::Result<Self> {
        let file = WadFile::open(path.as_ref())?;
        Ok(self.add(file))
    }

    fn add(&self, file: WadFile) -> Self {
        let mut clone = self.clone();
        clone.patches.push(Arc::new(file));
        clone
    }

    /// Retrieves a unique lump by name.
    ///
    /// Lumps in later files override lumps from earlier ones.
    ///
    /// It is an error if the lump is missing.
    pub fn lump(&self, name: &str) -> wad::Result<LumpRef<'_>> {
        self.lookup(|patch| patch.try_lump(name), |initial| initial.lump(name))
    }

    /// Retrieves a unique lump by name.
    ///
    /// Lumps in later files override lumps from earlier ones.
    ///
    /// Returns `Ok(None)` if the lump is missing.
    pub fn try_lump(&self, name: &str) -> wad::Result<Option<LumpRef<'_>>> {
        self.try_lookup(|file| file.try_lump(name))
    }

    /// Retrieves a block of `size` lumps following a unique named marker. The marker lump is
    /// included in the result.
    ///
    /// Blocks in later files override entire blocks from earlier files.
    ///
    /// It is an error if the block is missing.
    pub fn lumps_following(&self, start: &str, size: usize) -> wad::Result<LumpRefs<'_>> {
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
    pub fn try_lumps_following(
        &self,
        start: &str,
        size: usize,
    ) -> wad::Result<Option<LumpRefs<'_>>> {
        self.try_lookup(|file| file.try_lumps_following(start, size))
    }

    /// Retrieves a block of lumps between start and end markers. The marker lumps are included in
    /// the result.
    ///
    /// Blocks in later wads override entire blocks from earlier files.
    ///
    /// It is an error if the block is missing.
    pub fn lumps_between(&self, start: &str, end: &str) -> wad::Result<LumpRefs<'_>> {
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
    pub fn try_lumps_between(&self, start: &str, end: &str) -> wad::Result<Option<LumpRefs<'_>>> {
        self.try_lookup(|file| file.try_lumps_between(start, end))
    }

    fn lookup<'wad, T, U>(
        &'wad self,
        try_lookup: impl Fn(&'wad WadFile) -> wad::Result<Option<T>>,
        lookup: impl FnOnce(&'wad WadFile) -> wad::Result<T>,
    ) -> wad::Result<U>
    where
        T: FromFile<'wad, Out = U>,
    {
        for patch in self.patches.iter().rev() {
            if let Some(value) = try_lookup(patch)? {
                return Ok(value.from_file(patch));
            }
        }

        Ok(lookup(&self.initial)?.from_file(&self.initial))
    }

    fn try_lookup<'wad, T, U>(
        &'wad self,
        try_lookup: impl Fn(&'wad WadFile) -> wad::Result<Option<T>>,
    ) -> wad::Result<Option<U>>
    where
        T: FromFile<'wad, Out = U>,
    {
        for patch in self.patches.iter().rev() {
            if let Some(value) = try_lookup(patch)? {
                return Ok(Some(value.from_file(&patch)));
            }
        }

        Ok(try_lookup(&self.initial)?.map(|t| t.from_file(&self.initial)))
    }
}

/// Adds an extension method to check that a [`WadFile`] is the correct type.
trait ExpectWadType
where
    Self: Sized,
{
    fn expect(self, expected: WadType) -> wad::Result<Self>;
}

impl ExpectWadType for WadFile {
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
    use std::io;

    use super::*;
    use crate::test::*;

    #[test]
    fn not_a_wad() {
        assert_matches!(
            Wad::open("test/killer.txt"),
            Err(wad::Error::Io { source: err, ..}) if err.kind() == io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn lump_data() {
        assert_eq!(DOOM_WAD.lump("DEMO1").unwrap().size(), 20118);
        assert_eq!(DOOM_WAD.lump("E1M1").unwrap().size(), 0);
    }

    #[test]
    fn detect_duplicates() {
        assert_matches!(DOOM_WAD.lump("E1M1"), Ok(_));
        assert_matches!(DOOM_WAD.lump("THINGS"), Err(_));
        assert_matches!(DOOM_WAD.lump("VERTEXES"), Err(_));
        assert_matches!(DOOM_WAD.lump("SECTORS"), Err(_));
    }

    #[test]
    fn lumps_between() {
        let sprites = DOOM_WAD.lumps_between("S_START", "S_END").unwrap();
        assert_eq!(sprites.first().unwrap().name(), "S_START");
        assert_eq!(sprites.last().unwrap().name(), "S_END");
        assert_eq!(sprites.len(), 485);
        assert_eq!(sprites[100].name(), "SARGB4B6");

        // Backwards.
        assert_matches!(DOOM_WAD.lumps_between("S_END", "S_START"), Err(_));
    }

    #[test]
    fn lumps_following() {
        let map = DOOM_WAD.lumps_following("E1M8", 11).unwrap();
        assert_eq!(map.len(), 11);
        assert_eq!(
            map.iter().map(LumpRef::name).collect::<Vec<_>>(),
            [
                "E1M8", "THINGS", "LINEDEFS", "SIDEDEFS", "VERTEXES", "SEGS", "SSECTORS", "NODES",
                "SECTORS", "REJECT", "BLOCKMAP"
            ],
        );

        // Check in and out of bounds sizes.
        assert_matches!(DOOM_WAD.try_lumps_following("E1M1", 0), Ok(Some(_)));
        assert_matches!(DOOM_WAD.try_lumps_following("E1M1", 9999), Err(_));
    }

    #[test]
    fn iwad_then_pwads() {
        // IWAD + PWAD = success.
        let _ = Wad::open(DOOM_WAD_PATH)
            .unwrap()
            .patch(KILLER_WAD_PATH)
            .unwrap();

        // IWAD + IWAD = error.
        let wad = Wad::open(DOOM_WAD_PATH).unwrap();
        assert_matches!(wad.patch(DOOM2_WAD_PATH), Err(_));

        // Can't start with a PWAD.
        assert_matches!(Wad::open(KILLER_WAD_PATH), Err(_));
    }

    #[test]
    fn no_type_checking() -> wad::Result<()> {
        // Nonsensical ordering.
        let silly_wad = Wad::open_unchecked(KILLER_WAD_PATH)?
            .patch_unchecked(DOOM2_WAD_PATH)?
            .patch_unchecked(DOOM_WAD_PATH)?
            .patch_unchecked(BIOTECH_WAD_PATH)?;

        assert_matches!(silly_wad.lump("E1M1"), Ok(_));
        assert_matches!(silly_wad.lump("MAP01"), Ok(_));

        Ok(())
    }

    #[test]
    fn layering() {
        assert_eq!(DOOM2_WAD.lump("DEMO3").unwrap().size(), 17898);
        assert_eq!(
            DOOM2_WAD
                .lumps_following("MAP01", 11)
                .unwrap()
                .iter()
                .map(|lump| (lump.name(), lump.size()))
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

        let wad = DOOM2_WAD.patch(BIOTECH_WAD_PATH).unwrap();
        assert_eq!(wad.lump("DEMO3").unwrap().size(), 9490);
        assert_eq!(
            wad.lumps_following("MAP01", 11)
                .unwrap()
                .iter()
                .map(|lump| (lump.name(), lump.size()))
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

    // Make sure `Wad` is `Send` and `Sync`.
    trait IsSendAndSync: Send + Sync {}
    impl IsSendAndSync for Wad {}
}

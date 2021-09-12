use std::{io, path::Path, sync::Arc};

use crate::{Lump, Wad, WadFile, WadType};

/// A stack of one or more WAD files layered on top of each other, with later
/// files overlaying earlier ones. Usually the first WAD is a IWAD and the rest
/// are PWADs, but that's not a strict requirement. Other combinations are
/// allowed.
#[derive(Clone)]
#[must_use]
pub struct WadStack {
    wads: Vec<Arc<dyn Wad + Send + Sync>>,
}

impl WadStack {
    /// Creates a stack starting with a IWAD such as `doom.wad`.
    pub fn iwad(file: impl AsRef<Path>) -> io::Result<Self> {
        Self::iwad_impl(file.as_ref())
    }

    fn iwad_impl(file: &Path) -> io::Result<Self> {
        let wad = WadFile::open(file)?;

        match wad.wad_type() {
            WadType::Iwad => Ok(Self::unchecked(wad)),
            WadType::Pwad => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} not an IWAD", file.display()),
            )),
        }
    }

    /// Returns a new stack with a PWAD overlaid.
    pub fn pwad(&self, file: impl AsRef<Path>) -> io::Result<Self> {
        self.pwad_impl(file.as_ref())
    }

    fn pwad_impl(&self, file: &Path) -> io::Result<Self> {
        let wad = WadFile::open(file)?;

        match wad.wad_type() {
            WadType::Pwad => Ok(self.and_unchecked(wad)),
            WadType::Iwad => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} not a PWAD", file.display()),
            )),
        }
    }

    /// Creates a stack without a starting IWAD. Use this if you want to bypass
    /// IWAD/PWAD type checking.
    pub fn empty_unchecked() -> Self {
        Self { wads: Vec::new() }
    }

    /// Creates a stack starting with a generic [`Wad`], which need not be an IWAD.
    /// Use this if you want to bypass IWAD/PWAD type checking.
    pub fn unchecked(wad: impl Wad + Send + Sync + 'static) -> Self {
        Self::empty_unchecked().and_unchecked(wad)
    }

    /// Returns a new stack with a generic [`Wad`] overlaid, which need not be a
    /// PWAD. Use this if you want to bypass IWAD/PWAD type checking.
    pub fn and_unchecked(&self, wad: impl Wad + Send + Sync + 'static) -> Self {
        let mut clone = self.clone();
        clone.wads.push(Arc::new(wad));
        clone
    }
}

impl Wad for WadStack {
    /// Retrieves a named lump. The name must be unique.
    ///
    /// Lumps in later files override lumps from earlier ones.
    fn lump(&self, name: &str) -> Option<&Lump> {
        self.wads.iter().rev().find_map(|wad| wad.lump(name))
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
    fn lumps_after(&self, start: &str, size: usize) -> Option<&[Lump]> {
        self.wads
            .iter()
            .rev()
            .find_map(|wad| wad.lumps_after(start, size))
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
    fn lumps_between(&self, start: &str, end: &str) -> Option<&[Lump]> {
        self.wads
            .iter()
            .rev()
            .find_map(|wad| wad.lumps_between(start, end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

    #[test]
    fn iwad_then_pwads() {
        // IWAD + PWAD = success.
        let _ = WadStack::iwad(DOOM_WAD_PATH)
            .unwrap()
            .pwad(KILLER_WAD_PATH)
            .unwrap();

        // IWAD + IWAD = error.
        let wad = WadStack::iwad(DOOM_WAD_PATH).unwrap();
        assert!(wad.pwad(DOOM2_WAD_PATH).is_err());

        // Can't start with a PWAD.
        assert!(WadStack::iwad(KILLER_WAD_PATH).is_err());
    }

    #[test]
    fn layering() {
        assert_eq!(DOOM2_WAD.lump("DEMO3").unwrap().size(), 17898);
        assert_eq!(
            DOOM2_WAD
                .lumps_after("MAP01", 10)
                .unwrap()
                .iter()
                .map(|lump| (lump.name.as_str(), lump.size()))
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
        assert_eq!(
            DOOM2_WAD.lumps_between("S_START", "S_END").unwrap().len(),
            1381
        );

        let wad = DOOM2_WAD.and_unchecked(&*BIOTECH_WAD);
        assert_eq!(wad.lump("DEMO3").unwrap().size(), 9490);
        assert_eq!(
            wad.lumps_after("MAP01", 10)
                .unwrap()
                .iter()
                .map(|lump| (lump.name.as_str(), lump.size()))
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

    #[test]
    fn no_type_checking() {
        // Nonsensical ordering.
        let silly_wad = WadStack::empty_unchecked()
            .and_unchecked(&*KILLER_WAD)
            .and_unchecked(&*DOOM2_WAD)
            .and_unchecked(&*DOOM_WAD)
            .and_unchecked(&*BIOTECH_WAD);

        assert!(silly_wad.lump("E1M1").is_some());
        assert!(silly_wad.lump("MAP01").is_some());
    }

    // Doesn't need to run, just compile.
    fn _can_add_static_refs() {
        let wad: &'static _ = Box::leak(Box::new(WadStack::empty_unchecked()));
        let _ = WadStack::unchecked(wad);
    }

    // Doesn't need to run, just compile.
    fn _can_add_trait_objects() {
        let boxed: Box<dyn Wad + Send + Sync> = Box::new(WadStack::empty_unchecked());
        let arced: Arc<dyn Wad + Send + Sync> = Arc::new(WadStack::empty_unchecked());

        let _ = WadStack::unchecked(boxed);
        let _ = WadStack::unchecked(arced);
    }

    // Make sure WadStack is Send and Sync.
    trait IsSendAndSync: Send + Sync {}
    impl IsSendAndSync for WadStack {}
}

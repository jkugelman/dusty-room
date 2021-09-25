use std::collections::BTreeMap;
use std::fmt;

use ndarray::ArrayView2;

use crate::wad::{self, Lump, Wad};

/// A list of floor and ceiling textures, indexed by name.
#[derive(Clone)]
pub struct FlatBank<'wad>(BTreeMap<&'wad str, Flat<'wad>>);

impl<'wad> FlatBank<'wad> {
    /// Loads all the flats from a [`Wad`].
    ///
    /// Flats are found between the `F_START` and `F_END` marker lumps.
    pub fn load(wad: &'wad Wad) -> wad::Result<Self> {
        let lumps = wad.lumps_between("F_START", "F_END")?;
        let mut flats = BTreeMap::new();

        for lump in lumps {
            if lump.is_empty() {
                continue;
            }

            let flat = Flat::load(&lump)?;
            let existing = flats.insert(flat.name, flat);

            if existing.is_some() {
                return Err(lump.error(&format!("duplicate flat {}", lump.name())));
            }
        }

        Ok(Self(flats))
    }

    pub fn get(&self, name: &str) -> Option<&Flat<'wad>> {
        self.0.get(name)
    }
}

impl fmt::Debug for FlatBank<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.0.values())
    }
}

/// A floor or ceiling texture.
#[derive(Clone)]
pub struct Flat<'wad> {
    name: &'wad str,
    pixels: ArrayView2<'wad, u8>,
}

impl<'wad> Flat<'wad> {
    /// Load a flat from a lump.
    pub fn load(lump: &Lump<'wad>) -> wad::Result<Self> {
        let width: usize = Self::width().into();
        let height: usize = Self::height().into();

        lump.expect_size(width * height)?;

        Ok(Self {
            name: lump.name(),
            pixels: ArrayView2::from_shape((width, height), lump.data()).unwrap(),
        })
    }

    /// Flat name, the name of its [`Lump`].
    pub fn name(&self) -> &'wad str {
        self.name
    }

    /// Width in pixels. Flats are always 64x64.
    pub const fn width() -> u16 {
        64
    }

    /// Height in pixels. Flats are always 64x64.
    pub const fn height() -> u16 {
        64
    }
}

impl fmt::Debug for Flat<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

impl fmt::Display for Flat<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wad::test::*;

    #[test]
    fn load() {
        let flats = FlatBank::load(&DOOM2_WAD).unwrap();

        assert_matches!(flats.get("CEIL3_5"), Some(_));
        assert_matches!(flats.get("GATE2"), Some(_));
        assert_matches!(flats.get("NUKAGE1"), Some(_));
    }
}

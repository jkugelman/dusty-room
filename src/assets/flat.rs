use std::collections::BTreeMap;
use std::fmt;
use std::ops::{Deref, Index};

use bytes::Bytes;

use crate::wad::{self, Lump, Wad};

/// A bank of [sector] floor and ceiling textures, indexed by name.
///
/// [sector]: crate::map::Sector
#[derive(Clone)]
pub struct FlatBank(BTreeMap<String, Flat>);

impl FlatBank {
    /// Loads all the flats from a [`Wad`] found between the `F_START` and `F_END` marker lumps.
    pub fn load(wad: &Wad) -> wad::Result<Self> {
        let lumps = wad.lumps_between("F_START", "F_END")?;
        let mut flats = BTreeMap::new();

        for lump in lumps {
            if lump.is_empty() {
                continue;
            }

            let flat = Flat::load(&lump)?;
            let existing = flats.insert(flat.name.clone(), flat);

            if existing.is_some() {
                return Err(lump.error(format!("duplicate flat {}", lump.name())));
            }
        }

        Ok(Self(flats))
    }

    /// Looks up a flat name.
    pub fn get(&self, name: &str) -> Option<&Flat> {
        self.0.get(name)
    }
}

impl Index<&str> for FlatBank {
    type Output = Flat;

    /// Looks up a flat name.
    fn index(&self, name: &str) -> &Self::Output {
        self.get(name).expect("flat missing")
    }
}

impl Deref for FlatBank {
    type Target = BTreeMap<String, Flat>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for FlatBank {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let Self(flats) = self;

        write!(fmt, "{:?}", flats.values())
    }
}

/// A [sector] floor or ceiling texture.
///
/// [sector]: crate::map::Sector
#[derive(Clone)]
pub struct Flat {
    /// Name of the flat. Used by [sectors].
    ///
    /// [sectors]: crate::map::Sector
    pub name: String,

    pixels: Bytes,
}

impl Flat {
    /// Loads a flat from a lump.
    pub fn load(lump: &Lump) -> wad::Result<Self> {
        let width: usize = Self::width().into();
        let height: usize = Self::height().into();

        let mut cursor = lump.cursor();
        let name = lump.name().to_owned();
        cursor.need(width * height)?;
        let pixels = cursor.split_to(width * height);
        cursor.done()?;

        Ok(Self { name, pixels })
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

impl fmt::Debug for Flat {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let Self { name, pixels: _ } = self;

        write!(fmt, "{}", name)
    }
}

impl fmt::Display for Flat {
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

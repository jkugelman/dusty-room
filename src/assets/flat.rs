use std::collections::{btree_map, BTreeMap};
use std::fmt;
use std::ops::{Deref, DerefMut};

use ndarray::ArrayView2;

use crate::wad::{self, Lump, Wad};

/// A list of floor and ceiling textures, indexed by name.
#[derive(Clone)]
pub struct FlatBank<'wad>(BTreeMap<&'wad str, Flat<'wad>>);

impl fmt::Debug for FlatBank<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.0.values())
    }
}

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

            let flat = Flat::load(lump)?;
            let existing = flats.insert(flat.name, flat);

            if let Some(_) = existing {
                return Err(lump.error(&format!("duplicate flat {}", lump.name())));
            }
        }

        Ok(FlatBank(flats))
    }
}

impl<'wad> Deref for FlatBank<'wad> {
    type Target = BTreeMap<&'wad str, Flat<'wad>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FlatBank<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'wad> IntoIterator for FlatBank<'wad> {
    type Item = (&'wad str, Flat<'wad>);
    type IntoIter = btree_map::IntoIter<&'wad str, Flat<'wad>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, 'wad> IntoIterator for &'a FlatBank<'wad> {
    type Item = (&'a &'wad str, &'a Flat<'wad>);
    type IntoIter = btree_map::Iter<'a, &'wad str, Flat<'wad>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'wad> IntoIterator for &'a mut FlatBank<'wad> {
    type Item = (&'a &'wad str, &'a mut Flat<'wad>);
    type IntoIter = btree_map::IterMut<'a, &'wad str, Flat<'wad>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
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
    pub fn load(lump: Lump<'wad>) -> wad::Result<Self> {
        let lump = lump.expect_size(64 * 64)?;

        Ok(Self {
            name: lump.name(),
            pixels: ArrayView2::from_shape(Self::shape(), lump.data()).unwrap(),
        })
    }

    /// Flat name, the name of its [`Lump`].
    pub fn name(&self) -> &str {
        self.name
    }

    /// Flats are always 64x64 pixels.
    pub const fn shape() -> (usize, usize) {
        (64, 64)
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

        assert!(flats.contains_key("CEIL3_5"));
        assert!(flats.contains_key("GATE2"));
        assert!(flats.contains_key("NUKAGE1"));
    }
}

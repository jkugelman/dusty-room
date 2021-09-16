use std::collections::{btree_map, BTreeMap};
use std::fmt;
use std::ops::{Deref, DerefMut};

use super::palette::Pixels;
use super::wad::{self, LumpRef, Wad};

/// A 64x64 floor or ceiling texture.
#[derive(Clone)]
pub struct Flat {
    name: String,
    pixels: Pixels,
}

impl Flat {
    /// Load a flat from a lump.
    pub fn load(lump: LumpRef) -> wad::Result<Self> {
        let shape = Self::shape();
        let size = shape.0 * shape.1;

        if lump.size() != size {
            return Err(lump.error(&format!("expected {} bytes, got {}", size, lump.size())));
        }

        let name = lump.name().into();
        let pixels = Pixels::from_shape_vec(shape, lump.data().to_vec()).unwrap();

        Ok(Flat { name, pixels })
    }

    /// Flat name, the name of its [lump].
    ///
    /// [lump]: wad::LumpRef
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Flats are always 64x64.
    pub const fn shape() -> (usize, usize) {
        (64, 64)
    }
}

impl Deref for Flat {
    type Target = Pixels;

    fn deref(&self) -> &Self::Target {
        &self.pixels
    }
}

impl fmt::Debug for Flat {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

impl fmt::Display for Flat {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

/// A list of floor and ceiling textures, indexed by name.
#[derive(Clone)]
pub struct FlatBank {
    flats: BTreeMap<String, Flat>,
}

impl fmt::Debug for FlatBank {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.flats.values())
    }
}

impl FlatBank {
    /// Loads all the flats from a [`Wad`].
    ///
    /// Flats are found between the `F_START` and `F_END` marker lumps.
    pub fn load(wad: &Wad) -> wad::Result<Self> {
        let lumps = wad.lumps_between("F_START", "F_END")?;
        let mut flats = BTreeMap::new();

        for lump in lumps {
            if lump.is_empty() {
                continue;
            }

            let flat = Flat::load(lump)?;
            let existing = flats.insert(flat.name.clone(), flat);

            if let Some(_) = existing {
                return Err(lump.error(&format!("duplicate flat {}", lump.name())));
            }
        }

        Ok(FlatBank { flats })
    }
}

impl Deref for FlatBank {
    type Target = BTreeMap<String, Flat>;

    fn deref(&self) -> &Self::Target {
        &self.flats
    }
}

impl DerefMut for FlatBank {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flats
    }
}

impl IntoIterator for FlatBank {
    type Item = (String, Flat);
    type IntoIter = btree_map::IntoIter<String, Flat>;

    fn into_iter(self) -> Self::IntoIter {
        self.flats.into_iter()
    }
}

impl<'a> IntoIterator for &'a FlatBank {
    type Item = (&'a String, &'a Flat);
    type IntoIter = btree_map::Iter<'a, String, Flat>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut FlatBank {
    type Item = (&'a String, &'a mut Flat);
    type IntoIter = btree_map::IterMut<'a, String, Flat>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

    #[test]
    fn load() {
        let flats = FlatBank::load(&DOOM2_WAD).unwrap();

        assert!(flats.contains_key("CEIL3_5"));
        assert!(flats.contains_key("GATE2"));
        assert!(flats.contains_key("NUKAGE1"));
    }
}

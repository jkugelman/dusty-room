use std::convert::TryInto;
use std::mem;
use std::ops::Index;

use bytes::{Buf, Bytes};

use crate::wad::{self, Cursor, Wad};

/// A bank of color palettes loaded from the `PLAYPAL` lump.
///
/// The active palette can be switched at any time. There is no palette selected initially so make
/// sure to choose one.
#[derive(Debug)]
pub struct PaletteBank {
    palettes: Vec<Palette>,
    active_index: Option<usize>,
}

impl PaletteBank {
    /// Loads a bank of color palettes from the `PLAYPAL` lump.
    pub fn load(wad: &Wad) -> wad::Result<Self> {
        let lump = wad.lump("PLAYPAL")?;
        let mut cursor = lump.cursor();

        let mut palettes = Vec::with_capacity(lump.size() / PALETTE_BYTES);

        while cursor.has_remaining() {
            palettes.push(Palette::load(&mut cursor)?);
        }

        cursor.done()?;

        Ok(PaletteBank {
            palettes,
            active_index: None,
        })
    }

    /// The number of palettes in the bank.
    pub fn count(&self) -> usize {
        self.palettes.len()
    }

    /// Gets the active palette.
    pub fn active(&self) -> &Palette {
        &self.palettes[self.active_index.expect("active palette not set")]
    }

    /// Sets and returns the active palette.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of range.
    pub fn set_active(&mut self, index: usize) -> &Palette {
        assert!(index < self.count());
        self.active_index = Some(index);
        self.active()
    }
}

/// A 256-color palette. Part of a [`PaletteBank`].
#[derive(Debug, Clone)]
pub struct Palette {
    raw: Bytes,
}

const PALETTE_COLORS: usize = 256;
const PALETTE_BYTES: usize = 3 * PALETTE_COLORS;

impl Palette {
    fn load(cursor: &mut Cursor) -> wad::Result<Self> {
        let raw = cursor.need(PALETTE_BYTES)?.split_to(PALETTE_BYTES);
        Ok(Self { raw })
    }
}

impl Index<u8> for Palette {
    type Output = (u8, u8, u8);

    fn index(&self, index: u8) -> &Self::Output {
        let index: usize = index.into();
        let rgb: &[u8; 3] = self.raw[index * 3..index * 3 + 3].try_into().unwrap();
        // SAFETY: `[u8; 3]` and `(u8, u8, u8)` have the same size and layout.
        unsafe { mem::transmute(rgb) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wad::test::*;

    #[test]
    fn load() {
        let mut palettes = PaletteBank::load(&DOOM_WAD).unwrap();

        assert_eq!(palettes.count(), 14);

        let p0 = palettes.set_active(0);
        assert_eq!(p0[0], (0, 0, 0));
        assert_eq!(p0[255], (167, 107, 107));

        let p13 = palettes.set_active(13);
        assert_eq!(p13[0], (0, 32, 0));
        assert_eq!(p13[255], (147, 125, 94));
    }
}

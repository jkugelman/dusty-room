use std::convert::TryInto;
use std::mem;
use std::ops::Index;

use crate::wad::{self, Wad};

/// A bank of color palettes loaded from the `PLAYPAL` lump.
///
/// The active palette can be switched at any time. There is no palette selected initially so make
/// sure to choose one.
#[derive(Debug)]
pub struct PaletteBank<'wad> {
    palettes: Vec<Palette<'wad>>,
    active_index: Option<usize>,
}

impl<'wad> PaletteBank<'wad> {
    /// Loads a bank of color palettes from the `PLAYPAL` lump.
    pub fn load(wad: &'wad Wad) -> wad::Result<Self> {
        let lump = wad.lump("PLAYPAL")?;
        let lump = lump.expect_size_multiple(PALETTE_BYTES)?;

        let palettes: Vec<Palette> = lump
            .data()
            .chunks_exact(PALETTE_BYTES)
            .map(|chunk| -> &[u8; PALETTE_BYTES] { chunk.try_into().unwrap() })
            .map(Palette::from_raw)
            .collect();

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
pub struct Palette<'wad>(&'wad [(u8, u8, u8); PALETTE_COLORS]);

const PALETTE_COLORS: usize = 256;
const PALETTE_BYTES: usize = 3 * PALETTE_COLORS;

impl<'wad> Palette<'wad> {
    pub fn from_raw(raw: &'wad [u8; 3 * PALETTE_COLORS]) -> Self {
        // SAFETY: `[u8; 3 * PALETE_COLORS]` has the same size and layout as
        // `[(u8, u8, u8); PALETTE_COLORS]`.
        Self(unsafe { mem::transmute(raw) })
    }
}

impl<'wad> Index<u8> for Palette<'wad> {
    type Output = (u8, u8, u8);

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[usize::from(index)]
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

use std::convert::TryInto;
use std::mem;

use crate::wad::{self, Wad};

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
    pub fn load(wad: &Wad) -> wad::Result<PaletteBank> {
        let lump = wad.lump("PLAYPAL")?;
        let lump = lump.expect_size_multiple(PALETTE_BYTES)?;

        let palettes: Vec<Palette> = lump
            .data()
            .chunks_exact(PALETTE_BYTES)
            .map(|chunk| -> [u8; PALETTE_BYTES] { chunk.try_into().unwrap() })
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
pub struct Palette {
    rgb: [(u8, u8, u8); PALETTE_COLORS],
}

const PALETTE_COLORS: usize = 256;
const PALETTE_BYTES: usize = 3 * PALETTE_COLORS;

impl Palette {
    pub fn from_rgb(rgb: [(u8, u8, u8); PALETTE_COLORS]) -> Self {
        Self { rgb }
    }

    pub fn from_raw(raw: [u8; 3 * PALETTE_COLORS]) -> Self {
        // SAFETY: `[u8; 3 * PALETTE_COLORS]` has the same size and layout as
        // `[(u8, u8, u8); PALETTE_COLORS]`.
        Self::from_rgb(unsafe { mem::transmute(raw) })
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
        assert_eq!(p0.rgb[0], (0, 0, 0));
        assert_eq!(p0.rgb[255], (167, 107, 107));

        let p13 = palettes.set_active(13);
        assert_eq!(p13.rgb[0], (0, 32, 0));
        assert_eq!(p13.rgb[255], (147, 125, 94));
    }
}

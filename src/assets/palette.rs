use std::array::TryFromSliceError;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::Deref;

use image::{Rgb, RgbaImage};

use crate::assets::image::Image;
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

        let palettes: Vec<Palette> = lump
            .expect_size_multiple(PALETTE_BYTES)?
            .data()
            .chunks_exact(PALETTE_BYTES)
            .map(|samples: &[u8]| samples.try_into().unwrap())
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
pub union Palette {
    // SAFETY: All the union fields have the same size and layout and are safe to read from
    // at any time.
    rgb: [Rgb<u8>; PALETTE_COLORS],
    triple: [(u8, u8, u8); PALETTE_COLORS],
    bytes: [u8; PALETTE_BYTES],
}

const PALETTE_COLORS: usize = 256;
const PALETTE_BYTES: usize = 3 * PALETTE_COLORS;

impl Deref for Palette {
    type Target = [Rgb<u8>; PALETTE_COLORS];

    fn deref(&self) -> &Self::Target {
        // SAFETY: See explanation above.
        unsafe { &self.rgb }
    }
}

impl From<[u8; PALETTE_BYTES]> for Palette {
    fn from(bytes: [u8; PALETTE_BYTES]) -> Self {
        Palette { bytes }
    }
}

impl TryFrom<&[u8]> for Palette {
    type Error = TryFromSliceError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let bytes: [u8; PALETTE_BYTES] = bytes.try_into()?;
        Ok(Self { bytes })
    }
}

impl Clone for Palette {
    fn clone(&self) -> Self {
        Self {
            // SAFETY: See explanation above.
            rgb: unsafe { self.rgb.clone() },
        }
    }
}

impl fmt::Debug for Palette {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // SAFETY: See explanation above.
        unsafe { self.rgb }.fmt(fmt)
    }
}

/// Adds an extension method to convert 8-bit images to RGBA.
pub trait ToRgba {
    /// Returns a copy of this image with the 8-bit colors converted to RGBA using the chosen
    /// palette.
    fn to_rgba(&self, palette: &Palette) -> RgbaImage;
}

/// Adds an extension method to convert 8-bit images to RGBA.
impl ToRgba for Image {
    /// Returns a copy of this image with the 8-bit colors converted to RGBA using the chosen
    /// palette.
    fn to_rgba(&self, palette: &Palette) -> RgbaImage {
        // SAFETY: All the union fields have the same size and layout and are safe to read from
        // at any time.
        self.clone()
            .expand_palette(unsafe { &palette.triple }, None)
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
        assert_eq!(p0[0], Rgb::from([0, 0, 0]));
        assert_eq!(p0[255], Rgb::from([167, 107, 107]));

        let p13 = palettes.set_active(13);
        assert_eq!(p13[0], Rgb::from([0, 32, 0]));
        assert_eq!(p13[255], Rgb::from([147, 125, 94]));
    }
}

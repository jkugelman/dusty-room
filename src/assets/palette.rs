use std::fmt;
use std::mem::{self, size_of, MaybeUninit};

use ndarray::Array2;

use super::{wad, Wad};

/// An RGB color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
    /// Red value from 0-255.
    pub r: u8,
    /// Green value from 0-255.
    pub g: u8,
    /// Blue value from 0-255.
    pub b: u8,
}

impl Color {
    /// Creates a color with the given RGB values.
    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// An index into a [`Palette`].
pub type ColorIndex = u8;

/// A 2D array of indexed pixels.
pub type Pixels = Array2<ColorIndex>;

/// A 256-color palette. Part of a [`PaletteBank`].
pub type Palette = [Color; 256];

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

        if lump.size() % size_of::<Palette>() != 0 {
            return Err(lump.error(&format!(
                "size {} not a multiple of {}",
                lump.size(),
                size_of::<Palette>()
            )));
        }

        let palettes: Vec<Palette> = lump
            .data()
            .chunks_exact(size_of::<Palette>())
            .map(|chunk: &[u8]| {
                // Create an uninitialized array of colors.
                //
                // SAFETY: The `assume_init` is safe because the type we are claiming to have
                // initialized here is a bunch of `MaybeUninit`s, which do not require
                // initialization.
                let mut palette: [MaybeUninit<Color>; 256] =
                    unsafe { MaybeUninit::uninit().assume_init() };

                for i in 0..256 {
                    palette[i].write(Color {
                        r: chunk[i * 3 + 0],
                        g: chunk[i * 3 + 1],
                        b: chunk[i * 3 + 2],
                    });
                }

                // SAFETY: Everything is initialized. Transmute the array to the initialized type.
                let palette: [Color; 256] = unsafe { mem::transmute::<_, [Color; 256]>(palette) };

                palette
            })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

    #[test]
    fn load() {
        let mut palettes = PaletteBank::load(&DOOM_WAD).unwrap();

        assert_eq!(palettes.count(), 14);

        let p0 = palettes.set_active(0);
        assert_eq!(p0[0], Color::rgb(0, 0, 0));
        assert_eq!(p0[255], Color::rgb(167, 107, 107));

        let p13 = palettes.set_active(13);
        assert_eq!(p13[0], Color::rgb(0, 32, 0));
        assert_eq!(p13[255], Color::rgb(147, 125, 94));
    }
}
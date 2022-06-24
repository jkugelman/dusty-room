use crate::assets::FlatBank;
use crate::assets::PaletteBank;
use crate::assets::TextureBank;
use crate::wad::{self, Wad};

/// Holds all of the fixed assets loaded from a [`Wad`]: graphics, sounds, music, text strings, etc.
/// Map data is stored [elsewhere] since typically only one map is loaded at a time.
///
/// [elsewhere]: crate::map::Map
#[derive(Debug)]
pub struct Assets {
    pub palette_bank: PaletteBank,
    pub flat_bank: FlatBank,
    pub texture_bank: TextureBank,
}

impl Assets {
    /// Loads assets from a [`Wad`].
    pub fn load(wad: &Wad) -> wad::Result<Self> {
        let palette_bank = PaletteBank::load(wad)?;
        let flat_bank = FlatBank::load(wad)?;
        let texture_bank = TextureBank::load(wad)?;

        Ok(Assets { palette_bank, flat_bank, texture_bank })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wad::test::*;

    #[test]
    fn load() {
        Assets::load(&DOOM_WAD).unwrap();
        Assets::load(&DOOM2_WAD).unwrap();
        Assets::load(&KILLER_WAD).unwrap();
        Assets::load(&BIOTECH_WAD).unwrap();
    }
}

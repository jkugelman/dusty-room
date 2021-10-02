use crate::assets::FlatBank;
use crate::assets::PaletteBank;
use crate::assets::TextureBank;
use crate::wad::{self, Wad};

/// Holds all of the assets loaded from a [`Wad`]: maps, sprites, textures, sounds, etc.
#[derive(Debug)]
pub struct Assets {
    _palette_bank: PaletteBank,
    _flat_bank: FlatBank,
    texture_bank: TextureBank,
}

impl Assets {
    /// Loads assets from a [`Wad`].
    pub fn load(wad: &Wad) -> wad::Result<Self> {
        let _palette_bank = PaletteBank::load(wad)?;
        let _flat_bank = FlatBank::load(wad)?;
        let texture_bank = TextureBank::load(wad)?;

        Ok(Assets {
            _palette_bank,
            _flat_bank,
            texture_bank,
        })
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

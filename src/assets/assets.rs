use crate::assets::flat::FlatBank;
use crate::assets::palette::PaletteBank;
use crate::assets::texture::TextureBank;
use crate::assets::PatchBank;
use crate::wad::{self, Wad};

/// Holds all of the assets loaded from a [`Wad`]: maps, sprites, textures, sounds, etc.
#[derive(Debug)]
pub struct Assets<'wad> {
    _palette_bank: PaletteBank<'wad>,
    _flat_bank: FlatBank<'wad>,
    _patch_bank: PatchBank<'wad>,
    _texture_bank: TextureBank<'wad>,
}

impl<'wad> Assets<'wad> {
    /// Loads assets from a [`Wad`].
    pub fn load(wad: &'wad Wad) -> wad::Result<Self> {
        let palette_bank = PaletteBank::load(wad)?;
        let flat_bank = FlatBank::load(wad)?;
        let patch_bank = PatchBank::load(wad)?;
        let texture_bank = TextureBank::load(wad)?;

        Ok(Assets {
            _palette_bank: palette_bank,
            _flat_bank: flat_bank,
            _patch_bank: patch_bank,
            _texture_bank: texture_bank,
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

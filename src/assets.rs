pub mod flat;
pub mod map;
pub mod palette;
pub mod wad;

use self::flat::FlatBank;
use self::palette::PaletteBank;
use self::wad::Wad;

/// Holds all of the assets loaded from a [`Wad`]: maps, sprites, textures, sounds, etc.
#[derive(Debug)]
pub struct Assets {
    palette_bank: PaletteBank,
    flat_bank: FlatBank,
}

impl Assets {
    /// Load assets from a [`Wad`].
    pub fn load(wad: &Wad) -> wad::Result<Self> {
        let palette_bank = PaletteBank::load(&wad)?;
        let flat_bank = FlatBank::load(&wad)?;

        Ok(Assets {
            palette_bank,
            flat_bank,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

    #[test]
    fn load() {
        Assets::load(&DOOM_WAD).unwrap();
        Assets::load(&DOOM2_WAD).unwrap();
        Assets::load(&KILLER_WAD).unwrap();
        Assets::load(&BIOTECH_WAD).unwrap();
    }
}

use std::collections::BTreeMap;
use std::convert::TryInto;

use bytes::Buf;

use crate::wad::{self, Cursor, Lump, Wad};

#[derive(Clone, Debug)]
pub struct TextureBank(BTreeMap<String, Texture>);

impl TextureBank {
    /// Loads all the textures from a [`Wad`].
    ///
    /// Textures are listed in the `TEXTURE1` and `TEXTURE2` lumps.
    pub fn load(wad: &Wad) -> wad::Result<Self> {
        let mut textures = BTreeMap::new();

        for lump in Self::texture_lumps(wad)? {
            Self::load_from(&lump, &mut textures)?;
        }

        Ok(Self(textures))
    }

    fn texture_lumps(wad: &Wad) -> wad::Result<Vec<Lump>> {
        let iter = Some(wad.lump("TEXTURE1")?).into_iter();
        let iter = iter.chain(wad.try_lump("TEXTURE2")?);
        Ok(iter.collect())
    }

    fn load_from(lump: &Lump, textures: &mut BTreeMap<String, Texture>) -> wad::Result<()> {
        let mut cursor = lump.cursor();

        let count = cursor.need(4)?.get_u32_le();

        // Read texture offsets. The WAD is untrusted so clamp how much memory is pre-allocated.
        // Don't worry about overflow converting from `u32` to `usize`. The wrong capacity won't
        // affect correctness.
        let mut offsets = Vec::with_capacity(count.clamp(0, 1024) as usize);
        for _ in 0..count {
            offsets.push(cursor.need(4)?.get_u32_le());
        }

        cursor.clear();
        cursor.done()?;

        // Read textures.
        for offset in offsets {
            let texture = Texture::load(lump, offset.try_into().unwrap())?;
            textures.insert(texture.name().to_owned(), texture);
        }

        Ok(())
    }

    /// Retrieves a texture by name.
    pub fn get(&self, name: &str) -> Option<&Texture> {
        self.0.get(name)
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub patches: Vec<PatchPlacement>,
}

impl Texture {
    fn load(lump: &Lump, offset: usize) -> wad::Result<Self> {
        let mut cursor = lump.cursor();
        cursor.need(offset)?.advance(offset);

        cursor.need(22)?;
        let name = cursor.get_name();
        let _flags = cursor.get_u16_le();
        let _unused = cursor.get_u16_le();
        let width = cursor.get_u16_le();
        let height = cursor.get_u16_le();
        let _unused = cursor.get_u32_le();

        let patch_count: usize = cursor.get_u16_le().into();
        let mut patches = Vec::with_capacity(patch_count.clamp(0, 64));
        cursor.need(patch_count * 10)?;

        for _ in 0..patch_count {
            let x = cursor.get_u16_le();
            let y = cursor.get_u16_le();
            let patch = cursor.get_u16_le();
            let _unused = cursor.get_u16_le();
            let _unused = cursor.get_u16_le();

            patches.push(PatchPlacement { x, y, patch });
        }

        cursor.clear();
        cursor.done()?;

        Ok(Self {
            name,
            width,
            height,
            patches,
        })
    }

    /// The texture's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The texture's width in pixels.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// The texture's height in pixels.
    pub fn height(&self) -> u16 {
        self.height
    }
}

#[derive(Clone, Debug)]
pub struct PatchPlacement {
    pub x: u16,
    pub y: u16,
    pub patch: u16,
}

impl PatchPlacement {
    pub fn load(cursor: &mut Cursor) -> wad::Result<Self> {
        cursor.need(10)?;
        let x = cursor.get_u16_le();
        let y = cursor.get_u16_le();
        let patch = cursor.get_u16_le();
        let _unused = cursor.get_u16_le();
        let _unused = cursor.get_u16_le();

        Ok(Self { x, y, patch })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::PatchBank;
    use crate::wad::test::*;

    #[test]
    fn load() {
        let patches = PatchBank::load(&BIOTECH_WAD).unwrap();
        let textures = TextureBank::load(&BIOTECH_WAD).unwrap();

        let exit_door = textures.get("EXITDOOR").unwrap();
        assert_eq!(exit_door.name(), "EXITDOOR");
        assert_eq!(exit_door.width(), 128);
        assert_eq!(exit_door.height(), 72);
        assert_eq!(exit_door.patches.len(), 4);
        assert_eq!(exit_door.patches[0].x, 0);
        assert_eq!(exit_door.patches[0].y, 0);
        assert_eq!(
            patches.get(exit_door.patches[0].patch).unwrap().name,
            "DOOR3_6"
        );
        assert_eq!(exit_door.patches[1].x, 64);
        assert_eq!(exit_door.patches[1].y, 0);
        assert_eq!(
            patches.get(exit_door.patches[1].patch).unwrap().name,
            "DOOR3_4"
        );
        assert_eq!(exit_door.patches[2].x, 88);
        assert_eq!(exit_door.patches[2].y, 0);
        assert_eq!(
            patches.get(exit_door.patches[2].patch).unwrap().name,
            "DOOR3_5"
        );
        assert_eq!(exit_door.patches[3].x, 112);
        assert_eq!(exit_door.patches[3].y, 0);
        assert_eq!(
            patches.get(exit_door.patches[3].patch).unwrap().name,
            "T14_5"
        );
    }
}

use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ops::Index;
use std::sync::Arc;

use bytes::Buf;

use crate::assets::{Patch, PatchBank};
use crate::wad::{self, Cursor, Lump, Wad};

#[derive(Clone, Debug)]
pub struct TextureBank(BTreeMap<String, Arc<Texture>>);

impl TextureBank {
    /// Loads all the textures from a [`Wad`].
    ///
    /// Textures are listed in the `TEXTURE1` and `TEXTURE2` lumps.
    pub fn load(wad: &Wad) -> wad::Result<Self> {
        let patches = PatchBank::load(wad)?;
        let mut textures = BTreeMap::new();

        for lump in Self::texture_lumps(wad)? {
            Self::load_from(&lump, &mut textures, &patches)?;
        }

        Ok(Self(textures))
    }

    fn texture_lumps(wad: &Wad) -> wad::Result<Vec<Lump>> {
        let iter = Some(wad.lump("TEXTURE1")?).into_iter();
        let iter = iter.chain(wad.try_lump("TEXTURE2")?);
        Ok(iter.collect())
    }

    fn load_from(
        lump: &Lump,
        textures: &mut BTreeMap<String, Arc<Texture>>,
        patch_bank: &PatchBank,
    ) -> wad::Result<()> {
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
            let texture = Texture::load(lump, offset.try_into().unwrap(), patch_bank)?;
            textures.insert(texture.name().to_owned(), Arc::new(texture));
        }

        Ok(())
    }

    /// Retrieves a texture by name.
    pub fn get(&self, name: &str) -> Option<&Arc<Texture>> {
        self.0.get(name)
    }
}

impl Index<&str> for TextureBank {
    type Output = Arc<Texture>;

    fn index(&self, name: &str) -> &Self::Output {
        &self.0[name]
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    name: String,
    width: u16,
    height: u16,
    patches: Vec<PatchPlacement>,
}

impl Texture {
    fn load(lump: &Lump, offset: usize, patch_bank: &PatchBank) -> wad::Result<Self> {
        let mut cursor = lump.cursor();
        cursor.need(offset)?.advance(offset);

        let name = cursor.need(8)?.get_name();
        let _flags = cursor.need(2)?.get_u16_le();
        let _unused = cursor.need(2)?.get_u16_le();
        let width = cursor.need(2)?.get_u16_le();
        let height = cursor.need(2)?.get_u16_le();
        let _unused = cursor.need(4)?.get_u32_le();

        let patch_count = cursor.need(2)?.get_u16_le();
        let mut patches = Vec::with_capacity(patch_count.clamp(0, 64).into());
        for _ in 0..patch_count {
            patches.push(PatchPlacement::load(lump, &mut cursor, patch_bank)?);
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
struct PatchPlacement {
    x: u16,
    y: u16,
    patch: Arc<Patch>,
}

impl PatchPlacement {
    pub fn load(lump: &Lump, cursor: &mut Cursor, patches: &PatchBank) -> wad::Result<Self> {
        let x = cursor.need(2)?.get_u16_le();
        let y = cursor.need(2)?.get_u16_le();
        let patch_index = cursor.need(2)?.get_u16_le();
        let _unused = cursor.need(2)?.get_u16_le();
        let _unused = cursor.need(2)?.get_u16_le();

        let patch: &Arc<Patch> = patches.get(patch_index).map_err(|err| match err {
            None => lump.error(format!("bad patch index {}", patch_index)),
            Some(patch_name) => lump.error(format!("texture needs missing patch {}", patch_name)),
        })?;
        let patch = Arc::clone(patch);

        Ok(Self { x, y, patch })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wad::test::*;

    #[test]
    fn load() {
        let textures = TextureBank::load(&BIOTECH_WAD).unwrap();

        let exit_door = textures.get("EXITDOOR").unwrap();
        assert_eq!(exit_door.name(), "EXITDOOR");
        assert_eq!(exit_door.width(), 128);
        assert_eq!(exit_door.height(), 72);
        assert_eq!(exit_door.patches.len(), 4);
        assert_eq!(exit_door.patches[0].x, 0);
        assert_eq!(exit_door.patches[0].y, 0);
        assert_eq!(exit_door.patches[0].patch.name(), "DOOR3_6");
        assert_eq!(exit_door.patches[1].x, 64);
        assert_eq!(exit_door.patches[1].y, 0);
        assert_eq!(exit_door.patches[1].patch.name(), "DOOR3_4");
        assert_eq!(exit_door.patches[2].x, 88);
        assert_eq!(exit_door.patches[2].y, 0);
        assert_eq!(exit_door.patches[2].patch.name(), "DOOR3_5");
        assert_eq!(exit_door.patches[3].x, 112);
        assert_eq!(exit_door.patches[3].y, 0);
        assert_eq!(exit_door.patches[3].patch.name(), "T14_5");
    }
}

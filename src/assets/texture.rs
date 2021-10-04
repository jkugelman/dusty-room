use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ops::{Deref, Index};

use bytes::Buf;

use crate::wad::{self, Lump, Wad};

/// A bank of [`Texture`]s from the `TEXTURE1` and `TEXTURE2` lumps, indexed by name.
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

        cursor.need(4)?;
        let count = cursor.get_u32_le();

        // Read texture offsets. The WAD is untrusted so clamp how much memory is pre-allocated.
        // Don't worry about overflow converting from `u32` to `usize`. The wrong capacity won't
        // affect correctness.
        let mut offsets = Vec::with_capacity(count.clamp(0, 1024) as usize);
        cursor.need((count * 4).try_into().unwrap())?;

        for _ in 0..count {
            offsets.push(cursor.get_u32_le());
        }

        cursor.clear();
        cursor.done()?;

        // Read textures.
        for offset in offsets {
            let texture = Texture::load(lump, offset.try_into().unwrap())?;
            textures.insert(texture.name.clone(), texture);
        }

        Ok(())
    }

    /// Looks up a texture name. Case insensitive.
    pub fn get(&self, name: &str) -> Option<&Texture> {
        self.0.get(&name.to_ascii_uppercase())
    }
}

impl Index<&str> for TextureBank {
    type Output = Texture;

    /// Looks up a texture name. Case insensitive.
    fn index(&self, name: &str) -> &Self::Output {
        self.get(name).expect("texture not found")
    }
}

impl Deref for TextureBank {
    type Target = BTreeMap<String, Texture>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A wall texture drawn on the upper, lower, and middle areas of [sidedefs]. Each wall texture is
/// composed of one or more [patches] drawn at different offsets. Patches can be repeated, tiled,
/// and overlapped. Textures can also have transparent areas where no patches are drawn.
///
/// [sidedefs]: crate::map::Sidedef
/// [patches]: crate::assets::Patch
#[derive(Clone, Debug)]
pub struct Texture {
    /// Name of the texture. Used by [sidedefs].
    ///
    /// [sidedefs]: crate::map::Sidedef
    pub name: String,

    /// Total width in pixels.
    pub width: u16,

    /// Total height in pixels.
    pub height: u16,

    /// A list of patches and their X and Y offsets.
    patches: Vec<PatchPlacement>,
}

impl Texture {
    fn load(lump: &Lump, offset: usize) -> wad::Result<Self> {
        let mut cursor = lump.cursor();
        cursor.skip(offset)?;

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
}

#[derive(Clone, Debug)]
struct PatchPlacement {
    /// X offset of the patch on the texture's "canvas".
    pub x: u16,

    /// Y offset of the patch on the texture's "canvas".
    pub y: u16,

    /// Patch number to draw.
    pub patch: u16,
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
        assert_eq!(exit_door.name, "EXITDOOR");
        assert_eq!(exit_door.width, 128);
        assert_eq!(exit_door.height, 72);
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

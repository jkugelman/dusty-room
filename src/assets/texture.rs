use std::collections::BTreeMap;
use std::io::{Cursor, Seek, SeekFrom};
use std::ops::Index;
use std::sync::Arc;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::assets::{LoadError, Patch, PatchBank, ResultExt};
use crate::wad::{self, read_name, Lump, NameExt, Wad};

#[derive(Clone, Debug)]
pub struct TextureBank<'wad>(BTreeMap<&'wad str, Texture<'wad>>);

impl<'wad> TextureBank<'wad> {
    /// Loads all the textures from a [`Wad`].
    ///
    /// Textures are listed in the `TEXTURE1` and `TEXTURE2` lumps.
    pub fn load(wad: &'wad Wad) -> wad::Result<Self> {
        let patches = PatchBank::load(wad)?;
        let mut textures = BTreeMap::new();

        for lump in Self::texture_lumps(wad)? {
            Self::load_from(&lump, &mut textures, &patches)
                .explain(|| lump.error("bad texture data"))?;
        }

        Ok(Self(textures))
    }

    fn texture_lumps(wad: &'wad Wad) -> wad::Result<Vec<Lump<'wad>>> {
        let iter = Some(wad.lump("TEXTURE1")?).into_iter();
        let iter = iter.chain(wad.try_lump("TEXTURE2")?);
        Ok(iter.collect())
    }

    fn load_from(
        lump: &Lump<'wad>,
        textures: &mut BTreeMap<&'wad str, Texture<'wad>>,
        patch_bank: &PatchBank<'wad>,
    ) -> Result<(), LoadError> {
        let mut cursor = Cursor::new(lump.data());

        let count = cursor.read_u32::<LittleEndian>()?;

        // Read texture offsets. The WAD is untrusted so clamp how much memory is pre-allocated.
        // Don't worry about overflow converting from `u32` to `usize`. The wrong capacity won't
        // affect correctness.
        let mut offsets = Vec::with_capacity(count.clamp(0, 1024) as usize);
        for _ in 0..count {
            offsets.push(cursor.read_u32::<LittleEndian>()?);
        }

        // Read textures.
        for offset in offsets {
            cursor.seek(SeekFrom::Start(offset.into()))?;
            let texture = Texture::load(lump, &mut cursor, patch_bank)?;
            textures.insert(texture.name(), texture);
        }

        Ok(())
    }

    /// Retrieves a texture by name.
    pub fn get(&self, name: &str) -> Option<&Texture<'wad>> {
        self.0.get(name)
    }
}

impl<'wad> Index<&str> for TextureBank<'wad> {
    type Output = Texture<'wad>;

    fn index(&self, name: &str) -> &Self::Output {
        &self.0[name]
    }
}

#[derive(Clone, Debug)]
pub struct Texture<'wad> {
    name: &'wad str,
    width: u16,
    height: u16,
    patches: Vec<PatchPlacement<'wad>>,
}

impl<'wad> Texture<'wad> {
    fn load(
        lump: &Lump<'wad>,
        cursor: &mut Cursor<&'wad [u8]>,
        patch_bank: &PatchBank<'wad>,
    ) -> Result<Self, LoadError> {
        let name = match read_name(cursor)? {
            Ok(name) if name.is_legal() => Ok(name),
            Ok(name) => Err(lump.error(&format!("bad texture name {:?}", name))),
            Err(name) => Err(lump.error(&format!("bad texture name {:?}", name))),
        }?;

        let _flags = cursor.read_u16::<LittleEndian>()?;
        let _unused = cursor.read_u16::<LittleEndian>()?;
        let width = cursor.read_u16::<LittleEndian>()?;
        let height = cursor.read_u16::<LittleEndian>()?;
        let _unused = cursor.read_u32::<LittleEndian>()?;

        let patch_count = cursor.read_u16::<LittleEndian>()?;
        let mut patches = Vec::with_capacity(patch_count.clamp(0, 64).into());
        for _ in 0..patch_count {
            patches.push(PatchPlacement::load(lump, cursor, patch_bank)?);
        }

        Ok(Self {
            name,
            width,
            height,
            patches,
        })
    }

    /// The texture's name.
    pub fn name(&self) -> &'wad str {
        self.name
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
struct PatchPlacement<'wad> {
    x: u16,
    y: u16,
    patch: Arc<Patch<'wad>>,
}

impl<'wad> PatchPlacement<'wad> {
    pub fn load(
        lump: &Lump<'wad>,
        cursor: &mut Cursor<&[u8]>,
        patches: &PatchBank<'wad>,
    ) -> Result<Self, LoadError> {
        let x = cursor.read_u16::<LittleEndian>()?;
        let y = cursor.read_u16::<LittleEndian>()?;
        let patch_index = cursor.read_u16::<LittleEndian>()?;
        let _unused = cursor.read_u16::<LittleEndian>()?;
        let _unused = cursor.read_u16::<LittleEndian>()?;

        let patch: &Arc<Patch> = patches.get(patch_index).map_err(|err| -> LoadError {
            match err {
                None => LoadError::BadLump,
                Some(patch_name) => LoadError::Wad(
                    lump.error(&format!("texture needs missing patch {}", patch_name)),
                ),
            }
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

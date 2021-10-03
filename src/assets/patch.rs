use std::convert::TryInto;
use std::fmt;
use std::ops::Index;

use bytes::{Buf, Bytes};

use crate::wad::{self, Lump, Wad};

/// A bank of patches from the `PNAMES` lump.
///
/// The patches are all optional because sometimes `PNAMES` lists missing patches. The shareware
/// version of `doom.wad` is missing the `TEXTURE2` textures from the registered game, yet `PNAMES`
/// still lists all of the patches. It still loads because none of the textures in `TEXTURE1` use
/// the missing patches.
#[derive(Clone, Debug)]
pub struct PatchBank(Vec<(String, Option<Patch>)>);

impl PatchBank {
    /// Loads all the patches from a [`Wad`].
    ///
    /// Patch names are listed in the `PNAMES` lump, and each patch is loaded from the lump of that
    /// name.
    pub fn load(wad: &Wad) -> wad::Result<Self> {
        let lump = wad.lump("PNAMES")?;
        let mut cursor = lump.cursor();

        cursor.need(4)?;
        let count = cursor.get_u32_le();

        let mut patches = Vec::with_capacity(count.clamp(0, 1024) as usize);
        cursor.need((count * 8).try_into().unwrap())?;

        for _ in 0..count {
            let name = cursor.get_name();
            let lump = wad.try_lump(&name)?;
            let patch = lump.as_ref().map(Patch::load).transpose()?;
            patches.push((name, patch));
        }

        cursor.done()?;

        Ok(Self(patches))
    }

    /// The number of patches.
    pub fn len(&self) -> u16 {
        self.0.len().try_into().unwrap()
    }

    /// Returns `true` if there are no patches.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the patch at the specified index.
    ///
    /// # Errors
    ///
    /// Returns `Err(None)` if the index is out of range.
    ///
    /// Returns `Err(Some(name))` with the missing patch name if `PNAMES` lists the name of a
    /// missing patch, as happens with the shareware version of `doom.wad`.
    pub fn get(&self, index: u16) -> Result<&Patch, Option<&str>> {
        let (name, patch): &(String, Option<Patch>) = self.0.get(usize::from(index)).ok_or(None)?;
        patch.as_ref().ok_or(Some(name))
    }
}

impl Index<u16> for PatchBank {
    type Output = Patch;

    fn index(&self, index: u16) -> &Self::Output {
        self.0[usize::from(index)].1.as_ref().unwrap()
    }
}

/// An image that is used as the building block for a composite [texture].
///
/// [texture]: crate::assets::Texture
#[derive(Clone)]
pub struct Patch {
    /// Patch name.
    pub name: String,

    /// Width in pixels.
    pub width: u16,

    /// Height in pixels.
    pub height: u16,

    /// X offset. The number of pixels to the left of the center where the first column gets drawn.
    ///
    /// With wall patches this is always `width / 2 - 1`.
    pub x: i16,

    /// Y offset. The number of pixels above the origin where the top row gets drawn.
    ///
    /// With wall patches this is always `height - 5`.
    pub y: i16,

    columns: Vec<Column>,
}

#[derive(Clone)]
struct Column {
    posts: Vec<Post>,
}

#[derive(Clone)]
struct Post {
    y_offset: u16,
    pixels: Bytes,
}

impl Patch {
    /// Loads a patch from a lump.
    pub fn load(lump: &Lump) -> wad::Result<Self> {
        let mut cursor = lump.cursor();

        cursor.need(8)?;
        let name = lump.name().to_owned();
        let width = cursor.get_u16_le();
        let height = cursor.get_u16_le();
        let y = cursor.get_i16_le();
        let x = cursor.get_i16_le();

        // Read column offsets. The WAD is untrusted so clamp how much memory is pre-allocated.
        let mut column_offsets = Vec::with_capacity(width.clamp(0, 512).into());
        cursor.need((4 * width).into())?;

        for _ in 0..width {
            column_offsets.push(cursor.get_u32_le());
        }

        cursor.clear();
        cursor.done()?;

        // Read columns. The WAD is untrusted so clamp how much memory is pre-allocated.
        let mut columns = Vec::with_capacity(width.clamp(0, 512).into());
        for offset in column_offsets {
            columns.push(Self::read_column(lump, offset.try_into().unwrap())?);
        }

        Ok(Self {
            name,
            width,
            height,
            x,
            y,
            columns,
        })
    }

    fn read_column(lump: &Lump, offset: usize) -> wad::Result<Column> {
        let mut cursor = lump.cursor();
        cursor.skip(offset)?;

        let mut posts = Vec::new();
        let mut last_y_offset = None;

        loop {
            cursor.need(1)?;
            let y_offset = match (cursor.get_u8() as u16, last_y_offset) {
                // The end of the column is marked by an offset of 255.
                (255, _) => {
                    break;
                }

                // Handle so-called ["tall patches"]: Since posts are saved top to bottom, a
                // post's Y offset is normally greater than the last post's. If it's not
                // then we'll add them together. This enables Y offsets larger than the
                // traditional limit of 254.
                //
                // ["tall patches"]: https://doomwiki.org/wiki/Picture_format#Tall_patches
                (y_offset, Some(last_y_offset)) if y_offset <= last_y_offset => {
                    last_y_offset + y_offset
                }

                // The common case.
                (y_offset, _) => y_offset,
            };

            cursor.need(1)?;
            let length = cursor.get_u8() as usize;

            cursor.need(usize::from(length) + 2)?;
            let _unused = cursor.get_u8();
            let pixels = cursor.split_to(length);
            let _unused = cursor.get_u8();

            posts.push(Post { y_offset, pixels });
            last_y_offset = Some(y_offset);
        }

        cursor.clear();
        cursor.done()?;

        Ok(Column { posts })
    }
}

impl fmt::Debug for Patch {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            name,
            width,
            height,
            x,
            y,
            columns: _,
        } = self;

        fmt.debug_struct("Patch")
            .field("name", &name)
            .field("width", &width)
            .field("height", &height)
            .field("x", &x)
            .field("y", &y)
            .finish()
    }
}

impl fmt::Display for Patch {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} ({}x{})", self.name, self.width, self.height)
    }
}

impl fmt::Debug for Post {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let Self { y_offset, pixels } = self;

        fmt.debug_struct("Post")
            .field("y_offset", &y_offset)
            .field("height", &pixels.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wad::test::*;

    #[test]
    fn load() {
        let patches = PatchBank::load(&DOOM2_WAD).unwrap();

        assert_eq!(patches.len(), 469);
        assert_eq!(patches[69].name, "RW12_2");
        assert_eq!(patches[420].name, "RW25_3");

        // Did we find the lowercased `w94_1` patch?
        assert_eq!(patches[417].name, "W94_1");
        assert_eq!(patches[417].width, 128);
        assert_eq!(patches[417].height, 128);
        assert_eq!(patches[417].x, 123);
        assert_eq!(patches[417].y, 63);
    }

    #[test]
    fn missing() {
        let patches = PatchBank::load(&DOOM_WAD).unwrap();

        assert_matches!(patches.get(161), Ok(patch) if patch.name == "WALL24_1");
        assert_matches!(patches.get(162), Ok(patch) if patch.name == "W94_1");
        assert_matches!(patches.get(163), Err(Some("W104_1")));
        assert_matches!(patches.get(164), Err(Some("DOOR9_2")));
    }
}

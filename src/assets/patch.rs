use std::convert::TryInto;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::ops::{Deref, DerefMut};
use std::{fmt, slice, vec};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::assets::{LoadError, ResultExt};
use crate::wad::{self, Lump, Wad};

/// A patch is an image that is used as the building block for a composite [`Texture`].
///
/// [`Texture`]: crate::assets::Texture
#[derive(Debug, Clone)]
pub struct Patch<'wad> {
    name: String,
    width: u16,
    height: u16,
    top: i16,
    left: i16,
    columns: Vec<Column<'wad>>,
}

#[derive(Debug, Clone)]
struct Column<'wad> {
    posts: Vec<Post<'wad>>,
}

#[derive(Debug, Clone)]
struct Post<'wad> {
    y_offset: u16,
    pixels: &'wad [u8],
}

impl<'wad> Patch<'wad> {
    pub fn load(lump: &Lump<'wad>) -> wad::Result<Self> {
        // Emulate a [`try` block] with an [IIFE].
        // [`try` block]: https://doc.rust-lang.org/beta/unstable-book/language-features/try-blocks.html
        // [IIFE]: https://en.wikipedia.org/wiki/Immediately_invoked_function_expression
        (|| -> Result<Self, LoadError> {
            let mut cursor = Cursor::new(lump.data());

            let name = lump.name().to_owned();
            let width = cursor.read_u16::<LittleEndian>()?;
            let height = cursor.read_u16::<LittleEndian>()?;
            let top = cursor.read_i16::<LittleEndian>()?;
            let left = cursor.read_i16::<LittleEndian>()?;

            // Read column offsets. The WAD is untrusted so clamp how much memory is pre-allocated.
            let mut column_offsets = Vec::with_capacity(width.clamp(0, 512).into());
            for _ in 0..width {
                column_offsets.push(cursor.read_u32::<LittleEndian>()?);
            }

            // Read columns. The WAD is untrusted so clamp how much memory is pre-allocated.
            let mut columns = Vec::with_capacity(width.clamp(0, 512).into());
            for column_offset in column_offsets {
                cursor.seek(SeekFrom::Start(column_offset.into()))?;
                let column = Self::read_column(&mut cursor, lump)?;
                columns.push(column);
            }

            Ok(Self {
                name,
                width,
                height,
                top,
                left,
                columns,
            })
        })()
        .explain(|| lump.error("bad patch data"))
    }

    fn read_column(
        cursor: &mut Cursor<&[u8]>,
        lump: &Lump<'wad>,
    ) -> Result<Column<'wad>, LoadError> {
        let mut posts = Vec::new();
        let mut last_y_offset = None;

        loop {
            let y_offset = match (cursor.read_u8()? as u16, last_y_offset) {
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
            let length: usize = cursor.read_u8()?.into();

            // Skip unused byte.
            let _ = cursor.read_u8()?;

            // Save memory by having pixel data be a direct slice from the lump.
            let start: usize = cursor.position().try_into().unwrap();
            cursor.seek(SeekFrom::Current(length.try_into().unwrap()))?;
            let end: usize = cursor.position().try_into().unwrap();
            let pixels = &lump.data().get(start..end).ok_or(LoadError::OutOfBounds)?;

            // Skip unused byte.
            let _ = cursor.read_u8()?;

            posts.push(Post { y_offset, pixels });
            last_y_offset = Some(y_offset);
        }

        Ok(Column { posts })
    }

    /// The patch's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Width in pixels.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Top offset.
    pub fn top(&self) -> i16 {
        self.top
    }

    /// Left offset.
    pub fn left(&self) -> i16 {
        self.left
    }
}

impl fmt::Display for Patch<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} ({}x{})", self.name, self.width, self.height)
    }
}

/// A list of patches from the `PNAMES` lump.
///
/// The patches are all optional because sometimes `PNAMES` lists missing patches. The shareware
/// version of `doom.wad` is missing the `TEXTURE2` textures from the registered game, yet `PNAMES`
/// still lists all of the patches. It still loads because none of the textures in `TEXTURE1` use
/// the missing patches.
#[derive(Clone, Debug)]
pub struct PatchBank<'wad>(Vec<Option<Patch<'wad>>>);

impl<'wad> PatchBank<'wad> {
    /// Loads all the patches from a [`Wad`].
    ///
    /// Patch names are listed in the `PNAMES` lump, and each patch is loaded from the lump of that
    /// name.
    pub fn load(wad: &'wad Wad) -> wad::Result<Self> {
        let lump = wad.lump("PNAMES")?;

        // Emulate a [`try` block] with an [IIFE].
        // [`try` block]: https://doc.rust-lang.org/beta/unstable-book/language-features/try-blocks.html
        // [IIFE]: https://en.wikipedia.org/wiki/Immediately_invoked_function_expression
        (|| -> Result<Self, LoadError> {
            let mut cursor = Cursor::new(lump.data());

            let count = cursor.read_u32::<LittleEndian>()?;

            // The WAD is untrusted so clamp how much memory is pre-allocated. Don't worry about
            // overflow converting from `u32` to `usize`. The wrong capacity won't affect correctness.
            let mut patches = Vec::with_capacity(count.clamp(0, 1024) as usize);

            for _ in 0..count {
                let mut name = [0u8; 8];
                cursor.read_exact(&mut name)?;

                // Convert the name to uppercase like DOOM does. We have to emulate this because
                // `doom.wad` and `doom2.wad` include a lowercase `w94_1` in their `PNAMES`.
                name.make_ascii_uppercase();

                let name = Lump::read_raw_name(&name)
                    .map_err(|name| lump.error(&format!("contains bad lump name {:?}", name)))?;

                let lump = wad.try_lump(name)?;
                let patch = lump.as_ref().map(Patch::load).transpose()?;
                patches.push(patch);
            }

            Ok(Self(patches))
        })()
        .explain(|| lump.error("bad patch list data"))
    }
}

impl<'wad> Deref for PatchBank<'wad> {
    type Target = Vec<Option<Patch<'wad>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PatchBank<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'wad> IntoIterator for PatchBank<'wad> {
    type Item = Option<Patch<'wad>>;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, 'wad> IntoIterator for &'a PatchBank<'wad> {
    type Item = &'a Option<Patch<'wad>>;
    type IntoIter = slice::Iter<'a, Option<Patch<'wad>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'wad> IntoIterator for &'a mut PatchBank<'wad> {
    type Item = &'a mut Option<Patch<'wad>>;
    type IntoIter = slice::IterMut<'a, Option<Patch<'wad>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
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
        assert_eq!(patches[69].as_ref().unwrap().name(), "RW12_2");
        assert_eq!(patches[420].as_ref().unwrap().name(), "RW25_3");

        // Did we find the lowercased `w94_1` patch?
        assert_eq!(patches[417].as_ref().unwrap().name(), "W94_1");
    }

    #[test]
    fn missing() {
        let patches = PatchBank::load(&DOOM_WAD).unwrap();

        assert_matches!(patches[161], Some(_));
        assert_matches!(patches[162], Some(_));
        assert_matches!(patches[163], None);
        assert_matches!(patches[164], None);
    }
}

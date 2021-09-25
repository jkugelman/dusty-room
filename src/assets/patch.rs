use std::convert::TryInto;
use std::fmt;
use std::io::{BufRead, Cursor, Seek, SeekFrom};
use std::ops::Index;
use std::sync::Arc;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::assets::{LoadError, ResultExt};
use crate::wad::{self, read_name, Lump, Wad};

/// A list of patches from the `PNAMES` lump.
///
/// The patches are all optional because sometimes `PNAMES` lists missing patches. The shareware
/// version of `doom.wad` is missing the `TEXTURE2` textures from the registered game, yet `PNAMES`
/// still lists all of the patches. It still loads because none of the textures in `TEXTURE1` use
/// the missing patches.
#[derive(Clone, Debug)]
pub struct PatchBank<'wad>(Vec<(&'wad str, Option<Arc<Patch<'wad>>>)>);

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
            // overflow converting from `u32` to `usize`. The wrong capacity won't affect
            // correctness.
            let mut patches = Vec::with_capacity(count.clamp(0, 1024) as usize);

            for _ in 0..count {
                let name = read_name(&mut cursor)?
                    .map_err(|name| lump.error(&format!("contains bad lump name {:?}", name)))?;
                let lump = wad.try_lump(name)?;
                let patch = lump.as_ref().map(Patch::load).transpose()?.map(Arc::new);
                patches.push((name, patch));
            }

            Ok(Self(patches))
        })()
        .explain(|| lump.error("bad patch list data"))
    }

    /// The number of patches.
    pub fn len(&self) -> u16 {
        self.0.len().try_into().unwrap()
    }

    /// Returns the patch at the specified index.
    ///
    /// # Errors
    ///
    /// Returns `Err(None)` if the index is out of range.
    ///
    /// Returns `Err(Some(name))` if `PNAMES` had the name of a missing patch, as happens with the
    /// shareware version of `doom.wad`.
    pub fn get(&self, index: u16) -> Result<&Arc<Patch<'wad>>, Option<&'wad str>> {
        let (name, patch): &(&str, Option<Arc<Patch>>) =
            self.0.get(usize::from(index)).ok_or(None)?;
        Ok(patch.as_ref().ok_or(Some(*name))?)
    }
}

impl<'wad> Index<u16> for PatchBank<'wad> {
    type Output = Patch<'wad>;

    fn index(&self, index: u16) -> &Self::Output {
        self.0[usize::from(index)].1.as_ref().unwrap()
    }
}

/// A patch is an image that is used as the building block for a composite [`Texture`].
///
/// [`Texture`]: crate::assets::Texture
#[derive(Clone)]
pub struct Patch<'wad> {
    name: &'wad str,
    width: u16,
    height: u16,
    x: i16,
    y: i16,
    columns: Vec<Column<'wad>>,
}

#[derive(Debug, Clone)]
struct Column<'wad> {
    posts: Vec<Post<'wad>>,
}

#[derive(Clone)]
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

            let name = lump.name();
            let width = cursor.read_u16::<LittleEndian>()?;
            let height = cursor.read_u16::<LittleEndian>()?;
            let y = cursor.read_i16::<LittleEndian>()?;
            let x = cursor.read_i16::<LittleEndian>()?;

            // Read column offsets. The WAD is untrusted so clamp how much memory is pre-allocated.
            let mut column_offsets = Vec::with_capacity(width.clamp(0, 512).into());
            for _ in 0..width {
                column_offsets.push(cursor.read_u32::<LittleEndian>()?);
            }

            // Read columns. The WAD is untrusted so clamp how much memory is pre-allocated.
            let mut columns = Vec::with_capacity(width.clamp(0, 512).into());
            for column_offset in column_offsets {
                cursor.seek(SeekFrom::Start(column_offset.into()))?;
                let column = Self::read_column(lump, &mut cursor)?;
                columns.push(column);
            }

            Ok(Self {
                name,
                width,
                height,
                x,
                y,
                columns,
            })
        })()
        .explain(|| lump.error("bad patch data"))
    }

    fn read_column(
        lump: &Lump<'wad>,
        cursor: &mut Cursor<&[u8]>,
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
            let length = cursor.read_u8()?;
            let _unused = cursor.read_u8()?;

            // Save memory by having pixel data be a direct slice from the lump.
            let start = cursor.position() as usize;
            cursor.consume(length.into());
            let end = cursor.position() as usize;
            let pixels = &lump.data().get(start..end).ok_or(LoadError::BadLump)?;

            let _unused = cursor.read_u8()?;

            posts.push(Post { y_offset, pixels });
            last_y_offset = Some(y_offset);
        }

        Ok(Column { posts })
    }

    /// The patch's name.
    pub fn name(&self) -> &'wad str {
        self.name
    }

    /// Width in pixels.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Left offset.
    pub fn x(&self) -> i16 {
        self.x
    }

    /// Top offset.
    pub fn y(&self) -> i16 {
        self.y
    }
}

impl fmt::Debug for Patch<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Patch")
            .field("name", &self.name)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}

impl fmt::Display for Patch<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} ({}x{})", self.name, self.width, self.height)
    }
}

impl fmt::Debug for Post<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Post")
            .field("y_offset", &self.y_offset)
            .field("height", &self.pixels.len())
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
        assert_eq!(patches[69].name(), "RW12_2");
        assert_eq!(patches[420].name(), "RW25_3");

        // Did we find the lowercased `w94_1` patch?
        assert_eq!(patches[417].name(), "W94_1");
        assert_eq!(patches[417].width(), 128);
        assert_eq!(patches[417].height(), 128);
        assert_eq!(patches[417].x(), 123);
        assert_eq!(patches[417].y(), 63);
    }

    #[test]
    fn missing() {
        let patches = PatchBank::load(&DOOM_WAD).unwrap();

        assert_matches!(patches.get(161), Ok(patch) if patch.name() == "WALL24_1");
        assert_matches!(patches.get(162), Ok(patch) if patch.name() == "W94_1");
        assert_matches!(patches.get(163), Err(Some("W104_1")));
        assert_matches!(patches.get(164), Err(Some("DOOR9_2")));
    }
}

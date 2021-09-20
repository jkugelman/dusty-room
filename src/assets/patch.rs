use std::convert::TryInto;
use std::fmt;
use std::io::{self, Cursor, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::wad::{self, Lump};

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
        (|| -> io::Result<Self> {
            let mut cursor = Cursor::new(lump.data());

            let name = lump.name().to_owned();
            let width = cursor.read_u16::<LittleEndian>()?;
            let height = cursor.read_u16::<LittleEndian>()?;
            let top = cursor.read_i16::<LittleEndian>()?;
            let left = cursor.read_i16::<LittleEndian>()?;

            // Read column offsets.
            let mut column_offsets = Vec::with_capacity(width.into());
            for _ in 0..width {
                column_offsets.push(cursor.read_u32::<LittleEndian>()?);
            }

            // Read columns.
            let mut columns = Vec::with_capacity(width.into());

            for column_offset in column_offsets {
                cursor.seek(SeekFrom::Start(column_offset.into()))?;

                // Read posts.
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
                    use io::ErrorKind::UnexpectedEof;
                    let pixels = &lump.data().get(start..end).ok_or(UnexpectedEof)?;

                    // Skip unused byte.
                    let _ = cursor.read_u8()?;

                    posts.push(Post { y_offset, pixels });
                    last_y_offset = Some(y_offset);
                }

                columns.push(Column { posts });
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
        .map_err(|_| lump.error("bad patch data"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wad::test::*;

    #[test]
    fn load() {
        let lumps = DOOM2_WAD.lumps_between("P_START", "P_END").unwrap();

        for lump in lumps.into_iter().filter(Lump::has_data) {
            Patch::load(&lump).expect("failed to load");
        }
    }
}

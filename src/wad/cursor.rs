use std::convert::TryInto;
use std::ops::{Deref, DerefMut};

use bytes::{Buf, Bytes};

use crate::wad::{self, parse_name, Lump};

/// A moving cursor for reading data from a [`Lump`]. `Cursor` is a thin wrapper around [`Bytes`]
/// that allows for checking if there's data available before reading it.
///
/// It is important to always call [`done`] when when parsing is finished to ensure there is no
/// extra trailing data. You can [`clear`] the cursor if trailing data is expected.
///
/// # Examples
///
/// Read a 12-byte lump containing a 4-byte number and an 8-byte name:
///
/// ```no_run
/// # use bytes::Buf;
/// # let lump = dusty_room::wad::Wad::load("")?.lump("")?;
/// #
/// let mut cursor = lump.cursor();
///
/// cursor.need(12)?;
/// let value = cursor.get_u32_le();
/// let name = cursor.get_name();
///
/// cursor.done()?;
/// #
/// # Ok::<(), dusty_room::wad::Error>(())
/// ```
///
/// Ignore unread trailing data:
///
/// ```no_run
/// # use bytes::Buf;
/// # let lump = dusty_room::wad::Wad::load("")?.lump("")?;
/// # let mut cursor = lump.cursor();
/// #
/// cursor.clear();
/// cursor.done()?;
/// #
/// # Ok::<(), dusty_room::wad::Error>(())
///
/// ```
///
/// [`done`]: Self::done
/// [`clear`]: Bytes::clear
pub struct Cursor<'lump> {
    lump: &'lump Lump,
    data: Bytes,
}

impl<'lump> Cursor<'lump> {
    pub(super) fn new(lump: &'lump Lump, data: Bytes) -> Self {
        Self { lump, data }
    }
}

impl Cursor<'_> {
    /// Checks that there are at least `size` bytes remaining. Always call this before reading
    /// anything as [`Bytes`]'s methods will panic if there is insufficient data.
    pub fn need(&self, size: usize) -> wad::Result<()> {
        if self.len() >= size {
            Ok(())
        } else {
            Err(self.lump.error("not enough data"))
        }
    }

    /// Checks that there are at least `count` bytes remaining, then calls `self.advance(count)`.
    pub fn skip(&mut self, count: usize) -> wad::Result<()> {
        self.need(count)?;
        self.advance(count);
        Ok(())
    }

    /// Checks if there is unread data, then drops the cursor. This function should always be called
    /// when parsing is finished to ensure there is no extra trailing data. You can [`clear`] the
    /// cursor if trailing data is expected.
    ///
    /// [`clear`]: Bytes::clear
    pub fn done(self) -> wad::Result<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self.lump.error("too much data"))
        }
    }

    /// Reads an 8-byte, NUL padded name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # let lump = dusty_room::wad::Wad::load("")?.lump("")?;
    /// # let mut cursor = lump.cursor();
    /// #
    /// cursor.need(8)?;
    /// let name = cursor.get_name();
    /// #
    /// # Ok::<(), dusty_room::wad::Error>(())
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if there are fewer than 8 bytes remaining.
    pub fn get_name(&mut self) -> String {
        parse_name(self.split_to(8).as_ref().try_into().unwrap())
    }
}

impl Deref for Cursor<'_> {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Cursor<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

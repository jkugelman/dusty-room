use std::borrow::Cow;
use std::convert::TryInto;
use std::ops::{Deref, DerefMut};

use bytes::{Buf, Bytes};

use crate::wad::{self, parse_name, Lump};

/// A thin wrapper around [`Bytes`] that allows for checking if there's data available before
/// reading it.
pub struct Cursor<'lump> {
    lump: &'lump Lump,
    data: Bytes,
    done: bool,
}

impl<'lump> Cursor<'lump> {
    pub(super) fn new(lump: &'lump Lump, data: Bytes) -> Self {
        Self {
            lump,
            data,
            done: false,
        }
    }
}

impl Cursor<'_> {
    /// Checks that there are at least `size` bytes remaining.
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

    /// Checks if there is unread data, then drops the cursor. This function **must** be called when
    /// parsing is finished. If the cursor is dropped without calling `done` it will panic. You can
    /// [`clear`] the cursor if you don't care if there's unread data.
    ///
    /// [`clear`]: Bytes::clear
    ///
    /// # Examples
    ///
    /// Check if there is unread data:
    ///
    /// ```no_run
    /// let value = cursor.need(4)?.get_u32();
    /// cursor.done()?;
    /// ```
    ///
    /// Ignore unread data:
    ///
    /// ```no_run
    /// let value = cursor.need(4)?.get_u32();
    /// cursor.clear();
    /// cursor.done()?;
    /// ```
    pub fn done(mut self) -> wad::Result<()> {
        self.done = true;

        if self.is_empty() {
            Ok(())
        } else {
            Err(self.lump.error("too much data"))
        }
    }

    /// Reads an 8-byte, NUL padded name.
    ///
    /// The caller is responsible for calling `self.need(8)?`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let name: String = cursor.need(8)?.get_name();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if there are fewer than 8 bytes remaining.
    pub fn get_name(&mut self) -> String {
        parse_name(self.split_to(8).as_ref().try_into().unwrap())
    }

    /// Creates a [`wad::Error::Malformed`] blaming this cursor's lump.
    pub fn error(&self, desc: impl Into<Cow<'static, str>>) -> wad::Error {
        self.lump.error(desc)
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

impl Drop for Cursor<'_> {
    /// # Panics
    ///
    /// Panics if you forgot to call `self.done()?`.
    fn drop(&mut self) {
        assert!(self.done, "did not call cursor.done()");
    }
}

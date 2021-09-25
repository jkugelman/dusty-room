use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use std::{fmt, slice, vec};

use crate::wad::{self, WadFile};

/// A lump of data from a [`Wad`] or [`WadFile`].
///
/// Lumps are cheap to create as they simply borrow a slice of data from their WAD file. Cloning
/// should be treated as an expensive operation, though. The plan is to eventually make lumps
/// mutable by making the data slice [copy-on-write][`Cow`], which would make cloning modified lumps
/// expensive.
///
/// [`Wad`]: crate::wad::Wad
#[derive(Clone)]
pub struct Lump<'wad> {
    file: &'wad WadFile,
    name: &'wad str,
    data: &'wad [u8],
}

impl<'wad> Lump<'wad> {
    /// Creates a lump pointing at a slice of data from a `WadFile`.
    pub(super) fn new(file: &'wad WadFile, name: &'wad str, data: &'wad [u8]) -> Self {
        Self { file, name, data }
    }

    /// Reads a lump name from a raw 8-byte, NUL padded byte array.
    ///
    /// This function strips trailing NULs and verifies that all characters are legal. Legal
    /// characters are ASCII letters `A-Z`, digits `0-9`, and any of the punctuation `[]-_\`.
    ///
    /// # Errors
    ///
    /// If the name contains any illegal characters it is still converted into a string but is
    /// returned as an `Err` instead.
    pub fn read_raw_name(raw: &[u8; 8]) -> Result<&str, String> {
        let mut legal = true;
        let mut i = 0;

        while i < raw.len() {
            match raw[i] {
                b'\0' => break,
                b'A'..=b'Z' | b'0'..=b'9' | b'[' | b']' | b'-' | b'_' | b'\\' => {}
                _ => {
                    legal = false;
                }
            }

            i += 1;
        }

        if i > 0 && legal {
            // SAFETY: We've verified that there are only ASCII characters, which are by definition
            // valid UTF-8.
            Ok(unsafe { std::str::from_utf8_unchecked(&raw[..i]) })
        } else {
            // Convert the name into a string. It might not be valid UTF-8 so don't bother with
            // `std::str::from_utf8`. Instead treat it as Latin-1, i.e. all bytes are valid and map
            // 1-to-1 to the corresponding Unicode codepoints.
            Err(raw[..i].iter().map(|&b| b as char).collect::<String>())
        }
    }

    /// The file containing the lump.
    pub fn file(&self) -> &'wad WadFile {
        self.file
    }

    /// The lump name, for example `VERTEXES` or `THINGS`.
    pub fn name(&self) -> &'wad str {
        self.name
    }

    /// The lump data, a binary blob.
    pub fn data(&self) -> &'wad [u8] {
        self.data
    }

    /// The size of the lump.
    ///
    /// This is equivalent to `self.data().len()`.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if this is a marker lump with no data.
    ///
    /// This is equivalent to `self.data.len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns `true` if the lump contains data.
    ///
    /// This is equivalent to `!self.is_empty()`.
    pub fn has_data(&self) -> bool {
        !self.is_empty()
    }

    /// Checks that the lump has the expected name.
    pub fn expect_name(&self, name: &str) -> wad::Result<&Self> {
        if self.name == name {
            Ok(self)
        } else {
            Err(self.error(&format!("{} missing", name)))
        }
    }

    /// Checks that the lump is the expected number of bytes.
    pub fn expect_size(&self, size: usize) -> wad::Result<&Self> {
        if self.size() == size {
            Ok(self)
        } else {
            Err(self.error(&format!("expected {} bytes, got {}", size, self.size())))
        }
    }

    /// Checks that the lump contains a multiple of `size` bytes.
    pub fn expect_size_multiple(&self, size: usize) -> wad::Result<&Self> {
        if self.size() % size == 0 {
            Ok(self)
        } else {
            Err(self.error(&format!(
                "expected a multiple of {} bytes, got {}",
                size,
                self.size()
            )))
        }
    }

    /// Creates a [`wad::Error::Malformed`] blaming this lump.
    pub fn error(&self, desc: &str) -> wad::Error {
        self.file.error(format!("{}: {}", self.name(), desc))
    }
}

impl fmt::Debug for Lump<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{} ({} bytes) from {}",
            self.name,
            self.size(),
            self.file,
        )
    }
}

impl fmt::Display for Lump<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

/// A block of one or more [`Lump`]s from a [`Wad`] or [`WadFile`].
///
/// Usually the first lump gives the name of the block.
///
/// Lumps are cheap to create as they simply borrow a slice of data from their WAD file. Cloning
/// should be treated as an expensive operation, though. The plan is to eventually make lumps
/// mutable by making these references [copy-on-write][`Cow`], which would make cloning modified
/// lumps expensive.
///
/// [`Wad`]: crate::wad::Wad
#[derive(Clone, Debug)]
pub struct Lumps<'wad>(Vec<Lump<'wad>>);

impl<'wad> Lumps<'wad> {
    /// Creates a block of lumps.
    ///
    /// # Panics
    ///
    /// Panics if `lumps` is empty.
    pub(super) fn new(lumps: Vec<Lump<'wad>>) -> Self {
        assert!(!lumps.is_empty());
        Self(lumps)
    }

    /// The file containing the lumps.
    pub fn file(&self) -> &'wad WadFile {
        // It doesn't matter which lump we look at. They all come from the same file.
        self.first().file
    }

    /// The name of the first lump.
    pub fn name(&self) -> &'wad str {
        self.first().name
    }

    /// The first lump in the block.
    pub fn first(&self) -> &Lump<'wad> {
        self.0.first().unwrap()
    }

    /// The last lump in the block.
    pub fn last(&self) -> &Lump<'wad> {
        self.0.last().unwrap()
    }

    /// Creates a [`wad::Error::Malformed`] blaming this block.
    pub fn error(&self, desc: impl Into<Cow<'static, str>>) -> wad::Error {
        self.first().file.error(desc)
    }
}

impl<'wad> Deref for Lumps<'wad> {
    type Target = Vec<Lump<'wad>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Lumps<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'wad> IntoIterator for Lumps<'wad> {
    type Item = Lump<'wad>;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, 'wad> IntoIterator for &'a Lumps<'wad> {
    type Item = &'a Lump<'wad>;
    type IntoIter = slice::Iter<'a, Lump<'wad>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'wad> IntoIterator for &'a mut Lumps<'wad> {
    type Item = &'a mut Lump<'wad>;
    type IntoIter = slice::IterMut<'a, Lump<'wad>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
